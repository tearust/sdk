use tea_codec_macros::define_scope;
use thiserror::Error;

define_scope! {
	Serde {
		TypeIdMismatch => TypeIdMismatch, @Display, @Debug;
		InvalidFormat => InvalidFormat, @Display, @Debug;
	}
}

#[derive(Debug, Error)]
#[error("Type id does not match, expected \"{0}\", actual \"{1}\"")]
pub struct TypeIdMismatch(pub &'static str, pub String);

#[derive(Debug, Error)]
#[error("Type id \"{0}\" is not supported here")]
pub struct UnexpectedType(pub String);

#[derive(Debug, Error)]
#[error("Invalid byte format when reading {0}")]
pub struct InvalidFormat(pub &'static str);
