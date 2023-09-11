use std::{
	collections::hash_map::DefaultHasher,
	hash::{Hash, Hasher},
	io::{stderr, stdout, Write},
	mem::{size_of, MaybeUninit},
	path::Path,
	sync::Arc,
	time::Duration,
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
	timeout_retry,
	worker::{error::Result, wasm::memory::MemoryLimit},
};
use bincode::serialized_size;
use tokio::{fs, sync::RwLock, time::Instant};
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

mod memory;

const CACHE_DIR: &str = ".cache";

const PAGE_BYTES: u32 = 64 * 1024;
const MAX_PAGES: u32 = 65536;

pub struct Host {
	metadata: Arc<Metadata>,
	states: Arc<RwLock<StateArray>>,
	instance_count: u8,
	compiled_path: String,
}

pub(crate) struct StateArray {
	states: Vec<SharedState>,
}

pub(crate) type SharedState = Arc<RwLock<State>>;

pub(crate) struct State {
	instance: Instance,
	idle: bool,
}

impl State {
	pub(crate) fn reset_idle(&mut self) {
		self.idle = true;
	}

	pub(crate) fn instance(&mut self) -> &mut Instance {
		&mut self.instance
	}
}

impl Host {
	pub async fn new(source: Vec<u8>, instance_count: u8) -> Result<Self> {
		let metadata = Arc::new(verify(&source)?);
		let mut hasher = DefaultHasher::new();
		source.hash(&mut hasher);
		let hash = hasher.finish();
		let compiled_path = format!("{CACHE_DIR}/{hash:x}");

		let result = Self {
			metadata: metadata.clone(),
			states: Arc::new(RwLock::new(StateArray { states: vec![] })),
			instance_count,
			compiled_path,
		};

		result.create_instances(source).await?;
		Ok(result)
	}

	pub fn metadata(&self) -> Arc<Metadata> {
		self.metadata.clone()
	}

	pub(crate) async fn create_instances(&self, source: Vec<u8>) -> Result<()> {
		let now = Instant::now();
		let source: Arc<[u8]> = source.into();

		let (first_module, first_store, module_bytes): (Module, Store, Arc<[u8]>) =
			if Path::new(&self.compiled_path).exists() {
				let bytes = self.read_module_cache().await?;
				let s = create_store();
				let m = Module::deserialize_checked(&s, &bytes)?;
				(m, s, bytes.into())
			} else {
				let (m, s, b) = Self::first_proto_module(source.clone()).await?;
				let bytes: Arc<[u8]> = b.into();
				Self::write_module_cache(self.compiled_path.clone(), bytes.clone());
				(m, s, bytes)
			};

		let mut states = self.states.write().await;
		states.states.push(Arc::new(RwLock::new(
			Self::new_state(self.metadata(), first_module, first_store).await?,
		)));
		drop(states);

		if self.instance_count > 1 {
			for _ in 0..self.instance_count - 1 {
				let source = source.clone();
				let metadata = self.metadata.clone();
				let module_bytes = module_bytes.clone();
				let states = self.states.clone();

				tokio::spawn(async move {
					match Self::state_from_proto(metadata, module_bytes, source).await {
						Ok(state) => {
							let mut states = states.write().await;
							states.states.push(Arc::new(RwLock::new(state)));
						}
						Err(e) => {
							println!("ignored a create instance state result because error: {e:?}");
						}
					}
				});
			}
		}

		println!(
			r#"create instance takes: {:?}, worker source size: {}M bytes, cached module size: {}M bytes"#,
			now.elapsed(),
			source.len() / 1024 / 1024,
			module_bytes.len() / 1024 / 1024,
		);
		Ok(())
	}

	async fn first_proto_module(source: Arc<[u8]>) -> Result<(Module, Store, Vec<u8>)> {
		let store = create_store();
		let module = Module::new(&store, source)?;
		let bytes = module.serialize()?.into();
		Ok((module, store, bytes))
	}

	pub(crate) async fn instance_from_cache(&self) -> Result<State> {
		let module_bytes = self.read_module_cache().await?;
		let store = create_store();
		let module = Module::deserialize_checked(&store, &module_bytes)?;
		Self::new_state(self.metadata.clone(), module, store).await
	}

	async fn state_from_proto(
		metadata: Arc<Metadata>,
		proto_module: Arc<[u8]>,
		source: Arc<[u8]>,
	) -> Result<State> {
		let store = create_store();
		let module = match Module::deserialize_checked(&store, proto_module.as_ref()) {
			Ok(module) => module,
			Err(e) => {
				println!(
					"re-compile module because deserialize checked error: {:?}",
					e
				);
				if source.is_empty() {
					return Err(e.into());
				}
				Module::new(&store, source)?
			}
		};
		Self::new_state(metadata, module, store).await
	}

	async fn new_state(metadata: Arc<Metadata>, module: Module, store: Store) -> Result<State> {
		let instance = Self::create_instance(metadata, store, module).await?;
		Ok(State {
			instance,
			idle: true,
		})
	}

	pub(crate) async fn create_instance(
		metadata: Arc<Metadata>,
		mut store: Store,
		module: Module,
	) -> Result<Instance> {
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
	}

	pub(crate) async fn get_instance(&self) -> SharedState {
		loop {
			let states = self.states.read().await;
			if let Some(i) = states.idle_instance().await {
				let s = i.clone();
				s.write().await.idle = false;
				return s;
			}
			drop(states);

			tokio::time::sleep(Duration::from_millis(1)).await;
		}
	}

	pub fn write_module_cache(compiled_path: String, module_bytes: Arc<[u8]>) {
		tokio::spawn(async move {
			if let Err(e) = async {
				_ = fs::create_dir(CACHE_DIR).await;
				let mut file = fs::OpenOptions::new()
					.write(true)
					.create(true)
					.append(false)
					.open(compiled_path)
					.await?;
				write_var_bytes(&mut file, &module_bytes).await?;
				Ok(()) as Result<_>
			}
			.await
			{
				println!("write module cache error: {}", e);
			} else {
				println!("write module cache successfully");
			}
		});
	}

	async fn read_module_cache(&self) -> Result<Vec<u8>> {
		let mut file = fs::OpenOptions::new()
			.read(true)
			.open(&self.compiled_path)
			.await?;
		let wasm = read_var_bytes(&mut file).await?;
		Ok(wasm)
	}
}

fn create_store() -> Store {
	#[allow(unused_mut)]
	let mut compiler_config = {
		#[cfg(feature = "LLVM")]
		{
			wasmer::LLVM::new()
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

impl StateArray {
	pub(crate) async fn idle_instance(&self) -> Option<&SharedState> {
		for item in self.states.iter() {
			if item.read().await.idle {
				return Some(item);
			}
		}
		None
	}
}

pub(crate) struct InstanceState {
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
				warn!("Operation return error: {error}");
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

	#[cfg(feature = "verbose_log")]
	pub fn metadata(&self) -> &Arc<Metadata> {
		&self.0.metadata
	}
}
