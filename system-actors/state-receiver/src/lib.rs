#![feature(min_specialization)]

use serde::{Deserialize, Serialize};
use tea_codec::{pricing::Priced, serde::TypeId};

pub mod error;

extern crate tea_codec as tea_sdk;

pub const NAME: &[u8] = b"tea:statereceiver";

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct HandleMessageRequest(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[response(())]
pub struct ActorTxnCheckMessage {
	pub from_b_node: bool,
	pub txn_bytes: Vec<u8>,
	pub should_freeze: bool,
}
