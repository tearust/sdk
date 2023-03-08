use crate::tapp::{machine::MiningInfo, Account, TimestampShort};
use serde::{Deserialize, Serialize};

pub type CmlId = u64;
pub type Performance = u32;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CmlIntrinsic {
	pub id: CmlId,
	pub owner: Account,
	pub attribute: CmlAttribute,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CmlAttributeRaw {
	#[serde(alias = "v")]
	pub version: u64,
	#[serde(alias = "d")]
	pub data: CmlAttribute,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CmlAttribute {
	#[serde(alias = "l")]
	pub lifespan: TimestampShort,
	#[serde(alias = "p")]
	pub performance: Performance,
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

	pub fn should_dead(&self, at_height: TimestampShort) -> bool {
		match self.variable.init_at {
			Some(init_at) => at_height >= init_at + self.intrinsic.attribute.lifespan,
			None => false,
		}
	}
}
