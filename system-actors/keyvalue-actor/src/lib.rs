#![feature(min_specialization)]

use serde::{Deserialize, Serialize};
use tea_actorx_core::ActorId;
use tea_codec::pricing::Priced;
use tea_codec::serde::TypeId;

pub mod actions;
pub mod error;

extern crate tea_codec as tea_sdk;
pub mod manager;

pub const NAME: &[u8] = b"tea:keyvalue";

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(0)]
#[response(())]
pub struct BindActorRequest(pub ActorId);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(0)]
#[response(())]
pub struct CleanExpiredRequest;

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(0)]
pub struct ExportRequest;

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ExportResponse {
	pub data: Vec<u8>,
	pub actor_id: ActorId,
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(0)]
#[response(())]
pub struct ImportRequest {
	pub data: Vec<u8>,
	pub actor_id: ActorId,
}
