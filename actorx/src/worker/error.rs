use std::fmt::{Formatter, Write};
use tea_sdk::errorx::Global;
use wasmer::RuntimeError;

use crate::{core::error::GasFeeExhausted, error::ActorX, sign::error::Signer};

pub type Error = ActorXWorker;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(thiserror::Error)]
pub enum ActorXWorker {
	#[error("Wasm worker error: {0}")]
	Unnamed(String),

	#[error("Stdio error: {0}")]
	StdIo(#[from] std::io::Error),

	#[error("UTF-8 error: {0}")]
	FromUtf8Error(#[from] std::string::FromUtf8Error),

	#[error("Bincode error: {0}")]
	BincodeSerde(#[from] bincode::Error),

	#[error("Global error: {0}")]
	Global(#[from] Global),

	#[error("Actor error: {0}")]
	ActorX(#[from] ActorX),

	#[error("Signer error: {0}")]
	Signer(#[from] Signer),

	#[error("GasFeeExhausted: {0}")]
	GasFeeExhausted(#[from] GasFeeExhausted),

	#[error(transparent)]
	DeserializeError(#[from] wasmer::DeserializeError),
	#[error(transparent)]
	HostInstantiation(#[from] wasmer::InstantiationError),
	#[error(transparent)]
	WasmCompile(#[from] wasmer::CompileError),
	#[error(transparent)]
	WasmExport(#[from] wasmer::ExportError),
	#[error(transparent)]
	WasmRuntime(#[from] wasmer::RuntimeError),
	#[error(transparent)]
	WasmMemoryAccess(#[from] wasmer::MemoryAccessError),
	#[error(transparent)]
	WasmSerialize(#[from] wasmer::SerializeError),
}

impl std::fmt::Debug for ActorXWorker {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			ActorXWorker::Unnamed(e) => write!(f, "{}", e),
			ActorXWorker::StdIo(e) => write!(f, "{}", e),
			ActorXWorker::FromUtf8Error(e) => write!(f, "{}", e),
			ActorXWorker::BincodeSerde(e) => write!(f, "{}", e),
			ActorXWorker::Signer(e) => write!(f, "{}", e),
			ActorXWorker::Global(e) => write!(f, "{}", e),
			ActorXWorker::GasFeeExhausted(e) => write!(f, "{}", e),
			ActorXWorker::ActorX(e) => write!(f, "{}", e),
			ActorXWorker::DeserializeError(e) => write!(f, "{}", e),
			ActorXWorker::HostInstantiation(e) => write!(f, "{}", e),
			ActorXWorker::WasmCompile(e) => write!(f, "{}", e),
			ActorXWorker::WasmExport(e) => write!(f, "{}", e),
			ActorXWorker::WasmRuntime(e) => write!(f, "{}", debug_runtime_error(e)),
			ActorXWorker::WasmMemoryAccess(e) => write!(f, "{}", e),
			ActorXWorker::WasmSerialize(e) => write!(f, "{}", e),
		}
	}
}

fn debug_runtime_error(e: &RuntimeError) -> String {
	let mut result = String::new();
	result.push_str("Backtrace:");
	for (i, t) in e.trace().iter().enumerate() {
		if let Some(name) = t.function_name() {
			result.write_fmt(format_args!("\n{i}: {name}")).unwrap();
		}
		result.push('\n');
	}
	result
}
