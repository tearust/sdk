use serde::{Deserialize, Serialize};
use tea_codec::{pricing::Priced, serde::TypeId};

pub mod error;

pub const NAME: &[u8] = b"tea:console";

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct UpgradeVersionRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct ImportStateRequest(pub String);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct LoadActorRequest {
	pub file_path: String,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct DumpRegistryRequest;
