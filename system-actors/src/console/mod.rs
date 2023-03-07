use serde::{Deserialize, Serialize};
use tea_codec::{pricing::Priced, serde::TypeId};

pub mod error;

pub const NAME: &[u8] = b"tea:console";

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct UpgradeVersionRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct ImportStateRequest(pub String);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct DumpRegistryRequest;
