use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::TimestampShort;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalVersionsReadable {
	pub system_versions: SystemVersions,
	pub pre_client_version_expire_at: Option<TimestampShort>,
	pub pre_enclave_version_expire_at: Option<TimestampShort>,
}
