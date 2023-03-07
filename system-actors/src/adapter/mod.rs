use serde::{Deserialize, Serialize};
use tea_codec::{pricing::Priced, serde::TypeId};

pub mod error;

pub const NAME: &[u8] = b"tea:adapter";
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(Vec<u8>)]
pub struct InternalHttpRequest(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct RegisterHttp(pub Vec<String>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(Vec<u8>)]
pub struct HttpRequest {
	pub action: String,
	pub payload: Vec<u8>,
}
