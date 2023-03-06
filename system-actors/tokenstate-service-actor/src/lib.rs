#![feature(min_specialization)]

use serde::{Deserialize, Serialize};
use tapp_common::TimestampShort;
use tea_actorx_core::ActorId;
use tea_codec::serde::TypeId;

pub mod error;

extern crate tea_codec as tea_sdk;

pub const NAME: &[u8] = b"com.tea.tokenstate-service-actor";

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct IsOutdatedRequest(pub ActorId);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct IsOutdatedReply(pub bool);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(Vec<u8>)]
pub struct SaveToBufferRequest;

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct RestoreFromBuffer(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct RestoreFromLocal;

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct SavePersist;

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct UpgradeVersion {
	pub data: Vec<u8>,
	pub persist_only: bool,
	pub at_time: TimestampShort,
}
