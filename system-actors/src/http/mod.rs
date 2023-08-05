use serde::{Deserialize, Serialize};
use tea_codec::serde::TypeId;

pub mod error;

pub const NAME: &[u8] = b"tea:http";

pub use tea_runtime_codec::runtime::http::{HttpRequest, HttpResponse};

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct OracleHttpRequest {
	pub method: String,
	pub url: String,
	pub headers: Option<Vec<(String, String)>>,
	pub payload: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct OracleHttpResponse {
	pub text: String,
}
