#[cfg(feature = "host")]
use command_fds::FdMappingCollision;
use tea_codec::define_scope;
#[cfg(feature = "host")]
use tea_sdk::errorx::{Descriptor, Scope};
use tea_sdk::serde::error::Serde;
use thiserror::Error;

use crate::core::actor::ActorId;

define_scope! {
	ActorX2: Serde {
		BadWorkerOutput => BadWorkerOutput;
		WorkerCrashed => WorkerCrashed;
		AccessNotPermitted => AccessNotPermitted;
		ActorNotExist => ActorNotExist;
		NotSupported => NotSupported;
		ActorDeactivating => ActorDeactivating;
		GasFeeExhausted => GasFeeExhausted;
		OutOfActorHostContext => OutOfActorHostContext;
		ActorHostDropped => ActorHostDropped;
	}
}

#[cfg(feature = "host")]
impl Descriptor<FdMappingCollision> for ActorX2 {
	fn name(_: &FdMappingCollision) -> Option<std::borrow::Cow<str>> {
		Some(format!("{}.FdMappingCollision", ActorX2::NAME).into())
	}

	fn type_id(_: &FdMappingCollision) -> Option<std::any::TypeId> {
		Some(std::any::TypeId::of::<FdMappingCollision>())
	}
}

#[derive(Debug, Error)]
#[error("Gas fee is exhausted within wasm actor {0}")]
pub struct GasFeeExhausted(pub ActorId);

#[derive(Debug, Error)]
pub enum BadWorkerOutput {
	#[error("Unknown MasterCommand {0} from the worker of {1}")]
	UnknownMasterCommand(u8, ActorId),

	#[error("Non existing channel {0} from the worker of {1}")]
	ChannelNotExist(u64, ActorId),
}

#[derive(Debug, Error)]
#[error("Worker crashed")]
pub struct WorkerCrashed;

#[derive(Debug, Error)]
#[error("Access to actor {0} is not permitted")]
pub struct AccessNotPermitted(pub ActorId);

#[derive(Debug, Error)]
#[error("Attempting to invoke actor {0} that does not exist")]
pub struct ActorNotExist(pub ActorId);

#[derive(Debug, Error)]
#[error("{0} is not supported")]
pub struct NotSupported(pub &'static str);

#[derive(Debug, Error)]
#[error("Invoking an actor requires an actor host context set for the current task")]
pub struct OutOfActorHostContext;

#[derive(Debug, Error)]
#[error("The actor host is dropped for the future with with_actor_host is complete")]
pub struct ActorHostDropped;

#[derive(Debug, Error)]
#[error("Actor {0} is deactivating")]
pub struct ActorDeactivating(pub ActorId);
