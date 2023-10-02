// VMH definitions of runtime
pub mod env;
pub mod error;
pub mod io;
pub mod ipfs;
pub mod message;
#[cfg(feature = "nitro")]
pub mod nitro;
pub mod persist;
pub mod registry;
pub mod state;
pub mod utils;

pub const ADAPTER_RPC_CHANNEL_NAME: &str = "adapter rpc";
pub const LAYER1_RPC_CHANNEL_NAME: &str = "layer1 rpc";

pub use error::Result;
pub use tea_sdk::*;
