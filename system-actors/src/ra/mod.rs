use crate::nitro::{AttestationDocRequest, RaPeerRequest};
use serde::{Deserialize, Serialize};
use tea_codec::serde::TypeId;

pub mod error;

pub const NAME: &[u8] = b"com.tea.ra-actor";

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct VerifyPeerRequest {
	pub data: RaPeerRequest,
	pub is_seat: bool,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct VerifyPeerResponse(pub bool);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(Vec<u8>)]
pub struct ResponseVerifyPeer(pub AttestationDocRequest);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct OnStartMining(pub Vec<u8>);
