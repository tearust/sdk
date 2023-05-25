use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use tea_codec::pricing::Priced;
use tea_codec::serde::TypeId;

pub mod error;

pub const NAME: &[u8] = b"tea:persist";

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(0)]
pub struct ProtoRequest(pub Cow<'static, str>, pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ProtoResponse(pub Vec<u8>);
