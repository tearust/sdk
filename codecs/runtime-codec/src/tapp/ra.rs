use crate::tapp::error::Errors;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str::FromStr};
use strum::{Display, EnumString};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Display, EnumString)]
pub enum EnclaveType {
	AwsNitro,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Display, EnumString)]
pub enum PcrType {
	PCR0,
	PCR1,
	PCR2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionPcrs {
	pub version: String,
	pub enclave_type: EnclaveType,
	pub pcrs: HashMap<PcrType, String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TeaNodeProfile {
	pub tea_id: Vec<u8>,
	pub ephemeral_public_key: Option<Vec<u8>>,
	pub conn_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
	Pending,
	Active,
}

impl TryFrom<usize> for PcrType {
	type Error = crate::tapp::error::Error;

	fn try_from(value: usize) -> Result<Self, Self::Error> {
		match value {
			0 => Ok(PcrType::PCR0),
			1 => Ok(PcrType::PCR1),
			2 => Ok(PcrType::PCR2),
			_ => Err(Errors::UnknowPcrValue(value).into()),
		}
	}
}

impl PartialOrd for PcrType {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		self.slot_index().partial_cmp(&other.slot_index())
	}
}

impl Ord for PcrType {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.slot_index().cmp(&other.slot_index())
	}
}

impl PcrType {
	pub fn slot_index(&self) -> usize {
		match self {
			PcrType::PCR0 => 0,
			PcrType::PCR1 => 1,
			PcrType::PCR2 => 2,
		}
	}

	pub fn need_varify(index: usize) -> bool {
		index <= 2
	}
}

impl TeaNodeProfile {
	pub fn is_some(&self) -> bool {
		self.ephemeral_public_key.is_some() && self.conn_id.is_some()
	}
}

impl std::fmt::Display for NodeStatus {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			NodeStatus::Active => write!(f, "active"),
			NodeStatus::Pending => write!(f, "pending"),
		}
	}
}

impl FromStr for NodeStatus {
	type Err = crate::tapp::error::Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"active" => Ok(NodeStatus::Active),
			"pending" => Ok(NodeStatus::Pending),
			_ => Err(Errors::ParseNodeStatusFailed(s.to_string()).into()),
		}
	}
}
