use crate::tapp::{
	cml::CmlId,
	error::{Error, Errors, Result},
	Account, BlockNumber,
};
use primitive_types::H160;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;

#[doc(hidden)]
pub type IssuerId = H160;

/// Miner machine status enum.
/// Active is working, Offline means not work and out of network.
/// ScheduleDown means not work now, but still in network, use to upgrade miner or some schedule stuff.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum MiningStatus {
	Active,
	Offline,
	ScheduleDown,
}

/// Mining base intrinsic info.
/// issuer means the miner's vendor.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MiningIntrinsic {
	pub tea_id: Vec<u8>,
	pub issuer: IssuerId,
	pub owner: Account,
}

#[doc(hidden)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MiningVariable {
	pub status: MiningStatus,
	pub ip: String,
	pub orbitdb_id: Option<String>,
	pub suspend_height: Option<BlockNumber>,
	pub scheduled_down_height: Option<BlockNumber>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MiningInfo {
	pub intrinsic: MiningIntrinsic,
	pub variable: MiningVariable,
}

#[doc(hidden)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MachineInfoItem {
	pub tea_id: String,
	pub tea_id_hex: String,
	pub issuer: String,
	pub owner: String,
	pub mining_status: Option<String>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TappStartupItem {
	pub tea_id: Vec<u8>,
	pub cml_id: CmlId,
	pub ip: String,
}

impl FromStr for MiningStatus {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"active" => Ok(MiningStatus::Active),
			"offline" => Ok(MiningStatus::Offline),
			"scheduledown" => Ok(MiningStatus::ScheduleDown),
			_ => Err(Errors::ParseMiningStatus(s.to_string()).into()),
		}
	}
}

impl Display for MiningStatus {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			MiningStatus::Active => write!(f, "Active"),
			MiningStatus::Offline => write!(f, "Offline"),
			MiningStatus::ScheduleDown => write!(f, "ScheduleDown"),
		}
	}
}

impl Default for MiningStatus {
	fn default() -> Self {
		MiningStatus::Offline
	}
}

/// Base structure for active miner.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveMinerInfo {
	pub token_id: String,
	pub cml_id: CmlId,
	pub tea_id: String,
	pub mining_status: String,
	pub ip: String,
	pub owner: String,
	pub plantd_at: u64,
	pub node_status: String,
	pub cid: String,
	pub sell_price: String,
	pub total_supply: String,
}
