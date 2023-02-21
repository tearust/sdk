use std::fmt::Debug;

use serde::{Deserialize, Serialize};
use tea_codec::serde::TypeId;

pub const NAME: &[u8] = b"tea:billing";

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct GasFeeCostRequest(pub u64);
