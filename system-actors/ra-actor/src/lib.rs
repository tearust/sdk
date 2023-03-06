#![feature(min_specialization)]

use serde::{Deserialize, Serialize};
use tea_actorx_core::RegId;
use tea_codec::serde::TypeId;
use tea_nitro_actor_codec::{AttestationDocRequest, RaPeerRequest};

pub mod error;

extern crate tea_codec as tea_sdk;

pub const NAME: &[u8] = b"com.tea.ra-actor";
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct VerifyPeer {
	pub data: RaPeerRequest,
	pub seq_number: u64,
	pub source: RegId,
	pub is_seat: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(Vec<u8>)]
pub struct ResponseVerifyPeer(pub AttestationDocRequest);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct OnStartMining(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct PeerVerified(pub bool, pub u64);
