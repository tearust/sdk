use serde::{Deserialize, Serialize};
use tea_codec::pricing::Priced;
use tea_codec::serde::TypeId;

pub mod actions;
pub mod error;

pub const NAME: &[u8] = b"tea:keyvalue";

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(0)]
#[response(())]
pub struct CleanExpiredRequest;
