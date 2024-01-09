use tea_actorx::error::ActorX;
use tea_sdk::errorx::Global;
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
	#[error("get system error")]
	GetSystem,

	#[error(transparent)]
	ActorX(#[from] ActorX),
}

impl From<Error> for Global {
	fn from(e: Error) -> Self {
		Global::Unnamed(format!("{e:?}"))
	}
}
