use serde::{Deserialize, Serialize};
use tea_sdk::{actorx::ActorId, serde::TypeId};

pub const WASM_ID: ActorId = ActorId::Static(b"com.tea.wasm-b-actor");

#[derive(Serialize, Deserialize, TypeId)]
pub struct AddRequest(pub u32, pub u32);

#[derive(Serialize, Deserialize, TypeId)]
pub struct AddResponse(pub u32);

#[derive(Serialize, Deserialize, TypeId)]
pub struct SubRequest(pub u32, pub u32);

#[derive(Serialize, Deserialize, TypeId)]
pub struct SubResponse(pub u32);

#[derive(Serialize, Deserialize, TypeId)]
pub struct PongRequest {
	pub left_count: u32,
	pub sleep_ms: Option<u64>,
}

#[derive(Serialize, Deserialize, TypeId)]
pub struct PongResponse {
	pub total_ticks: u32,
}

#[derive(Serialize, Deserialize, TypeId)]
pub struct AddWithWaitingRequest {
	pub lhs: u32,
	pub rhs: u32,
	pub sleep_ms: Option<u64>,
}

#[derive(Serialize, Deserialize, TypeId)]
pub struct AddWithWaitingResponse(pub u32);

#[derive(Serialize, Deserialize, TypeId)]
pub struct SubWithWaitingRequest {
	pub lhs: u32,
	pub rhs: u32,
	pub sleep_ms: Option<u64>,
}

#[derive(Serialize, Deserialize, TypeId)]
pub struct SubWithWaitingResponse(pub u32);
