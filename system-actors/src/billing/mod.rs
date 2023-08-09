use std::time::Duration;

use serde::{Deserialize, Serialize};
pub use tea_actorx_core::billing::*;
use tea_codec::serde::TypeId;
// use tea_runtime_codec::tapp::AccountId;

pub mod error;

pub const SETTLEMENT_INTERVAL: Duration = Duration::from_secs(120);
pub const NAME: &[u8] = b"tea:billing";

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct RegisterGasFeeHandlerRequest;

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct HandleGasFeeRequest(pub Vec<u8>);

// #[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
// #[response(())]
// pub struct GasFeeCostRequest {
// 	pub acct: AccountId,
// 	pub gas: u64,
// }
