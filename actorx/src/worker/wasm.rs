use std::{
	collections::hash_map::DefaultHasher,
	hash::{Hash, Hasher},
	io::{stderr, stdout, Write},
	mem::{size_of, MaybeUninit},
	sync::Arc,
};

#[cfg(feature = "metering")]
use crate::core::error::GasFeeExhausted;
use crate::{
	core::{
		metadata::Metadata,
		wasm::{pricing, MEMORY_LIMIT},
		worker_codec::{read_var_bytes, write_var_bytes, Operation, OperationAbi},
	},
	sign::verify,
	worker::{
		error::{Error, Result},
		wasm::memory::MemoryLimit,
	},
};
use bincode::serialized_size;
use tokio::{fs, sync::RwLock};
use wasmer::{
	imports, EngineBuilder, Extern, Function, FunctionEnv, FunctionEnvMut, Imports,
	Instance as WasmInstance, Memory, Module, Store, TypedFunction,
};
#[cfg(feature = "metering")]
use ::{
	wasmer::CompilerConfig,
	wasmer_middlewares::{
		metering::{get_remaining_points, set_remaining_points},
		Metering,
	},
};

use super::threading::execute;

mod memory;

const CACHE_DIR: &str = ".cache";

const PAGE_BYTES: u32 = 64 * 1024;
const MAX_PAGES: u32 = 65536;

pub struct Host {
	source: Arc<[u8]>,
	compiled_path: String,
	state: RwLock<State>,
}

struct State {
	metadata: Arc<Metadata>,
	wasm: Arc<[u8]>,
}

impl Host {
	pub async fn new(source: Vec<u8>) -> Result<Self> {
		let mut hasher = DefaultHasher::new();
		source.hash(&mut hasher);
		let hash = hasher.finish();
		let compiled_path = format!("{CACHE_DIR}/{hash:x}");
		let result = Self {
			source: source.into(),
			compiled_path,
			state: RwLock::new(State {
				metadata: Arc::new(Metadata::EMPTY),
				wasm: Arc::new([]),
			}),
		};

		if result.read_cache().await.is_err() {
			result.read_new().await?;
		}

		Ok(result)
	}

	pub async fn metadata(&self) -> Arc<Metadata> {
		self.state.read().await.metadata.clone()
	}

	async fn read_cache(&self) -> Result<()> {
		let mut file = fs::OpenOptions::new()
			.read(true)
			.open(&self.compiled_path)
			.await?;
		let metadata = read_var_bytes(&mut file).await?;
		let metadata = bincode::deserialize(&metadata)?;
		let wasm = read_var_bytes(&mut file).await?;
		let mut state = self.state.write().await;
		state.metadata = metadata;
		state.wasm = wasm.into();
		Ok(())
	}

	pub(crate) async fn read_new(&self) -> Result<()> {
		let metadata = Arc::new(verify(&self.source)?);
		let source = self.source.clone();
		let module_bytes = execute(|| {
			let store = create_store();
			let module = Module::new(&store, source)?;
			module.serialize().map_err(Error::from)
		})
		.await
		.await??;
		_ = fs::create_dir(CACHE_DIR).await;
		let mut file = fs::OpenOptions::new()
			.write(true)
			.create(true)
			.open(&self.compiled_path)
			.await?;
		let metadata_bytes = bincode::serialize(&metadata)?;
		write_var_bytes(&mut file, &metadata_bytes).await?;
		write_var_bytes(&mut file, &module_bytes).await?;
		let mut state = self.state.write().await;
		state.metadata = metadata;
		state.wasm = (&*module_bytes).into();
		Ok(())
	}

	async fn crate_wasm(&self) -> Result<(Store, Module, Arc<Metadata>)> {
		let state = self.state.read().await;
		let metadata = state.metadata.clone();
		let wasm = state.wasm.clone();

		execute(move || {
			let store = create_store();
			let module = unsafe { Module::deserialize(&store, &*wasm)? };
			Ok((store, module, metadata))
		})
		.await
		.await?
	}

	pub async fn create_instance(&self) -> Result<Instance> {
		let (mut store, module, metadata) = self.crate_wasm().await?;
		execute(move || {
			let mut instance_state = Box::new(MaybeUninit::uninit());
			let print = create_print(&mut store, unsafe {
				as_static_mut(instance_state.assume_init_mut())
			});
			let mut imports = imports! {
				"env" => {
					"print" => print
				},
			};
			wasm_bindgen_polyfill(&mut store, &mut imports);
			let instance = WasmInstance::new(&mut store, &module, &imports)?;
			let init = instance.exports.get_typed_function(&store, "init")?;
			let memory = instance.exports.get_memory("memory")?.clone();
			let init_handle = instance.exports.get_typed_function(&store, "init_handle")?;
			let handler = instance.exports.get_typed_function(&store, "handle")?;
			let finish_handle = instance
				.exports
				.get_typed_function(&store, "finish_handle")?;

			let instance = Instance(unsafe {
				instance_state.as_mut_ptr().write(InstanceState {
					metadata,
					store,
					_module: module,
					instance,
					memory,
					init_handle,
					handler,
					finish_handle,
					init,
					abi_ptr: 0,
				});
				instance_state.assume_init()
			});

			Ok(instance)
		})
		.await
		.await?
	}
}

