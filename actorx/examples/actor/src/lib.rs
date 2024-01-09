#![feature(min_specialization)]
#![allow(incomplete_features)]

use crate::error::ActorXExamplesActor;
use tea_actorx_examples_codec::{
	AddRequest, AddResponse, FactorialRequest, FactorialResponse, GetSystemTimeRequest,
	GetSystemTimeResponse, GreetingsRequest, NATIVE_ID, WASM_ID,
};
use tea_sdk::{
	actorx::{actor, hooks::Activate, println, ActorId, HandlerActor},
	serde::handle::handles,
};

pub mod error;

#[cfg(test)]
mod tests;

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
	async fn handle(&self, _: Activate) -> tea_sdk::Result<_> {
		println!("Activate!");
		Ok(())
	}

	async fn handle(&self, GreetingsRequest(name): _) -> tea_sdk::Result<_> {
		let GetSystemTimeResponse(time) = NATIVE_ID
			.call(GetSystemTimeRequest)
			.await
			.map_err(|e| ActorXExamplesActor::from(e))?;
		println!("Hello {name}, the system time is {time}.");
		Ok(())
	}

	async fn handle(&self, AddRequest(lhs, rhs): _) -> tea_sdk::Result<_> {
		Ok(AddResponse(lhs + rhs))
	}

	async fn handle(&self, FactorialRequest(arg): _) -> tea_sdk::Result<_> {
		Ok(FactorialResponse(if arg <= 2 {
			arg
		} else {
			arg * WASM_ID
				.call(FactorialRequest(arg - 1))
				.await
				.map_err(|e| ActorXExamplesActor::from(e))?
				.0
		}))
	}
}
