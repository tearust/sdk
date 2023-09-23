use serde::{Deserialize, Serialize};
use tea_actorx::ActorId;
use tea_codec::serde::TypeId;
use tea_runtime_codec::tapp::{TimestampShort, Ts};

pub mod error;

pub const NAME: &[u8] = b"com.tea.tokenstate-service-actor";

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct IsOutdatedRequest(pub ActorId);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct IsOutdatedResponse(pub bool);

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

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct SyncLocalStateRequest(pub Ts);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct SyncLocalStateResponse {
	pub ctxs: Option<Vec<u8>>,
	pub state: Option<EncryptedCheckpointState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedCheckpointState {
	pub data: Vec<u8>,
	pub ciphertext: String,
}
