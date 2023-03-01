#![feature(min_specialization)]
#[macro_use]
extern crate serde_derive;
extern crate tea_codec as tea_sdk;

pub mod env;
pub mod error;
pub mod io;
pub mod message;
#[cfg(feature = "nitro")]
pub mod nitro;
pub mod persist;
pub mod state;
pub mod utils;

pub const ADAPTER_RPC_CHANNEL_NAME: &str = "adapter rpc";
pub const LAYER1_RPC_CHANNEL_NAME: &str = "layer1 rpc";

pub use error::Result;
pub use tea_sdk::*;
