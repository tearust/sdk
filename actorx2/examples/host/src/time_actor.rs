use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::Result;
use tea_actorx2_examples_codec::{GetSystemTimeRequest, GetSystemTimeResponse, NATIVE_ID};
use tea_sdk::{
	actorx2::{ActorId, HandlerActor},
	serde::handle2::{Handle, Handles},
	Handle,
};

pub struct Actor;

impl Handles for Actor {
	type List = Handle![GetSystemTimeRequest];
}

impl HandlerActor for Actor {
	fn id(&self) -> Option<ActorId> {
		Some(NATIVE_ID)
	}
}

impl Handle<GetSystemTimeRequest> for Actor {
	async fn handle(&self, _: GetSystemTimeRequest) -> Result<GetSystemTimeResponse> {
		Ok(GetSystemTimeResponse(
			SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis(),
		))
	}
}
