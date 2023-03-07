use serde::{Deserialize, Serialize};
use tea_codec::serde::TypeId;

pub mod error;

pub const NAME: &[u8] = b"tea:http";
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct Request(pub String, pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct Response(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct HyperRequest();

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct HyperResponse(pub Vec<u8>);
