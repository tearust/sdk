use std::{fmt::Debug, mem::size_of, sync::Arc};

use tea_actorx_core::{
	actor::{decode_invoke, decode_output, encode_input, InputMessageKind, OutputMessageKind},
	billing::{self, GasFeeCostRequest},
	hook::Deactivate,
	ActorId, InstanceId,
};
use tea_actorx_signer::{verify, Claim, Metadata};
use tea_codec::serde::{get_type_id, FromBytes, ToBytes, TypeId};
use tokio::sync::Mutex;
use wasmer::{
	imports, wasmparser::Operator, CompilerConfig, Cranelift, EngineBuilder, Extern, Function,
	FunctionEnv, FunctionEnvMut, Imports, Instance, Memory, Module, ModuleMiddleware, Store,
	TypedFunction,
};
use wasmer_middlewares::{
	metering::{get_remaining_points, set_remaining_points},
	Metering,
};

use crate::{
	actor::{looped::ActorFactory, ActorAgent, ActorContext, DynFuture},
	error::{AccessNotPermitted, Error},
	error::{GasFeeExhausted, Result},
};

use self::memory::MemoryLimit;

use super::shared;

mod memory;

pub const FUSE_GAS: u64 = u64::MAX;

const PAGE_BYTES: u32 = 64 * 1024;
const MAX_PAGES: u32 = 65536;

pub struct WasmActorFactory {
	metadata: Arc<Metadata>,
	metering: Box<dyn Fn() -> Arc<dyn ModuleMiddleware> + Send + Sync>,
	wasm: Vec<u8>,
	memory_limit: Option<u64>,
}

impl WasmActorFactory {
	pub fn new<C>(
		cost: C,
		wasm: Vec<u8>,
		memory_limit: Option<u64>,
	) -> Result<(Self, Arc<Metadata>)>
	where
		C: Fn(&Operator) -> u64 + Clone + Send + Sync + 'static,
	{
		let metadata = Arc::new(verify(&wasm)?);
		let metering = Box::new(move || Arc::new(Metering::new(0, cost.clone())) as _);
		Ok((
			Self {
				metering,
				wasm,
				metadata: metadata.clone(),
				memory_limit,
			},
			metadata,
		))
	}
}

unsafe fn as_static_mut<T>(v: &mut T) -> &'static mut T {
	&mut *(v as *mut T)
}

impl ActorFactory for WasmActorFactory {
	fn create(&self, context: ActorContext) -> DynFuture<Result<crate::actor::ActorAgent>> {
		Box::pin(async move {
			let metering = (self.metering)();
			let mut compiler_config = Cranelift::default();
			compiler_config.push_middleware(metering);

			let memory_limit = MemoryLimit::new(
				self.memory_limit
					.map(|x| (x / PAGE_BYTES as u64).max(MAX_PAGES as _) as u32)
					.unwrap_or(MAX_PAGES),
			);

			let mut store =
				Store::new_with_tunables(EngineBuilder::new(compiler_config), memory_limit);
			let module = Module::new(&store, &self.wasm)?;
			let mut actor = Box::<WasmActor>::new_uninit();
			let print = Self::create_print(&mut store, unsafe {
				as_static_mut(actor.assume_init_mut())
			});
			let mut imports = imports! {
				"env" => {
					"print" => print
				},
			};
			Self::wasm_bindgen_polyfill(&mut store, &mut imports);
			let instance = Instance::new(&mut store, &module, &imports)?;
			let init = instance
				.exports
				.get_typed_function::<(), ()>(&store, "init");
			let memory = instance.exports.get_memory("memory")?.clone();
			let init_handle = instance.exports.get_typed_function(&store, "init_handle")?;
			let handler = instance.exports.get_typed_function(&store, "handle")?;
			let finish_handle = instance
				.exports
				.get_typed_function(&store, "finish_handle")?;

			let print_handler = context.host.upgrade().unwrap().wasm_print_handler().await;

			let actor = unsafe {
				actor.as_mut_ptr().write(WasmActor {
					metadata: self.metadata.clone(),
					context: context.clone(),
					store: Mutex::new(store),
					module,
					print_handler,
					instance,
					memory,
					init_handle,
					handler,
					finish_handle,
				});
				actor.assume_init()
			};

			if let Ok(init) = init {
				init.call(&mut *actor.store.lock().await)?;
			}

			Ok(ActorAgent::shared(actor, context))
		})
	}

