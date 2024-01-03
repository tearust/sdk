use serde::{Deserialize, Serialize};
use tea_codec::errorx::Global;
use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum Error {
	#[error("Global error: {0}")]
	Global(#[from] Global),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
