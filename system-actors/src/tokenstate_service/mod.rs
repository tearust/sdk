use serde::{Deserialize, Serialize};
use tea_actorx::ActorId;
use tea_codec::serde::TypeId;
use tea_runtime_codec::tapp::TimestampShort;

pub mod error;

pub const NAME: &[u8] = b"com.tea.tokenstate-service-actor";

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct IsOutdatedRequest(pub ActorId);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct IsOutdatedReply(pub bool);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(Vec<u8>)]
pub struct SaveToBufferRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct RestoreFromBuffer(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct RestoreFromLocal;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct SavePersist;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct UpgradeVersion {
	pub data: Vec<u8>,
	pub persist_only: bool,
	pub at_time: TimestampShort,
}
