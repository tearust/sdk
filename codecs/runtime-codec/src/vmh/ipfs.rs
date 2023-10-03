use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IpfsRequestType {
	GetFile(String),
	PersistFile(Vec<u8>),
}
