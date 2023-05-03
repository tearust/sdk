use crate::nitro::{AttestationDocRequest, RaPeerRequest};
use serde::{Deserialize, Serialize};
use tea_codec::serde::TypeId;

pub mod error;

pub const NAME: &[u8] = b"com.tea.ra-actor";
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct VerifyPeerRequest {
	pub data: RaPeerRequest,
	pub is_seat: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct VerifyPeerResponse(pub bool);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(Vec<u8>)]
pub struct ResponseVerifyPeer(pub AttestationDocRequest);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct OnStartMining(pub Vec<u8>);
