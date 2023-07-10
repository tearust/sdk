pub mod error;

use error::Result;
use ethereum_types::Address;
use serde::{Deserialize, Serialize};
use tea_sdk::ResultExt;

pub mod queries;
pub mod txns;

pub type BlockNumber = u64;
pub type CmlId = u64;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ContractAddresses {
	pub lock: String,
	pub maintainer: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum EventType {
	Topup,
	Withdraw,
	ValidatorChanged,
}

impl ContractAddresses {
	pub fn lock_address(&self) -> Result<Address> {
		self.string_to_address(&self.lock, "lock")
	}

	pub fn maintainer_address(&self) -> Result<Address> {
		self.string_to_address(&self.maintainer, "maintainer")
	}

	fn string_to_address(&self, addr: &str, _name: &str) -> Result<Address> {
		addr.parse().err_into()
	}
}

impl std::fmt::Display for EventType {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			EventType::Topup => write!(f, "topup"),
			EventType::Withdraw => write!(f, "withdraw"),
			EventType::ValidatorChanged => write!(f, "validator changed"),
		}
	}
}
