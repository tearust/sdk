use crate::error::Result;
use std::time::{SystemTime, UNIX_EPOCH};
use tea_sdk::{
	actorx::{ActorId, HandlerActor},
	serde::handle::handles,
};
use test_examples_codec::native_a::*;

pub struct Actor;

impl HandlerActor for Actor {
	fn id(&self) -> Option<ActorId> {
		Some(NATIVE_ID)
	}
}

#[handles]
impl Actor {
	// Handles GetSystemTimeRequest
	async fn handle(&self, _: GetSystemTimeRequest) -> Result<_> {
		Ok(GetSystemTimeResponse(Self::to_millis(SystemTime::now())?))
	}

	// Ordinary associated functions
	fn to_millis(time: SystemTime) -> Result<u128> {
		Ok(time.duration_since(UNIX_EPOCH)?.as_millis())
	}
}
