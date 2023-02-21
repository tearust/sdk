use std::fmt::{Debug, Write};

use tea_actorx_core::{error::ActorX, ActorId, RegId};
use tea_actorx_signer::error::Signer;
use tea_codec::define_scope;
use thiserror::Error;
use wasmer::RuntimeError;

define_scope! {
	Host: pub ActorX {
		wasmer::InstantiationError => @ActorX::HostInstantiation, @Display, @Debug;
		wasmer::CompileError => @ActorX::WasmCompile, @Display, @Debug;
		wasmer::ExportError => @ActorX::WasmExport, @Display, @Debug;
		wasmer::RuntimeError as v => @ActorX::WasmRuntime, v.message(), debug_runtime_error(v);
		wasmer::MemoryAccessError => @ActorX::WasmMemoryAccess, @Display, @Debug;
		HostErrors => @ActorX::Host, @Display, @Debug;
		GasFeeExhausted => @ActorX::GasFeeExhausted, @Display, @Debug;
		PriceUndefined => @ActorX::PriceUndefined, @Display, @Debug;
		AccessNotPermitted => @ActorX::AccessNotPermitted, @Display, @Debug;
		Signer => Signer, @Display, @Debug;
		RingInvocation => @ActorX::RingInvocation, @Display, @Debug;
		NativeActorCallingWasmActor => @ActorX::NativeActorCallingWasmActor, @Display, @Debug;
		NativeActorNotExists => @ActorX::NativeActorNotExists, @Display, @Debug;
	}
}

#[derive(Debug, Error)]
pub enum HostErrors {
	#[error("Actor deactivated")]
	ActorDeactivated,

	#[error("Invalid wasm output")]
	InvalidWasmOutput,
}

#[derive(Debug, Error)]
#[error("native actor {0} not exists")]
pub struct NativeActorNotExists(pub RegId);

#[derive(Debug, Error)]
pub enum GasFeeExhausted {
	#[error("Gas fee is exhausted performing native checks")]
	NativeCheck,

	#[error("Gas fee is exhausted within wasm actor {0}")]
	Wasm(ActorId),
}

#[derive(Debug, Error)]
#[error("The price of {0} is not defined or not available for wasm actors")]
pub struct PriceUndefined(pub &'static str);

#[derive(Debug, Error)]
#[error("Access to actor {0} is not permitted")]
pub struct AccessNotPermitted(pub RegId);

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

#[derive(Error)]
#[error("Actor \"{0}\" is trying to invoke actor \"{1}\", which makes it a deadlock of ring invocation.")]
pub struct RingInvocation(pub ActorId, pub ActorId, pub Vec<ActorId>);

impl Debug for RingInvocation {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("RingInvocation")
			.field("calling_stack", &self.2)
			.finish()
	}
}

#[derive(Error, Debug)]
#[error(
	"Native actor {0} is calling wasm actor {1} within its event loop, which may lead to deadlock. Use post instead or call with a spawned task."
)]
pub struct NativeActorCallingWasmActor(pub ActorId, pub ActorId);
