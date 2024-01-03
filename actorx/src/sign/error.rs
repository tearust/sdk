use tea_sdk::errorx::Global;
use thiserror::Error;

pub type Error = Signer;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Error)]
pub enum Signer {
	#[error(transparent)]
	OpenSsl(#[from] openssl::error::ErrorStack),

	#[error(transparent)]
	Signature(#[from] SignatureMismatch),

	#[error(transparent)]
	InvalidSignatureFormat(#[from] InvalidSignatureFormat),

	#[error(transparent)]
	Leb128ReadError(#[from] leb128::read::Error),

	#[error(transparent)]
	Global(#[from] Global),

	#[error(transparent)]
	YamlSerde(#[from] serde_yaml::Error),

	#[error(transparent)]
	StdIo(#[from] std::io::Error),
}

#[derive(Debug, Error)]
#[error("The signature of a wasm file does not match")]
pub struct SignatureMismatch;

#[derive(Debug, Error)]
#[error("Invalid signature format")]
pub struct InvalidSignatureFormat;
