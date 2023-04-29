#![feature(min_specialization)]

use serde::{Deserialize, Serialize};
use tea_sdk::{actorx::ActorId, serde::TypeId};

pub const WASM_ID: ActorId = ActorId::Static(b"com.tea.examples-actor");
pub const NATIVE_ID: ActorId = ActorId::Static(b"com.tea.time-actor");

#[derive(Serialize, Deserialize, TypeId)]
#[response(())]
pub struct GreetingsRequest(pub String);

#[derive(Serialize, Deserialize, TypeId)]
pub struct AddRequest(pub u32, pub u32);

#[derive(Serialize, Deserialize, TypeId)]
pub struct AddResponse(pub u32);

#[derive(Serialize, Deserialize, TypeId)]
pub struct GetSystemTimeRequest;

#[derive(Serialize, Deserialize, TypeId)]
pub struct GetSystemTimeResponse(pub u128);

#[derive(Serialize, Deserialize, TypeId)]
pub struct FactorialRequest(pub u64);

#[derive(Serialize, Deserialize, TypeId)]
pub struct FactorialResponse(pub u64);
