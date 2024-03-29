use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{Error, Result};
use tea_actorx_examples_codec::{GetSystemTimeRequest, GetSystemTimeResponse, NATIVE_ID};
use tea_sdk::{
	actorx::{ActorId, HandlerActor},
	serde::handle::handles,
};

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

	// Ordinary associated functions
	fn to_millis(time: SystemTime) -> Result<u128> {
		Ok(time
			.duration_since(UNIX_EPOCH)
			.map_err(|_| Error::GetSystem)?
			.as_millis())
	}
}
