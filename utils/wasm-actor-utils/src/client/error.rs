use tea_actorx::error::ActorX;
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

	#[error("Unclave utils error: {0}")]
	EnclaveUtilsError(#[from] crate::enclave::error::Error),

	#[error("serde json error: {0}")]
	SerdeJsonError(#[from] serde_json::Error),

	#[error("Global error: {0}")]
	Global(#[from] Global),

	#[error("Actor error: {0}")]
	ActorX(#[from] ActorX),

	#[error("Runtime tapp error: {0}")]
	RuntimeTappError(#[from] RuntimeTappError),

	#[error("Vmh error: {0}")]
	VmhError(#[from] VmhError),

	#[error("Parse address error: {0}")]
	ParseAddress(#[from] fixed_hash::rustc_hex::FromHexError),

	#[error("Parse balance error: {0}")]
	ParseBalance(String),

	#[error("Http error: {0}")]
	HttpError(String),

	#[error(transparent)]
	ProstError(#[from] prost::DecodeError),

	#[error(transparent)]
	Base64Error(#[from] base64::DecodeError),

	#[error(transparent)]
	FromUtf8Error(#[from] std::string::FromUtf8Error),

	#[error(transparent)]
	ParseIntError(#[from] std::num::ParseIntError),
}
