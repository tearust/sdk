use serde::{Deserialize, Serialize};
use tea_codec::serde::TypeId;
use tea_runtime_codec::tapp::{Account, PaymentInfo};

pub mod error;
pub mod txns;

pub const NAME: &[u8] = b"com.tea.payment-channel-actor";

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryChannelInfoRequest(pub Account);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryChannelInfoResponse {
	pub payer_list: Vec<PaymentInfo>,
	pub payee_list: Vec<PaymentInfo>,
}