fn create_store() -> Store {
	#[allow(unused_mut)]
	let mut compiler_config = {
		#[cfg(feature = "LLVM")]
		{
			wasmer::LLVM::default()
		}
		#[cfg(not(feature = "LLVM"))]
		{
			wasmer::Cranelift::default()
		}
	};
	#[cfg(feature = "metering")]
	{
		let metering_factory = Arc::new(Metering::new(0, pricing));
		compiler_config.push_middleware(metering_factory);
	}
	let memory_limit = MemoryLimit::new(
		MEMORY_LIMIT
			.map(|x| (x / PAGE_BYTES as u64).max(MAX_PAGES as _) as u32)
			.unwrap_or(MAX_PAGES),
	);
	let mut engine = EngineBuilder::new(compiler_config).engine();
	engine.set_tunables(memory_limit);

	Store::new(engine)
}

unsafe fn as_static_mut<T>(v: &mut T) -> &'static mut T {
	&mut *(v as *mut T)
}

fn create_print(store: &mut Store, actor: &'static mut InstanceState) -> Function {
	let print_env = FunctionEnv::new(store, actor);
	wasmer::Function::new_typed_with_env(store, &print_env, print)
}

fn print(mut env: FunctionEnvMut<&'static mut InstanceState>, ptr: u32, len: u32) {
	let actor = &mut **env.data_mut();
	let memory = actor.memory.view(&actor.store);
	let mut data = Vec::with_capacity(len as _);
	unsafe {
		data.set_len(len as _);
	}
	let result = (|| {
		let data = memory.read_uninit(ptr as _, &mut data)?;
		let mut stdout = stdout().lock();
		if stdout.write_all(data).is_err() {
			drop(stdout);
			let mut stderr = stderr().lock();
			_ = stderr.write_all(data);
		}
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

struct InstanceState {
	metadata: Arc<Metadata>,
	store: Store,
	_module: Module,
	instance: WasmInstance,
	memory: Memory,
	init: TypedFunction<(), u32>,
	init_handle: TypedFunction<(u8, u32, u32), ()>,
	handler: TypedFunction<(), ()>,
	finish_handle: TypedFunction<(), ()>,
	abi_ptr: u32,
}

pub struct Instance(Box<InstanceState>);

impl Instance {
	pub fn invoke(&mut self, op: Operation, _gas: Option<&mut u64>) -> Result<Operation> {
		#[cfg(feature = "metering")]
		set_remaining_points(
			&mut self.0.store,
			&self.0.instance,
			_gas.as_deref().copied().unwrap_or(u64::MAX),
		);
		let result = self.inner_invoke(op);
		#[cfg(feature = "metering")]
		match get_remaining_points(&mut self.0.store, &self.0.instance) {
			wasmer_middlewares::metering::MeteringPoints::Remaining(g) => {
				if let Some(local_gas) = _gas {
					*local_gas = g;
				}
				result
			}
			wasmer_middlewares::metering::MeteringPoints::Exhausted => {
				if let Some(local_gas) = _gas {
					*local_gas = 0;
				}
				Err(GasFeeExhausted(self.0.metadata.id.clone()).into())
			}
		}
		#[cfg(not(feature = "metering"))]
		result
	}

	#[allow(clippy::uninit_vec)]
	fn inner_invoke(&mut self, op: Operation) -> Result<Operation> {
		let store = &mut self.0.store;

		if self.0.abi_ptr == 0 {
			self.0.abi_ptr = self.0.init.call(store)?;
		}
		let mut abi = OperationAbi::default();
		let abi_buffer = unsafe {
			&mut *(&mut abi as *mut OperationAbi as *mut [u8; size_of::<OperationAbi>()])
		}
		.as_mut_slice();

		// marshalling input message
		match &op {
			Operation::Call { ctx, req } => {
				self.0
					.init_handle
					.call(store, 0, ctx.len() as _, req.len() as _)?;
			}
			Operation::ReturnOk { resp } => {
				self.0.init_handle.call(store, 1, resp.len() as _, 0)?
			}
			Operation::ReturnErr { error } => {
				self.0
					.init_handle
					.call(store, 2, serialized_size(error)? as _, 0)?
			}
		}

		let view = self.0.memory.view(store);
		view.read(self.0.abi_ptr as _, abi_buffer)?;
		unsafe {
			abi.marshal(op, |vec, ptr, _| {
				view.write(*ptr as _, &vec)?;
				Ok(()) as Result<_>
			})?
		};

		// call handle
		self.0.handler.call(store)?;

		// marshalling output messsage
		let view = self.0.memory.view(store);
		view.read(self.0.abi_ptr as _, abi_buffer)?;
		let output = unsafe {
			abi.unmarshal(|ptr, len| {
				let mut vec = Vec::with_capacity(len as _);
				vec.set_len(len as _);
				view.read(ptr as _, &mut vec)?;
				Ok(vec) as Result<_>
			})?
		};

		self.0.finish_handle.call(store)?;

		Ok(output)
	}

	pub fn metadata(&self) -> &Arc<Metadata> {
		&self.0.metadata
	}
}
