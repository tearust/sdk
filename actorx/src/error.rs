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

	#[error("stdio error: {0}")]
	StdIoError(String),

	#[error("bincode error: {0}")]
	BincodeSerde(String),

	#[error(transparent)]
	Global(#[from] Global),

	#[error(transparent)]
	BadWorkerOutput(#[from] BadWorkerOutput),

	#[error(transparent)]
	MissingCallingStack(#[from] MissingCallingStack),

	#[error(transparent)]
	InvokeDeserializeError(#[from] InvokeDeserializeError),
}

impl From<ActorX> for Global {
	fn from(e: ActorX) -> Self {
		match e {
			ActorX::Global(g) => g,
			_ => Global::Unnamed(e.to_string()),
		}
	}
}

impl From<std::io::Error> for ActorX {
	fn from(e: std::io::Error) -> Self {
		ActorX::StdIoError(e.to_string())
	}
}

impl From<bincode::Error> for ActorX {
	fn from(e: bincode::Error) -> Self {
		ActorX::BincodeSerde(e.to_string())
	}
}

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum BadWorkerOutput {
	#[error("Unknown MasterCommand {0} from the worker of {1}")]
	UnknownMasterCommand(u8, ActorId),

	#[error("Non existing channel {0} from the worker of {1}")]
	ChannelNotExist(u64, ActorId),
}

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum MissingCallingStack {
	#[error("The operation must be within a current actor context")]
	Current,
	#[error("The operation must be called with an actor caller")]
	Caller,
}

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
#[error("Failed to deserialize the invoke response to actor '{0}': {1}")]
pub struct InvokeDeserializeError(pub ActorId, pub String);
