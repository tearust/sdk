use std::time::Duration;

use serde::{Deserialize, Serialize};
use tea_codec::serde::TypeId;
use tea_runtime_codec::tapp::AccountId;

pub const NAME: &[u8] = b"tea:billing";

/// The duration for gas payment.
pub const SETTLEMENT_INTERVAL: Duration = Duration::from_secs(600);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct RegisterGasFeeHandlerRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct HandleGasFeeRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct GasFeeCostRequest {
	pub acct: AccountId,
	pub gas: u64,
}
