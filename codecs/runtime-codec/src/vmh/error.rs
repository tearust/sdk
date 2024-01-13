use serde::{Deserialize, Serialize};
use tea_sdk::errorx::Global;
use thiserror::Error;

pub type VmhResult<T, E = VmhError> = Result<T, E>;
pub type Error = VmhError;
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub type VmhError = Global;

impl From<TableAccess> for Global {
	fn from(e: TableAccess) -> Self {
		Global::Unnamed(format!("{e:?}"))
	}
}

impl From<PersistCheck> for Global {
	fn from(e: PersistCheck) -> Self {
		Global::Unnamed(format!("{e:?}"))
	}
}

impl From<VmhGeneralErrors> for Global {
	fn from(e: VmhGeneralErrors) -> Self {
		Global::Unnamed(format!("{e:?}"))
	}
}

impl From<InvalidASystemInvocation> for Global {
	fn from(e: InvalidASystemInvocation) -> Self {
		Global::Unnamed(format!("{e:?}"))
	}
}

impl From<InvalidBSystemInvocation> for Global {
	fn from(e: InvalidBSystemInvocation) -> Self {
		Global::Unnamed(format!("{e:?}"))
	}
}

// impl VmhError {
// 	pub fn invalid_a_system_invocation(name: String) -> Self {
// 		Self::InvalidASystemInvocation(InvalidASystemInvocation(name))
// 	}

// 	pub fn invalid_b_system_invocation(name: String) -> Self {
// 		Self::InvalidBSystemInvocation(InvalidBSystemInvocation(name))
// 	}
// }

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
pub enum VmhGeneralErrors {
	#[error("Unknown built-in env {0}")]
	UnknownBuiltInEnv(String),

	#[error("Unknown app command {0}")]
	UnknownAppCommand(String),

	#[error("Unknown upgrade type {0}")]
	UnknownUpgradeType(String),

	#[error("Txn hash file not exists {0}")]
	TxnHashFileNotExists(i64),
}
