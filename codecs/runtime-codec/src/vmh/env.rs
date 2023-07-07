use crate::solc::ContractAddresses;
use crate::tapp::seat::SeatId;
use crate::vmh::error::Errors;
use crate::vmh::io::RegistryKey;
use crate::vmh::{error::Result, utils::split_once};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

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
	pub contract_addresses: HashMap<String, ContractAddresses>,
	pub tappstore_id: String,
	pub chain_ids: HashMap<String, u64>,
	pub mining_startup_nodes: Vec<MiningStartupItem>,
	pub enclave_pcrs: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
pub struct MiningStartupItem {
	pub machine_id: String,
	pub seat_id: SeatId,
	pub conn_id: String,
	pub key: String,
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
	LoadActor(Vec<u8>),
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
	LoadActor(Vec<u8>),
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

// #[cfg(test)]
// mod tests {
// 	use super::GenesisConfig;

// 	#[test]
// 	fn deserialize_genesis_config_works() -> anyhow::Result<()> {
// 		let config_buf = include_bytes!("../../../resources/genesis.json");
// 		let config: GenesisConfig = serde_json::from_slice(config_buf)?;

// 		assert_eq!(
// 			config.tappstore_id,
// 			"0x610178dA211FEF7D417bC0e6FeD39F05609AD788"
// 		);
// 		assert_eq!(
// 			config.contract_addresses.lock,
// 			"0x0165878A594ca255338adfa4d48449f69242Eb8F"
// 		);

// 		assert_eq!(config.mining_startup_nodes.len(), 5);
// 		assert_eq!(
// 			config.mining_startup_nodes[0].conn_id,
// 			"12D3KooWKUd6bwsqNKzFgeruvk7pNSMUoBcrUKKU9Djqd1A3H9q8"
// 		);

// 		assert_eq!(config.enclave_pcrs["PCR0"], "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
// 		assert_eq!(config.enclave_pcrs["PCR1"], "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");
// 		assert_eq!(config.enclave_pcrs["PCR2"], "000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000");

// 		Ok(())
// 	}
// }
