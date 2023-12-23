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

#[derive(Serialize, Deserialize, TypeId)]
pub struct PingRequest {
	pub left_count: u32,
	pub sleep_ms: Option<u64>,
}

#[derive(Serialize, Deserialize, TypeId)]
pub struct PingResponse {
	pub total_ticks: u32,
}

#[derive(Serialize, Deserialize, TypeId)]
pub struct MulWithWaitingRequest {
	pub lhs: u32,
	pub rhs: u32,
	pub sleep_ms: Option<u64>,
}

#[derive(Serialize, Deserialize, TypeId)]
pub struct MulWithWaitingResponse(pub u32);
