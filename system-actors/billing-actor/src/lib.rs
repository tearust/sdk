#![feature(min_specialization)]

use std::time::Duration;

use serde::{Deserialize, Serialize};
pub use tea_actorx_core::billing::*;
use tea_codec::serde::TypeId;

pub mod error;

extern crate tea_codec as tea_sdk;

pub const SETTLEMENT_INTERVAL: Duration = Duration::from_secs(120);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct RegisterGasFeeHandlerRequest;

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct HandleGasFeeRequest(pub Vec<u8>);
