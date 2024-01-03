use serde::{Deserialize, Serialize};
use thiserror::Error;

pub type Error = RuntimeTappError;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeTappError {
	#[error(transparent)]
	StatementTypeParse(#[from] StatementTypeParse),

	#[error(transparent)]
	Errors(#[from] Errors),
}

#[derive(Error, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[error("Failed to parse '{0}' to statement type")]
pub struct StatementTypeParse(pub String);

#[derive(Error, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Errors {
	#[error("Failed to to parse mining status from \"{0}\"")]
	ParseMiningStatus(String),

	#[error("Failed to to parse market status from \"{0}\"")]
	ParseMarketStatus(String),

	#[error("Failed to to parse maintain status from \"{0}\"")]
	ParseMaintainStatus(String),

	#[error("failed to parse {0} to node status")]
	ParseNodeStatusFailed(String),

	#[error("failed to parse {0} to pcr value")]
	UnknowPcrValue(usize),

	#[error("failed to parse address from string")]
	ParseAddressError,
}