	fn metadata(&self) -> Option<&Arc<Metadata>> {
		Some(&self.metadata)
	}
}

impl WasmActorFactory {
	fn create_print(store: &mut Store, actor: &'static mut WasmActor) -> Function {
		let print_env = FunctionEnv::new(store, actor);
		wasmer::Function::new_typed_with_env(store, &print_env, Self::print)
	}

	fn print(mut env: FunctionEnvMut<&'static mut WasmActor>, ptr: u32, len: u32) {
		let actor = &mut **env.data_mut();
		let memory = actor.memory.view(&actor.store.get_mut());
		let mut data = Vec::with_capacity(len as _);
		unsafe {
			data.set_len(len as _);
		}
		let result = (|| {
			let data = memory.read_uninit(ptr as _, &mut data)?;
			let data = std::str::from_utf8(data)?;
			(actor.print_handler)(data);
			Ok(()) as Result<()>
		})();

		if let Err(e) = result {
			error!("falled to process logging: {e}");
		}
	}

	fn wasm_bindgen_polyfill(store: &mut Store, imports: &mut Imports) {
		fn panic() -> ! {
			unimplemented!("calling with wasm-bindgen is not supported")
		}

		fn _1to0(_: i32) {
			panic()
		}
		fn _2to0(_: i32, _: i32) {
			panic()
		}
		fn _1to1(_: i32) -> i32 {
			panic()
		}

		imports.register_namespace(
			"__wbindgen_placeholder__",
			[
				("__wbindgen_describe", Function::new_typed(store, _1to0)),
				("__wbindgen_throw", Function::new_typed(store, _2to0)),
			]
			.into_iter()
			.map(|(name, f)| (name.to_string(), Extern::Function(f))),
		);

		imports.register_namespace(
			"__wbindgen_externref_xform__",
			[
				(
					"__wbindgen_externref_table_grow",
					Function::new_typed(store, _1to1),
				),
				(
					"__wbindgen_externref_table_set_null",
					Function::new_typed(store, _1to0),
				),
			]
			.into_iter()
			.map(|(name, f)| (name.to_string(), Extern::Function(f))),
		);
	}
}

struct WasmActor {
	metadata: Arc<Metadata>,
	context: ActorContext,
	store: Mutex<Store>,
	module: Module,
	print_handler: Arc<dyn Fn(&str) + Send + Sync>,
	instance: Instance,
	memory: Memory,
	init_handle: TypedFunction<u32, u32>,
	handler: TypedFunction<(u32, u32), u32>,
	finish_handle: TypedFunction<(u32, u32), ()>,
}

impl WasmActor {
	async fn invoke(
		&self,
		msg: Vec<u8>,
		kind: InputMessageKind,
		caller: Option<ActorId>,
		mut local_gas: Option<&mut u64>,
	) -> Result<Vec<u8>> {
		let mut result = unsafe {
			unlock_lifetime(self)
				.inner_invoke(
					encode_input(kind, None, caller, &msg)?,
					local_gas.as_deref_mut(),
				)
				.await
		}?;
		loop {
			let (kind, quote_id, payload) = decode_output(&result)?;
			match kind {
				OutputMessageKind::HostCall | OutputMessageKind::HostPost => {
					let (target_id, msg) = decode_invoke(payload)?;

					if !self.metadata.claims.iter().any(|x| {
						if let Claim::ActorAccess(id) = x {
							target_id.reg == id.as_slice()
						} else {
							self.context.id.reg == target_id.reg
						}
					}) {
						return Err(AccessNotPermitted(target_id.reg.to_owned()).into());
					}

					let msg = async {
						let actor = self
							.context
							.host
							.registry(&target_id.reg)?
							.actor(&target_id.inst)
							.await?;

						if let OutputMessageKind::HostCall = kind {
							actor
								.call_with_caller_raw(msg.to_vec(), Some(self.context.id.clone()))
								.await
						} else {
							actor
								.post_with_caller_raw(msg.to_vec(), Some(self.context.id.clone()))
								.map(|_| Vec::new())
						}
					}
					.await;

					let msg = match msg {
						Ok(msg) => {
							encode_input(InputMessageKind::HostReturn, quote_id, None, &msg)?
						}
						Err(err) => encode_input(
							InputMessageKind::HostError,
							quote_id,
							None,
							&err.to_bytes()?,
						)?,
					};

					result = unsafe { self.inner_invoke(msg, local_gas.as_deref_mut()).await }?;
				}

				OutputMessageKind::GuestReturn => return Ok(payload.to_vec()),
				OutputMessageKind::GuestError => return Err(Error::from_bytes(payload)?),
			}
		}
	}

