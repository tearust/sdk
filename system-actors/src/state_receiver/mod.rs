use serde::{Deserialize, Serialize};
use tea_codec::{pricing::Priced, serde::TypeId};

pub const NAME: &[u8] = b"tea:statereceiver";

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct HandleMessageRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[response(())]
pub struct ActorTxnCheckMessage {
	pub from_b_node: bool,
	pub txn_bytes: Vec<u8>,
	pub should_freeze: bool,
}
