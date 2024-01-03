use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, RuntimeCodec>;
pub type Error = RuntimeCodec;

#[derive(Error, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeCodec {
	#[error(transparent)]
	UpgradeError(#[from] UpgradeError),

	#[error(transparent)]
	InvalidValidator(#[from] InvalidValidator),

	#[error("Utf8 error: {0}")]
	Utf8Error(String),

	#[error("Serde json error: {0}")]
	SerdeJsonError(String),

	#[error("http error: {0}")]
	HttpError(String),
}

#[derive(Error, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum InvalidValidator {
	#[error("Invalid validator: {0}")]
	Valued(String),

	#[error("My validator list is empty")]
	Empty,
}

#[derive(Error, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UpgradeError {
	#[error("version {0} is not compatible with {1}")]
	IncompatibleVersion(String, String),
}
