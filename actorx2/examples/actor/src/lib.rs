#![feature(min_specialization)]
#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]

use crate::error::Result;
use tea_actorx2_examples_codec::{
	AddRequest, AddResponse, GetSystemTimeRequest, GetSystemTimeResponse, GreetingsRequest,
	NATIVE_ID, WASM_ID,
};
use tea_sdk::{
	actorx2::{actor, println, ActorId, ActorIdExt, HandlerActor},
	serde::handle2::{Handle, Handles},
	Handle,
};

pub mod error;

actor!(Actor);

#[derive(Default)]
pub struct Actor;

impl Handles for Actor {
	type List = Handle![GreetingsRequest, AddRequest];
}

impl HandlerActor for Actor {
	fn id(&self) -> Option<ActorId> {
		Some(WASM_ID)
	}
}

impl Handle<GreetingsRequest> for Actor {
	async fn handle(&self, GreetingsRequest(name): GreetingsRequest) -> Result<()> {
		let GetSystemTimeResponse(time) = NATIVE_ID.call(GetSystemTimeRequest).await?;
		println!("Hello {name}, the system time is {time}.");
		Ok(())
	}
}

impl Handle<AddRequest> for Actor {
	async fn handle(&self, AddRequest(lhs, rhs): AddRequest) -> Result<AddResponse> {
		Ok(AddResponse(lhs + rhs))
	}
}
