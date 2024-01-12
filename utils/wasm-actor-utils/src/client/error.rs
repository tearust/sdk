use tea_runtime_codec::{tapp::error::RuntimeTappError, vmh::error::VmhError};
use tea_sdk::errorx::Global;
use thiserror::Error;

pub type Error = Errors;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Errors {
	#[error("actor utils error: {0}")]
	Unnamed(String),

	#[error("Unknown request")]
	UnknownRequest,

	#[error("Unknown action {0}")]
	UnknownAction(String),

	#[error("failed to parse address from string")]
	ParseAddressError,

	#[error("u128 length should be {0}")]
	U128Length(usize),

	#[error("You need at least 0.001 TEA or Credit to submit txn.")]
	NotEnoughBalanceForTxn,

	#[error("serde json error: {0}")]
	SerdeJsonError(String),

	#[error("Global error: {0}")]
	Global(#[from] Global),

	#[error("Runtime tapp error: {0}")]
	RuntimeTappError(String),

	#[error("Vmh error: {0}")]
	VmhError(String),

	#[error("Parse address error: {0}")]
	ParseAddress(String),

	#[error("Parse balance error: {0}")]
	ParseBalance(String),

	#[error("Http error: {0}")]
	HttpError(String),

	#[error("prost error: {0}")]
	ProstError(String),

	#[error("Base64 error: {0}")]
	Base64Error(String),

	#[error("FromUtf8 error: {0}")]
	FromUtf8Error(String),

	#[error("ParseInt error: {0}")]
	ParseIntError(String),

	#[error("Unclave utils error: {0}")]
	EnclaveUtilsError(String),
}

impl From<RuntimeTappError> for Error {
	fn from(e: RuntimeTappError) -> Self {
		Error::RuntimeTappError(format!("{e:?}"))
	}
}

impl From<VmhError> for Error {
	fn from(e: VmhError) -> Self {
		Error::VmhError(format!("{e:?}"))
	}
}

impl From<serde_json::Error> for Error {
	fn from(e: serde_json::Error) -> Self {
		Error::SerdeJsonError(e.to_string())
	}
}

impl From<fixed_hash::rustc_hex::FromHexError> for Error {
	fn from(e: fixed_hash::rustc_hex::FromHexError) -> Self {
		Error::ParseAddress(e.to_string())
	}
}

impl From<std::num::ParseIntError> for Error {
	fn from(e: std::num::ParseIntError) -> Self {
		Error::ParseIntError(e.to_string())
	}
}

impl From<std::string::FromUtf8Error> for Error {
	fn from(e: std::string::FromUtf8Error) -> Self {
		Error::FromUtf8Error(e.to_string())
	}
}

impl From<base64::DecodeError> for Error {
	fn from(e: base64::DecodeError) -> Self {
		Error::Base64Error(e.to_string())
	}
}

impl From<prost::DecodeError> for Error {
	fn from(e: prost::DecodeError) -> Self {
		Error::ProstError(e.to_string())
	}
}

impl From<crate::enclave::error::Error> for Error {
	fn from(e: crate::enclave::error::Error) -> Self {
		match e {
			crate::enclave::error::Error::Global(e) => Error::Global(e),
			_ => Error::EnclaveUtilsError(format!("{e:?}")),
		}
	}
}
