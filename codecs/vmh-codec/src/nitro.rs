use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct NitroCredentials {
	pub access_key_id: String,
	pub secret_access_key: String,
	pub session_token: Option<String>,
	pub expiry: Option<SystemTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NitroRequest {
	GetCredential,
	GetAwsSettings,
	SaveEncryptedKey(String, String),
	GetEncryptedKey(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsSettings {
	pub region: String,
	pub proxy_port: u16,
	pub key_id: String,
}

impl NitroCredentials {
	pub fn valid(&self) -> bool {
		match self.expiry {
			Some(expiry) => SystemTime::now() < expiry + Duration::from_secs(5 * 60),
			None => false,
		}
	}
}
