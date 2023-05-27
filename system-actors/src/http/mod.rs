use serde::{Deserialize, Serialize};
use tea_codec::serde::TypeId;

pub mod error;

pub const NAME: &[u8] = b"tea:http";

pub use tea_runtime_codec::runtime::http::{HttpRequest, HttpResponse};

/// Base request to send a oracle http request via tea system.
/// method could be GET, POST, PUT or DELETE.
/// url is the base request url, not that only support https.
/// headers is a vec truple, can input the base auth key or anything you need.
/// payload is the data body you need to post.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct OracleHttpRequest {
	pub method: String,
	pub url: String,
	pub headers: Option<Vec<(String, String)>>,
	pub payload: Option<String>,
}

/// Base response from sending an oracle http request via tea system.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct OracleHttpResponse {
	pub text: String,
}