	async unsafe fn inner_invoke(
		&self,
		msg: Vec<u8>,
		local_gas: Option<&mut u64>,
	) -> Result<Vec<u8>> {
		let mut store = self.store.lock().await;
		let store = &mut *store;

		set_remaining_points(
			store,
			&self.instance,
			local_gas.as_deref().copied().unwrap_or(u64::MAX),
		);
		let result = self.do_inner_invoke(store, msg);
		match get_remaining_points(store, &self.instance) {
			wasmer_middlewares::metering::MeteringPoints::Remaining(g) => {
				if let Some(local_gas) = local_gas {
					*local_gas = g;
				}
				result
			}
			wasmer_middlewares::metering::MeteringPoints::Exhausted => {
				if let Some(local_gas) = local_gas {
					*local_gas = 0;
				}
				Err(GasFeeExhausted::Wasm(self.context.id.clone()).into())
			}
		}
	}

	#[allow(clippy::uninit_vec)]
	unsafe fn do_inner_invoke(&self, store: &mut Store, msg: Vec<u8>) -> Result<Vec<u8>> {
		// marshalling input message
		let len = msg.len();
		let ptr = self.init_handle.call(store, len as _)?;
		let view = self.memory.view(store);
		view.write(ptr as _, &msg)?;

		// call handle
		let result = self.handler.call(store, ptr, len as _)?;

		// marshalling output messsage
		let view = self.memory.view(store);
		let mut len = [0u8; size_of::<u32>()];
		view.read(result as _, &mut len)?;
		let len = u32::from_le_bytes(len) as usize;
		let mut output = Vec::with_capacity(len + size_of::<u32>());
		output.set_len(len + size_of::<u32>());
		let read = view.read(result as _, output.as_mut_slice());
		let finish = self.finish_handle.call(store, result, len as _);
		read?;
		finish?;

		Ok(output)
	}
}

impl shared::Actor for Box<WasmActor> {
	fn kind(&self) -> super::ActorKind {
		super::ActorKind::Wasm
	}

	fn invoke(&self, msg: Vec<u8>, caller: Option<ActorId>) -> DynFuture<Result<Vec<u8>, Error>> {
		Box::pin(async move {
			let mut local_gas = if get_type_id(&msg) == Ok(Deactivate::TYPE_ID) {
				None
			} else {
				Some(FUSE_GAS)
			};

			let result = WasmActor::invoke(
				self,
				msg,
				InputMessageKind::GuestCall,
				caller,
				local_gas.as_mut(),
			)
			.await;

			if let Some(local_gas) = local_gas {
				let usage = FUSE_GAS - local_gas;
				if usage > 0 {
					send_cost(&self.context, usage).await?;
				}
			}

			result
		})
	}
}

pub(crate) async fn send_cost(context: &ActorContext, cost: u64) -> Result<()> {
	if let Ok(registry) = context.host.registry(billing::NAME) {
		registry
			.actor(&InstanceId::ZERO)
			.await?
			.post_with_caller(GasFeeCostRequest(cost), Some(context.id.clone()))?;
	} else {
		warn!(
			"No billing actor registered, gas fee {cost} from {} ignored.",
			context.id
		);
	}

	Ok(())
}

impl Debug for WasmActor {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("WasmActor")
			.field("metadata", &self.metadata)
			.field("context", &self.context)
			.field("store", &self.store)
			.field("module", &self.module)
			.finish()
	}
}

unsafe fn unlock_lifetime<T>(v: &T) -> &'static T {
	&*(v as *const T)
}
