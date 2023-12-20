use serde::{Deserialize, Serialize};
use tea_sdk::{actorx::ActorId, serde::TypeId};

pub const NATIVE_ID: ActorId = ActorId::Static(b"com.tea.time-actor");

#[derive(Serialize, Deserialize, TypeId)]
pub struct GetSystemTimeRequest;

#[derive(Serialize, Deserialize, TypeId)]
pub struct GetSystemTimeResponse(pub u128);

#[derive(Serialize, Deserialize, TypeId)]
#[response(())]
pub struct WaitingForRequest(pub u64);
