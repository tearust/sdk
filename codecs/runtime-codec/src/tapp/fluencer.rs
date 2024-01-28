use crate::tapp::{Account, Balance, TokenId};
use serde::{Deserialize, Serialize};

#[doc(hidden)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FluencerCandidateListItem {
	pub id: String,
	pub r#type: String,
	pub key: String,
	pub url: String,
	pub reward_credit: bool,
}

#[doc(hidden)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreditSystemInfo {
	pub id: String,
	pub total: String,
	pub end_time: String,
	pub status: String,
}

#[doc(hidden)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AirdropTaskInfo {
	pub id: String,
	pub token_id: TokenId,
	pub owner: Account,
	pub name: String,
	pub reward_type: String,
	pub reward_text: String,
	pub r#type: String,
	pub balance: Option<Balance>,
}
