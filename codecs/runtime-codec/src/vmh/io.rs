use serde::{Deserialize, Serialize};
use std::{collections::HashSet, fmt::Display, str::FromStr};
use strum::Display;

use crate::vmh::error::VmhGeneralErrors;

pub const REGISTRY_SOCKET_NAME: &str = "/tmp/registry.socket";
pub const COMMAND_SOCKET_NAME: &str = "/tmp/command.socket";
pub const OUTPUT_SOCKET_NAME: &str = "/tmp/output.socket";
pub const APP_REQ_SOCKET_NAME: &str = "/tmp/app-req.socket";

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RegistryItem {
	pub server_port: u32,
	pub direction: Direction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Direction {
	Inbond,
	Outbound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display)]
pub enum RegistryKey {
	/// use to output enclave logs to parent instance client
	Log,

	Layer1Inbound,
	Layer1Outbound,

	IpfsOutbound,

	AdapterInbound,

	HttpOutbound,
	HttpProxy,

	PersistOutbound,

	OrbitDbOutbound,

	Libp2pInbound,
	Libp2pOutbound,

	ThirdApiOutbound,

	NitroOutbound,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServerRequest {
	Normal(Vec<u8>),
	Quit,
}

#[cfg(not(feature = "nitro"))]
pub type HostType = String;
#[cfg(feature = "nitro")]
pub type HostType = u32;

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum UpgradeType {
	Provider,
	Client,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
	pub new_version: String,
	pub current_version: String,
	pub url: String,
}

impl VersionInfo {
	pub fn need_upgrade(&self) -> bool {
		!self.new_version.is_empty()
			&& self.new_version != self.current_version
			&& !self.url.is_empty()
	}
}

impl Display for UpgradeType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			UpgradeType::Client => write!(f, "client"),
			UpgradeType::Provider => write!(f, "provider"),
		}
	}
}

impl FromStr for UpgradeType {
	type Err = crate::vmh::error::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s {
			"client" => Ok(UpgradeType::Client),
			"provider" => Ok(UpgradeType::Provider),
			_ => Err(VmhGeneralErrors::UnknownUpgradeType(s.to_string()).into()),
		}
	}
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AppState {
	Initializing,
	Initialized,
	Pending,
}

impl Default for AppState {
	fn default() -> Self {
		AppState::Initializing
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegistryRequest {
	ListItems,
	Add(RegistryKey, Direction),
	Extend(Vec<(RegistryKey, Direction)>),
	Get(RegistryKey),
	GetState(UpgradeType),
	SetState(AppState, UpgradeType),
	SetHost(HostType),
	GetHost,
	RegisterUpgrade(UpgradeType, String),
	PutUpgradeValue(UpgradeType, String, Vec<u8>),
	TakeUpgradeValue(UpgradeType, String),
	ReadyForUpgrade(UpgradeType),
	GetSeq(UpgradeType),
	FetchAddSeq(UpgradeType),
	Dump,
	VersionInfo(UpgradeType),
	ResetVersion(String, UpgradeType),
	GetEnclaveAppPath,
	SetEnclaveAppPath(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClientKey {
	Layer1,
	Ipfs,
	Adapter,
	Http,
	Persist,
	Orbitdb,
	Libp2p,
	ThirdApi,
}

pub fn parse_clients(registry_keys: HashSet<RegistryKey>) -> HashSet<ClientKey> {
	let mut result = HashSet::new();

	if registry_keys.contains(&RegistryKey::Layer1Inbound)
		&& registry_keys.contains(&RegistryKey::Layer1Outbound)
	{
		result.insert(ClientKey::Layer1);
	}

	if registry_keys.contains(&RegistryKey::IpfsOutbound) {
		result.insert(ClientKey::Ipfs);
	}

	if registry_keys.contains(&RegistryKey::AdapterInbound) {
		result.insert(ClientKey::Adapter);
	}

	if registry_keys.contains(&RegistryKey::HttpOutbound) {
		result.insert(ClientKey::Http);
	}

	if registry_keys.contains(&RegistryKey::PersistOutbound) {
		result.insert(ClientKey::Persist);
	}

	if registry_keys.contains(&RegistryKey::PersistOutbound) {
		result.insert(ClientKey::Persist);
	}

	if registry_keys.contains(&RegistryKey::OrbitDbOutbound) {
		result.insert(ClientKey::Orbitdb);
	}

	if registry_keys.contains(&RegistryKey::ThirdApiOutbound) {
		result.insert(ClientKey::ThirdApi);
	}

	if registry_keys.contains(&RegistryKey::Libp2pOutbound)
		&& registry_keys.contains(&RegistryKey::Libp2pInbound)
	{
		result.insert(ClientKey::Libp2p);
	}

	result
}

impl std::fmt::Display for ClientKey {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			ClientKey::Layer1 => write!(f, "layer1"),
			ClientKey::Ipfs => write!(f, "ipfs"),
			ClientKey::Adapter => write!(f, "adapter"),
			ClientKey::Http => write!(f, "http"),
			ClientKey::Persist => write!(f, "persist"),
			ClientKey::Orbitdb => write!(f, "orbitdb"),
			ClientKey::Libp2p => write!(f, "libp2p"),
			ClientKey::ThirdApi => write!(f, "third-api"),
		}
	}
}
