use serde::{Deserialize, Serialize};
use tea_codec::errorx::Global;
use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum Error {
	#[error("Global error: {0}")]
	Global(#[from] Global),

	#[error(transparent)]
	NotSupportedSignContent(#[from] NotSupportedSignContent),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
#[error("not supported sign content")]
pub struct NotSupportedSignContent;
