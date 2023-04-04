use std::{
	collections::hash_map::DefaultHasher,
	hash::{Hash, Hasher},
	mem::{size_of, MaybeUninit},
	sync::Arc,
};

use crate::{error::Result, wasm::memory::MemoryLimit};
use bincode::serialized_size;
use tea_actorx2_core::{
	error::GasFeeExhausted,
	metadata::Metadata,
	sign::verify,
	wasm::{pricing, MEMORY_LIMIT},
	worker_codec::{read_var_bytes, write_var_bytes, Operation, OperationAbi},
};
use tokio::fs;
use wasmer::{
	imports, CompilerConfig, EngineBuilder, Extern, Function, FunctionEnv, FunctionEnvMut, Imports,
	Instance as WasmInstance, Memory, Module, Store, TypedFunction,
};
use wasmer_middlewares::{
	metering::{get_remaining_points, set_remaining_points},
	Metering,
};

mod memory;

const CACHE_DIR: &str = ".cache";

const PAGE_BYTES: u32 = 64 * 1024;
const MAX_PAGES: u32 = 65536;

pub struct Host {
	metadata: Arc<Metadata>,
	wasm: Vec<u8>,
}

impl Host {
	pub async fn new(bin: &[u8]) -> Result<Self> {
		let mut hasher = DefaultHasher::new();
		bin.hash(&mut hasher);
		let hash = hasher.finish();
		let compiled_path = format!("{CACHE_DIR}/{hash:x}");

		Ok(match Self::read_cache(&compiled_path).await {
			Ok(host) => host,
			Err(_) => Self::read_new(&bin, &compiled_path).await?,
		})
	}

	pub fn metadata(&self) -> &Arc<Metadata> {
		&self.metadata
	}

	async fn read_cache(compiled_path: &str) -> Result<Self> {
		let mut file = fs::OpenOptions::new()
			.read(true)
			.open(compiled_path)
			.await?;
		let metadata = read_var_bytes(&mut file).await?;
		let metadata = bincode::deserialize(&metadata)?;
		let wasm = read_var_bytes(&mut file).await?;

		Ok(Self { metadata, wasm })
	}

	async fn read_new(wasm: &[u8], compiled_path: &str) -> Result<Self> {
		let metadata = Arc::new(verify(wasm)?);
		let store = create_store();
		let module = Module::new(&store, wasm)?;
		_ = fs::create_dir(CACHE_DIR).await;
		let mut file = fs::OpenOptions::new()
			.write(true)
			.create(true)
			.open(compiled_path)
			.await?;
		let metadata_bytes = bincode::serialize(&metadata)?;
		write_var_bytes(&mut file, &metadata_bytes).await?;
		let module_bytes = module.serialize()?;
		write_var_bytes(&mut file, &module_bytes).await?;
		Ok(Self {
			metadata,
			wasm: module_bytes.into(),
		})
	}

	pub fn create_instance(&self) -> Result<Instance> {
		let mut store = create_store();
		let module = unsafe { Module::deserialize(&store, self.wasm.as_slice())? };
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
				metadata: self.metadata.clone(),
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
}

fn create_store() -> Store {
	let metering_factory = Arc::new(Metering::new(0, pricing));
	#[cfg(feature = "LLVM")]
	let compiler_config = wasmer::LLVM::default();
	#[cfg(not(feature = "LLVM"))]
	let mut compiler_config = wasmer::Cranelift::default();
	compiler_config.push_middleware(metering_factory);
	let memory_limit = MemoryLimit::new(
		MEMORY_LIMIT
			.map(|x| (x / PAGE_BYTES as u64).max(MAX_PAGES as _) as u32)
			.unwrap_or(MAX_PAGES),
	);

	Store::new_with_tunables(EngineBuilder::new(compiler_config), memory_limit)
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
		let data = std::str::from_utf8(data)?;
		print!("{data}");
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
	pub async fn invoke(&mut self, op: Operation, gas: Option<&mut u64>) -> Result<Operation> {
		set_remaining_points(
			&mut self.0.store,
			&self.0.instance,
			gas.as_deref().copied().unwrap_or(u64::MAX),
		);
		let result = self.inner_invoke(op);
		match get_remaining_points(&mut self.0.store, &self.0.instance) {
			wasmer_middlewares::metering::MeteringPoints::Remaining(g) => {
				if let Some(local_gas) = gas {
					*local_gas = g;
				}
				result
			}
			wasmer_middlewares::metering::MeteringPoints::Exhausted => {
				if let Some(local_gas) = gas {
					*local_gas = 0;
				}
				Err(GasFeeExhausted(self.0.metadata.id.clone()).into())
			}
		}
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
}
