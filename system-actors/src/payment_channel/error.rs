use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum Error {
	#[error(transparent)]
	NotSupportedSignContent(#[from] NotSupportedSignContent),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
#[error("not supported sign content")]
pub struct NotSupportedSignContent;
