use crate::solc::ContractAddresses;
use crate::tapp::seat::SeatId;
use crate::tapp::Hash;
use crate::vmh::error::Errors;
use crate::vmh::io::RegistryKey;
use crate::vmh::{error::Result, utils::split_once};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::str::FromStr;
use tea_sdk::serialize;

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct EnvSettings {
	pub genesis: GenesisConfig,
	pub machine_owner: String,
	pub tea_id: String,
	pub replica_test_mode: bool,
	pub apply_validator: bool,
	pub test_mode: bool,
	pub lastest_topup_height: u64,
	pub startup_proof: Option<String>,
	pub settings: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct GenesisConfig {
	pub network: String,
	pub contract_addresses: ContractAddresses,
	pub tappstore_id: String,
	pub chain_id: u64,
	pub mining_startup_nodes: Vec<MiningStartupItem>,
	pub enclave_pcrs: Vec<(String, String)>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct MiningStartupItem {
	pub machine_id: String,
	pub seat_id: SeatId,
	pub conn_id: String,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct HostSettings {
	pub tea_id: Vec<u8>,
	pub conn_id: String,
	pub env_settings: EnvSettings,
	pub manifest: Vec<u8>,
	pub state_magic_number: u64,
	pub loaded_clients: Vec<RegistryKey>,
	pub encryted_key: Option<Vec<u8>>,
	pub init_layer1_key: Option<Vec<u8>>,
	pub actor_download_path: String,
	pub ipfs_url_base: String,
	#[cfg(feature = "dev")]
	pub local_wasm_folder: Option<String>,
	#[cfg(feature = "dev")]
	pub genesis_settings: super::registry::GenesisSettings,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum HostCommand {
	Init(Box<HostSettings>),
	Stdin(String),
	/// first field is version name, second field is url, three fields downloaded execute buffer
	UpgradeEnclave(String, String, Vec<u8>, HashMap<String, Vec<u8>>),
	/// first field is version name, second field is url
	UpgradeClient(String, String),
	Shutdown,
	Export,
	Import(Vec<u8>),
	LoadActor(String, Vec<u8>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum AppRequest {
	Initialized(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppCommand {
	Shutdown,
	DeactiveAll,
	Export,
	Import(Vec<u8>),
	LoadActor(String, Vec<u8>),
}

impl std::fmt::Display for AppCommand {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			AppCommand::Shutdown => write!(f, "Shutdown"),
			AppCommand::DeactiveAll => write!(f, "DeactiveAll"),
			AppCommand::Export => write!(f, "Export"),
			AppCommand::Import(..) => write!(f, "Import"),
			AppCommand::LoadActor(..) => write!(f, "LoadActor"),
		}
	}
}

impl FromStr for AppCommand {
	type Err = crate::vmh::error::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"Shutdown" => Ok(AppCommand::Shutdown),
			"DeactiveAll" => Ok(AppCommand::DeactiveAll),
			"Export" => Ok(AppCommand::Export),
			_ => Err(Errors::UnknownAppCommand(s.to_string()).into()),
		}
	}
}

pub fn hex_decode(mut tea_id: &str) -> Result<Vec<u8>> {
	if tea_id.starts_with("0x") {
		tea_id = tea_id.trim_start_matches("0x");
	}
	Ok(hex::decode(tea_id)?)
}

impl EnvSettings {
	pub fn import_settings(&mut self, value: &str) -> Result<()> {
		if value.is_empty() {
			return Ok(());
		}

		for item in value.split_whitespace() {
			let (key, value) = split_once(item, ":")?;
			self.settings.insert(key.to_string(), value.to_string());
		}
		Ok(())
	}
}

impl GenesisConfig {
	pub fn hash(&self) -> Result<Hash> {
		let bytes = serialize(&self.contract_addresses)?;
		let hash_g_array = Sha256::digest(bytes.as_slice());
		let hash_key: Hash = hash_g_array.as_slice().try_into()?;
		Ok(hash_key)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn genesis_config_hash_works() -> Result<()> {
		let genesis_config = GenesisConfig {
			network: "test network".to_string(),
			tappstore_id: "tappstore id".to_string(),
			chain_id: 1234,
			contract_addresses: ContractAddresses {
				lock: "lock".to_string(),
				storage: "storage".to_string(),
				maintainer: "maintainer".to_string(),
				token_vesting: "token_vesting".to_string(),
				erc721: "erc721".to_string(),
			},
			mining_startup_nodes: vec![
				MiningStartupItem {
					machine_id: "machine_id_1".to_string(),
					seat_id: 1,
					conn_id: "conn_id_1".to_string(),
				},
				MiningStartupItem {
					machine_id: "machine_id_2".to_string(),
					seat_id: 2,
					conn_id: "conn_id_2".to_string(),
				},
			],
			enclave_pcrs: vec![
				("pcr_1".to_string(), "pcr_1_value".to_string()),
				("pcr_2".to_string(), "pcr_2_value".to_string()),
			],
		};

		let hash1 = genesis_config.hash()?;
		let hash2 = genesis_config.hash()?;
		assert_eq!(hash1, hash2);
		Ok(())
	}
}
