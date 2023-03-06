#![feature(min_specialization)]
#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]

pub mod error;
#[cfg(test)]
mod tests;

use actorx_example_codec::{
	AddRequest, AddResponse, GetSystemTimeRequest, GetSystemTimeResponse, HelloWorldRequest,
	HelloWorldResponse, PostPrint, TIME_ACTOR_NAME,
};
use error::Result;
use tea_sdk::{
	actorx::runtime::{actor, call, post, println, Activate, RegId},
	serde::handle::{Handle, Handles},
	Handle,
};

actor!(ActorHandler);

#[derive(Default, Clone)]
struct ActorHandler;

impl Handles<()> for ActorHandler {
	type List = Handle![HelloWorldRequest, AddRequest, Activate];
}

impl Handle<(), HelloWorldRequest> for ActorHandler {
	async fn handle(
		self,
		HelloWorldRequest(name): HelloWorldRequest,
		_: (),
	) -> Result<HelloWorldResponse> {
		post(
			RegId::from(TIME_ACTOR_NAME).inst(0),
			PostPrint("yeah".to_string()),
		)
		.await?;
		let GetSystemTimeResponse(time) =
			call(RegId::from(TIME_ACTOR_NAME).inst(0), GetSystemTimeRequest).await?;
		Ok(HelloWorldResponse(format!(
			"Hello, {name}! The current time is {time}."
		)))
	}
}

impl Handle<(), AddRequest> for ActorHandler {
	async fn handle(self, AddRequest(a, b): AddRequest, _: ()) -> Result<AddResponse> {
		Ok(AddResponse(a + b, vec![1; 1]))
	}
}

impl Handle<(), Activate> for ActorHandler {
	async fn handle(self, _: Activate, _: ()) -> Result<()> {
		println!("Activate!");
		Ok(())
	}
}
