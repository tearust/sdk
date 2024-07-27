use serde::{Deserialize, Serialize};
use tea_codec::serde::TypeId;
use tea_runtime_codec::tapp::{Account, ChannelId, PaymentInfo};

pub mod error;
pub mod txns;

pub const NAME: &[u8] = b"com.tea.payment-channel-actor";

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryChannelInfoRequest(pub Account, pub Option<u128>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryChannelInfoResponse {
	pub payer_list: Vec<PaymentInfo>,
	pub payee_list: Vec<PaymentInfo>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryChannelWithChannelIdRequest(pub Vec<ChannelId>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryChannelWithChannelIdResponse {
	pub list: Vec<PaymentInfo>,
}
