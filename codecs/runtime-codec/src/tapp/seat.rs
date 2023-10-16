use crate::tapp::{
	error::{Error, Errors, Result},
	Account, Balance, BlockNumber, TimestampShort, DOLLARS,
};
// use primitive_types::U256;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use u256_literal::u256;

pub type SeatId = u64;

pub const UNIT_TEA: Balance = DOLLARS;
pub const DAY_BLOCK: TimestampShort = 150 * 12; // 7200 as 1 day
pub const DISABLE_OP_BLOCK: TimestampShort = 15 * 12; // Cannot do anything, only for cronjob eveny day.
pub const MIN_DEPOSIT_FOR_SEAT: Balance = u256!(300_000_000_000_000_000_000);
pub const USDT_30_MIN_FEE_FOR_SEAT: Balance = u256!(140_000_000_000_000_000);

#[doc(hidden)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeatMaintainer {
	pub tea_id: Vec<u8>,
	pub conn_id: String,
	pub status: MaintainStatus,
	pub maintainer: Account,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MaintainStatus {
	Inactive,
	Active,
}

#[doc(hidden)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeatInfo {
	pub id: SeatId,
	pub maintainer: Option<SeatMaintainer>,

	pub init_at: Option<BlockNumber>,
	pub updated_at: Option<BlockNumber>,

	pub owner: Account,
	pub market: Option<SeatMarketInfo>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize, Deserialize)]
pub enum SeatMarketStatus {
	Inactive,
	Pending,
	Active,
}

#[doc(hidden)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SeatMarketInfo {
	pub seat_id: SeatId,

	pub owner: Option<Account>,
	pub price: Balance,

	pub deal_price: Option<Balance>,
	pub deal_user: Option<Account>,
	pub deal_at: Option<BlockNumber>,

	pub estimate_price: Option<Balance>,
	pub estimate_at: Option<BlockNumber>,

	pub status: SeatMarketStatus,

	pub deposit: Balance,
	pub usdt_deposit: Balance,
}

impl FromStr for SeatMarketStatus {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"inactive" => Ok(SeatMarketStatus::Inactive),
			"pending" => Ok(SeatMarketStatus::Pending),
			"active" => Ok(SeatMarketStatus::Active),
			_ => Err(Errors::ParseMarketStatus(s.to_string()).into()),
		}
	}
}

impl Display for SeatMarketStatus {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			SeatMarketStatus::Inactive => write!(f, "Inactive"),
			SeatMarketStatus::Active => write!(f, "Active"),
			SeatMarketStatus::Pending => write!(f, "Pending"),
		}
	}
}
impl Default for SeatMarketStatus {
	fn default() -> Self {
		SeatMarketStatus::Inactive
	}
}

impl SeatInfo {
	pub fn to_json_item(seat: SeatInfo) -> serde_json::Value {
		let j = SeatInfoJson {
			id: seat.id,
			maintainer: seat
				.maintainer
				.as_ref()
				.map(|v| format!("{:?}", v.maintainer)),
			init_at: seat.init_at,
			updated_at: seat.updated_at,
			miner_tea_id: seat
				.maintainer
				.as_ref()
				.map(|v| base64::encode(&v.tea_id))
				.unwrap_or_default(),
			miner_ip: seat
				.maintainer
				.as_ref()
				.map(|v| v.conn_id.clone())
				.unwrap_or_default(),
			miner_status: seat
				.maintainer
				.as_ref()
				.map(|v| v.status.to_string())
				.unwrap_or_default(),
			price: match seat.market {
				Some(ref m) => m.price.to_string(),
				_ => "".to_string(),
			},
			deal_price: match seat.market {
				Some(ref m) => m.deal_price.map(|p| p.to_string()),
				_ => None,
			},
			deal_user: match seat.market {
				Some(ref m) => m.deal_user.as_ref().map(|u| format!("{u:?}")),
				_ => None,
			},
			deal_at: match seat.market {
				Some(ref m) => m.deal_at,
				_ => None,
			},
			estimate_price: match seat.market {
				Some(ref m) => m.estimate_price.map(|p| p.to_string()),
				_ => None,
			},
			estimate_at: match seat.market {
				Some(ref m) => m.estimate_at,
				_ => None,
			},
			market_status: match seat.market {
				Some(ref m) => m.status.to_string(),
				_ => "".to_string(),
			},
			owner: match seat.market {
				Some(ref m) => match m.owner {
					Some(x) => format!("{x:?}"),
					None => "".to_string(),
				},
				_ => "".to_string(),
			},
			real_price: match seat.market {
				Some(ref m) => m.get_current_price().to_string(),
				_ => "".to_string(),
			},
			market_deposit: match seat.market {
				Some(ref m) => m.deposit.to_string(),
				_ => 0.to_string(),
			},
			usdt_deposit: match seat.market {
				Some(ref m) => m.usdt_deposit.to_string(),
				_ => 0.to_string(),
			},
		};

		serde_json::json!(j)
	}

	pub fn to_json(list: Vec<SeatInfo>) -> serde_json::Value {
		list.iter()
			.map(|item| SeatInfo::to_json_item(item.clone()))
			.collect::<serde_json::Value>()
	}
}

impl SeatMarketInfo {
	pub fn can_update_estimate(&self) -> bool {
		if self.estimate_price.is_none() {
			return true;
		}
		false
	}
	pub fn get_tax_price(&self) -> Balance {
		if let Some(e_price) = self.estimate_price {
			if e_price > self.price {
				return e_price;
			}
		}
		self.price
	}
	pub fn get_current_price(&self) -> Balance {
		if let Some(e_price) = self.estimate_price {
			return e_price;
		}
		self.price
	}
	pub fn can_giveup_ownership(&self) -> bool {
		if self.status == SeatMarketStatus::Active {
			return true;
		}
		false
	}
	pub fn can_deal(&self) -> bool {
		if self.status == SeatMarketStatus::Pending {
			return false;
		}
		true
	}
}

#[derive(Debug, Serialize, Deserialize)]
struct SeatInfoJson {
	pub id: SeatId,
	pub maintainer: Option<String>,

	pub init_at: Option<BlockNumber>,
	pub updated_at: Option<BlockNumber>,

	pub miner_tea_id: String,
	pub miner_ip: String,
	pub miner_status: String,

	pub owner: String,
	pub price: String,

	pub deal_price: Option<String>,
	pub deal_user: Option<String>,
	pub deal_at: Option<BlockNumber>,

	pub estimate_price: Option<String>,
	pub estimate_at: Option<BlockNumber>,

	pub market_status: String,

	pub real_price: String,
	pub market_deposit: String,
	pub usdt_deposit: String,
}

impl Default for MaintainStatus {
	fn default() -> Self {
		MaintainStatus::Inactive
	}
}

impl FromStr for MaintainStatus {
	type Err = Error;

	fn from_str(s: &str) -> Result<Self, Self::Err> {
		match s.to_lowercase().as_str() {
			"active" => Ok(MaintainStatus::Active),
			"inactive" => Ok(MaintainStatus::Inactive),
			_ => Err(Errors::ParseMaintainStatus(s.to_string()).into()),
		}
	}
}

impl Display for MaintainStatus {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		match self {
			MaintainStatus::Active => write!(f, "Active"),
			MaintainStatus::Inactive => write!(f, "Inactive"),
		}
	}
}
