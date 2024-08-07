use crate::tapp::{Account, Balance, TokenId};
use serde::{Deserialize, Serialize};

pub type CmlId = u64;

#[doc(hidden)]
#[derive(Debug, Serialize, Deserialize)]
pub struct EntitySettings {
	pub token_id: TokenId,
	pub name: String,
	pub owner: Account,
	pub ticker: String,
	pub detail: String,
	pub link: String,
	pub max_allowed_hosts: u32,
	pub tapp_type: String,
	pub billing_mode: String,
	pub buy_curve_k: u32,
	pub sell_curve_k: u32,
	pub init_amount: Balance,
	pub hosting_amount: Balance,
	pub cml_id: Option<CmlId>,
	// pub from_token_id: Option<TokenId>,
}
