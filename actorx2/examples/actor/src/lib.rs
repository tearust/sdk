#![feature(min_specialization)]
#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]

use crate::error::Result;
use tea_actorx2_examples_codec::{
	AddRequest, AddResponse, GetSystemTimeRequest, GetSystemTimeResponse, GreetingsRequest,
	NATIVE_ID, WASM_ID,
};
use tea_sdk::{
	actorx2::{actor, println, ActorId, HandlerActor},
	serde::handle::handles,
};

pub mod error;

actor!(Actor);

#[derive(Default)]
pub struct Actor;

impl HandlerActor for Actor {
	fn id(&self) -> Option<ActorId> {
		Some(WASM_ID)
	}
}

#[handles]
impl Actor {
	async fn handle(&self, GreetingsRequest(name): _) -> Result<_> {
		let GetSystemTimeResponse(time) = NATIVE_ID.call(GetSystemTimeRequest).await?;
		println!("Hello {name}, the system time is {time}.");
		Ok(())
	}

	async fn handle(&self, AddRequest(lhs, rhs): _) -> Result<_> {
		Ok(AddResponse(lhs + rhs))
	}
}
