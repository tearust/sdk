use tea_sdk::{actorx::error::ActorX, errorx::Global};
use thiserror::Error;

pub type Error = ActorXExamplesActor;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum ActorXExamplesActor {
	#[error(transparent)]
	ActorX(#[from] ActorX),
}

impl From<ActorXExamplesActor> for Global {
	fn from(value: ActorXExamplesActor) -> Self {
		Global::Unnamed(format!("{value:?}"))
	}
}
