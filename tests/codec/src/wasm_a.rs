use serde::{Deserialize, Serialize};
use tea_sdk::{actorx::ActorId, serde::TypeId};

pub const WASM_ID: ActorId = ActorId::Static(b"com.tea.wasm-a-actor");

#[derive(Serialize, Deserialize, TypeId)]
#[response(())]
pub struct GreetingsRequest(pub String);

#[derive(Serialize, Deserialize, TypeId)]
pub struct FactorialRequest(pub u64);

#[derive(Serialize, Deserialize, TypeId)]
pub struct FactorialResponse(pub u64);
