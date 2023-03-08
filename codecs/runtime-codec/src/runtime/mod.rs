/// The version of the codec as seen on crates.io
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

use serde::{Deserialize, Serialize};

pub const OP_DELAY_PUBLISH: &str = "DelayPublish";
pub const OP_TEST_RELAY: &str = "OP_TEST_RELAY";
pub const OP_INTERCOM_MESSAGE: &str = "IntercomMessage";

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DelayMessage {
	pub delay_seconds: u64,
	/// The message subject or topic
	pub subject: String,
	/// The reply-to field of the subject. This will be empty if there is no reply subject
	pub reply_to: String,
	/// The raw bytes of the message. Encoding/contents is determined by applications out of band
	#[serde(with = "serde_bytes")]
	#[serde(default)]
	pub body: Vec<u8>,
}

pub mod error;
pub mod http;
pub mod ops;

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadActorMessage {
	pub manifest: String,
	#[serde(with = "serde_bytes")]
	#[serde(default)]
	pub wasm_bytes: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunActorWithParams {
	pub manifest: String,
	#[serde(with = "serde_bytes")]
	#[serde(default)]
	pub actor_bytes: Vec<u8>,
	#[serde(with = "serde_bytes")]
	#[serde(default)]
	pub params: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActorVersionMessage {
	pub version: String,
}

pub trait ClientOpts {
	fn enable(&self) -> bool;
}
