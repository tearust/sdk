use tea_codec::define_scope;
use thiserror::Error;

use crate::error::ActorX2Core;

define_scope! {
	Signer: ActorX2Core {
		openssl::error::ErrorStack => OpenSsl, @Display, @Debug;
		SignatureMismatch => Signature, @Display, @Debug;
		InvalidSignatureFormat => InvalidSignatureFormat, @Display, @Debug;
		leb128::read::Error => Leb128ReadError, @Display, @Debug;
	}
}

#[derive(Debug, Error)]
#[error("The signature of a wasm file does not match")]
pub struct SignatureMismatch;

#[derive(Debug, Error)]
#[error("Invalid signature format")]
pub struct InvalidSignatureFormat;
