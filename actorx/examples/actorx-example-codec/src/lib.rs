#![feature(min_specialization)]

use serde::{Deserialize, Serialize};
use tea_sdk::{pricing::Priced, serde::TypeId};

pub mod error;

pub const WASM_ACTOR_NAME: &[u8] = b"actorx-example-actor";

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct HelloWorldRequest(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct HelloWorldResponse(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct AddRequest(pub i32, pub i32);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct AddResponse(pub i32, pub Vec<u8>);

pub const TIME_ACTOR_NAME: &[u8] = b"com.tea.time-actor";

#[derive(Debug, Serialize, Deserialize, TypeId, Priced)]
#[price(1000)]
pub struct GetSystemTimeRequest;

#[derive(Debug, Serialize, Deserialize, TypeId)]
pub struct GetSystemTimeResponse(pub u128);

#[derive(Debug, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct PostPrint(pub String);
