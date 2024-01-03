use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
#[error("Type id does not match, expected \"{0}\", actual \"{1}\"")]
pub struct TypeIdMismatch(pub String, pub String);

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
#[error("Type id \"{0}\" is not supported here")]
pub struct UnexpectedType(pub String);

#[derive(Debug, Clone, Error, PartialEq, Eq, Serialize, Deserialize)]
#[error("Invalid byte format when reading {0}")]
pub struct InvalidFormat(pub String);
