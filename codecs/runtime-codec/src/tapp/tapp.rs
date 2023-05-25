use crate::tapp::TokenId;
use serde::{Deserialize, Serialize};

#[doc(hidden)]
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct HostItem {
	pub cml_id: u64,
	pub host_height: u64,
	pub performance: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AccountBalance {
	pub tea_balance: String,
	pub token_balance: String,
	pub reserved_token_balance: String,
	pub allowance: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TappItem {
	pub tapp_id: TokenId,
	pub name: String,
	pub owner: String,
	pub ticker: String,
	pub detail: String,
	pub link: String,
	pub max_allowed_hosts: u32,
	pub min_allowed_hosts: u32,
	pub tapp_type: String,
	pub created_timestamp: u64,
	pub billing_mode: String,
	pub buy_curve_k: u32,
	pub sell_curve_k: u32,
	pub hosting_amount: u32,
	pub status: String,
	pub start_height: u64,
	pub hosts: Vec<HostItem>,
	pub consume_account_balance: String,

	pub total_supply: String,
	pub buy_price: String,
	pub sell_price: String,

	pub account_balance: AccountBalance,
	pub cid: String,
	pub dev_status: String,
}

impl TappItem {
	pub fn get_name(&self) -> &str {
		&self.name
	}
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DevTappItem {
	pub tapp_id: TokenId,
	pub name: String,
	pub owner: String,
	pub ticker: String,
	pub detail: String,
	pub tapp_type: String,
	pub cid: Option<String>,
	pub dev_status: String,
	pub actor_cid: Option<String>,
	pub actor_version: Option<u64>,
	pub state_actor_cid: Option<String>,
	pub state_actor_version: Option<u64>,
	pub actor_name: Option<String>,
	pub state_actor_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TappErrorLog {
	pub token_id: TokenId,
	pub actor_type: String,
	pub error_type: String,
	pub tea_id: Option<String>,
	pub details: String,
	pub create_at: u64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LeaderboardItem {
	pub address: String,
	pub init_asset_add: String,
	pub init_asset_sub: String,
	pub tea_balance: String,
	pub tea_deposit: String,
	pub seat_asset: String,
	pub token_asset: String,
	pub ref_code: Option<String>,
	pub email: Option<String>,
	pub telegram: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeedAuctionItem {
	pub cml_key: String,
	pub uri: String,
	pub status: String,
	pub create_at: String,

	pub latest_bidder: Option<String>,
	pub latest_price: Option<String>,
	pub bid_at: Option<String>,
	pub sold_at: Option<String>,

	pub base_price: String,
	pub step_price: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TappMetaData {
	pub name: String,
	pub owner: String,
	pub ticker: String,
	pub detail: String,
	pub link: String,
	pub tapp_type: String,
	pub cid: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TxnGasFeeItem {
	pub txn_name: String,
	pub fee: String,
}

const USER_DEV_TAPP_PREFIX: &str = "com.developer.";
pub fn from_dev_tapp_name(tapp_name: &str) -> String {
	str::replace(tapp_name, USER_DEV_TAPP_PREFIX, "")
}

pub fn to_dev_tapp_name(ori_name: &str) -> String {
	format!("{USER_DEV_TAPP_PREFIX}{ori_name}")
}
