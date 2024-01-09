use tea_actorx::error::ActorX;
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Error {
	#[error("get system time error")]
	GetSystemTime,

	#[error(transparent)]
	ActorX(#[from] ActorX),
}

impl From<Error> for tea_sdk::errorx::Global {
	fn from(e: Error) -> Self {
		tea_sdk::errorx::Global::Unnamed(format!("{e:?}"))
	}
}
