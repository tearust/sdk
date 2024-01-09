use crate::error::{Error, Result};
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
	async fn handle(&self, _: GetSystemTimeRequest) -> tea_sdk::Result<_> {
		Ok(GetSystemTimeResponse(Self::to_millis(SystemTime::now())?))
	}

	async fn handle(&self, WaitingForRequest(ms): WaitingForRequest) -> tea_sdk::Result<()> {
		tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
		Ok(())
	}

	// Ordinary associated functions
	fn to_millis(time: SystemTime) -> Result<u128> {
		Ok(time
			.duration_since(UNIX_EPOCH)
			.map_err(|_| Error::GetSystemTime)?
			.as_millis())
	}
}
