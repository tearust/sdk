use std::time::Duration;

use serde::{Deserialize, Serialize};
use tea_codec::serde::TypeId;

pub mod error;

pub const SETTLEMENT_INTERVAL: Duration = Duration::from_secs(600);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct RegisterGasFeeHandlerRequest;

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct HandleGasFeeRequest(pub Vec<u8>);
