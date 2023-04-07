use std::fmt::Write;

use tea_codec::define_scope;
use wasmer::RuntimeError;

define_scope! {
	ActorX2Worker {
		wasmer::InstantiationError => HostInstantiation, @Display, @Debug;
		wasmer::CompileError => WasmCompile, @Display, @Debug;
		wasmer::ExportError => WasmExport, @Display, @Debug;
		wasmer::RuntimeError as v => WasmRuntime, v.message(), debug_runtime_error(v);
		wasmer::MemoryAccessError => WasmMemoryAccess, @Display, @Debug;
		wasmer::SerializeError => WasmMemoryAccess, @Display, @Debug;
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
