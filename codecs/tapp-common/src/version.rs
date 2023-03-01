use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
	pub version: String,
	pub url: String,
	pub modules: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemVersions {
	pub client: VersionInfo,
	pub enclave: VersionInfo,
}
