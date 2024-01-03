use serde::{Deserialize, Serialize};
use tea_sdk::errorx::Global;
use thiserror::Error;

use crate::core::actor::ActorId;

pub type Error = ActorX;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorX {
	#[error("Wasm worker error: {0}")]
	WasmWorkerError(String),

	#[error("procfs error: {0}")]
	ProcError(String),

	#[error(transparent)]
	Global(#[from] Global),

	#[error(transparent)]
	BadWorkerOutput(#[from] BadWorkerOutput),

	#[error(transparent)]
	WorkerCrashed(#[from] WorkerCrashed),

	#[error(transparent)]
	AccessNotPermitted(#[from] AccessNotPermitted),

	#[error(transparent)]
	ActorNotExist(#[from] ActorNotExist),

	#[error(transparent)]
	NotSupported(#[from] NotSupported),

	#[error(transparent)]
	ActorDeactivating(#[from] ActorDeactivating),

	#[error(transparent)]
	GasFeeExhausted(#[from] GasFeeExhausted),

	#[error(transparent)]
	MissingCallingStack(#[from] MissingCallingStack),

	#[error(transparent)]
	ActorHostDropped(#[from] ActorHostDropped),

	#[error(transparent)]
	InvocationTimeout(#[from] InvocationTimeout),

	#[error(transparent)]
	ChannelReceivingTimeout(#[from] ChannelReceivingTimeout),

	#[error(transparent)]
	InvokeDeserializeError(#[from] InvokeDeserializeError),
}

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
#[error("Gas fee is exhausted")]
pub struct GasFeeExhausted;

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum BadWorkerOutput {
	#[error("Unknown MasterCommand {0} from the worker of {1}")]
	UnknownMasterCommand(u8, ActorId),

	#[error("Non existing channel {0} from the worker of {1}")]
	ChannelNotExist(u64, ActorId),
}

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
#[error("Worker crashed: {0}")]
pub struct WorkerCrashed(pub String);

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
#[error("Access to actor {0} is not permitted")]
pub struct AccessNotPermitted(pub ActorId);

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
#[error("Attempting to invoke actor {0} that does not exist")]
pub struct ActorNotExist(pub ActorId);

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
#[error("{0} is not supported")]
pub struct NotSupported(pub String);

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissingCallingStack {
	#[error("The operation must be within a current actor context")]
	Current,
	#[error("The operation must be called with an actor caller")]
	Caller,
}

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
#[error("The actor host is dropped for the future with with_actor_host is complete")]
pub struct ActorHostDropped;

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
#[error("Actor {0} is deactivating")]
pub struct ActorDeactivating(pub ActorId);

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
#[cfg_attr(
	any(feature = "host", feature = "wasm"),
	error("The invocation is timed out, calling stack: {0:?}")
)]
#[cfg_attr(
	not(any(feature = "host", feature = "wasm")),
	error("The invocation is timed out.")
)]
pub struct InvocationTimeout(#[cfg(any(feature = "host", feature = "wasm"))] pub Vec<u8>);

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
#[error("Receiving channel of actor {0} has timeout")]
pub struct ChannelReceivingTimeout(pub ActorId);

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
#[error("Failed to deserialize the invoke response to actor '{0}': {1}")]
pub struct InvokeDeserializeError(pub ActorId, pub String);
