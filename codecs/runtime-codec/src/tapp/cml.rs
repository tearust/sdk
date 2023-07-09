use crate::tapp::{machine::MiningInfo, Account, TimestampShort};
use serde::{Deserialize, Serialize};

pub type CmlId = u64;
pub type Performance = u32;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CmlIntrinsic {
	pub id: CmlId,
	pub owner: Account,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CmlVariable {
	pub init_at: Option<TimestampShort>,
	pub planted_at: Option<TimestampShort>,
	pub updated_at: Option<TimestampShort>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CmlInfo {
	pub intrinsic: CmlIntrinsic,
	pub approved_account: Option<String>,
	pub variable: CmlVariable,
	pub mining_info: Option<MiningInfo>,
}

impl CmlInfo {
	pub fn has_init(&self) -> bool {
		self.variable.init_at.is_some()
	}

	pub fn is_mining(&self) -> bool {
		self.variable.planted_at.is_some()
	}
}
