use tea_actorx_core::error::ActorX;
use tea_codec::define_scope;
use thiserror::Error;

define_scope! {
	Signer: ActorX {
		openssl::error::ErrorStack => OpenSsl, @Display, @Debug;
		SignatureMismatch => Signature, @Display, @Debug;
		rustc_hex::FromHexError => Parse, @Display, @Debug;
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
