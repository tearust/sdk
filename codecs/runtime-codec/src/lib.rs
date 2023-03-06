//! tea-codec
//!
//! # About the Tea Project (teaproject.org)
//!
//! Tea Project (Trusted Execution & Attestation) is a Wasm runtime build on top of RoT(Root of Trust)
//! from both trusted hardware environment ,GPS, and blockchain technologies. Developer, Host and Consumer
//! do not have to trust any others to not only protecting privacy but also preventing cyber attacks.
//! The execution environment under remoted attestation can be verified by blockchain consensys.
//! Crypto economy is used as motivation that hosts are willing run trusted computing nodes.
//!

//!

#![feature(min_specialization)]

/// The version of the codec as seen on crates.io
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
#[macro_use]
extern crate serde_derive;
extern crate log;
extern crate tea_codec as tea_sdk;

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
