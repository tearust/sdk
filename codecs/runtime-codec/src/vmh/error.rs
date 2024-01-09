use serde::{Deserialize, Serialize};
use tea_sdk::errorx::Global;
use thiserror::Error;

pub type VmhResult<T, E = VmhError> = Result<T, E>;
pub type Error = VmhError;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VmhError {
	#[error("Global error: {0}")]
	Global(#[from] Global),

	#[error("Table access error: {0}")]
	TableAccess(#[from] TableAccess),

	#[error("Persist check error: {0}")]
	PersistCheck(#[from] PersistCheck),

	#[error("Vmh general error: {0}")]
	VmhGeneral(#[from] Errors),

	#[error(transparent)]
	InvalidASystemInvocation(#[from] InvalidASystemInvocation),

	#[error(transparent)]
	InvalidBSystemInvocation(#[from] InvalidBSystemInvocation),

	#[error("Vmh error: {0}")]
	Unnamed(String),
}

impl VmhError {
	pub fn invalid_a_system_invocation(name: String) -> Self {
		Self::InvalidASystemInvocation(InvalidASystemInvocation(name))
	}

	pub fn invalid_b_system_invocation(name: String) -> Self {
		Self::InvalidBSystemInvocation(InvalidBSystemInvocation(name))
	}
}

#[derive(Error, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[error("{0} is not allowed to call A node system invocation")]
pub struct InvalidASystemInvocation(pub String);

#[derive(Error, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[error("{0} is not allowed to call B node system invocation")]
pub struct InvalidBSystemInvocation(pub String);

#[derive(Error, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum TableAccess {
	#[error("Failed to get row at {0} in table {1}")]
	GetRow(usize, String),

	#[error("Failed to convert table {0} to array")]
	ConvertToArray(String),

	#[error("Failed to get table {0}")]
	GetTable(String),
}

#[derive(Error, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PersistCheck {
	#[error("Prefix length to long, expect is {0} actual is {1}")]
	PrefixTooLong(usize, usize),
	#[error("Key {0:?} is too short to remove prefix")]
	KeyTooShort(Vec<u8>),
	#[error("Prefix length mismatched, expect is {0} actual is {1}")]
	PrefixLengthMismatch(usize, usize),
}

#[derive(Error, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Errors {
	#[error("Unknown built-in env {0}")]
	UnknownBuiltInEnv(String),

	#[error("Unknown app command {0}")]
	UnknownAppCommand(String),

	#[error("Unknown upgrade type {0}")]
	UnknownUpgradeType(String),

	#[error("Txn hash file not exists {0}")]
	TxnHashFileNotExists(i64),
}
