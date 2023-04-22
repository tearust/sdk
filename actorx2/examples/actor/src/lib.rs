#![feature(min_specialization)]
#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]

use crate::error::Result;
use tea_actorx2_examples_codec::{
	AddRequest, AddResponse, FactorialRequest, FactorialResponse, GetSystemTimeRequest,
	GetSystemTimeResponse, GreetingsRequest, NATIVE_ID, WASM_ID,
};
use tea_sdk::{
	actorx2::{actor, hooks::Activate, println, ActorId, HandlerActor},
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
	async fn handle(&self, _: Activate) -> Result<_> {
		println!("Activate!");
		Ok(())
	}

	async fn handle(&self, GreetingsRequest(name): _) -> Result<_> {
		let GetSystemTimeResponse(time) = NATIVE_ID.call(GetSystemTimeRequest).await?;
		println!("Hello {name}, the system time is {time}.");
		Ok(())
	}

	async fn handle(&self, AddRequest(lhs, rhs): _) -> Result<_> {
		Ok(AddResponse(lhs + rhs))
	}

	async fn handle(&self, FactorialRequest(arg): _) -> Result<_> {
		Ok(FactorialResponse(if arg <= 2 {
			arg
		} else {
			arg * WASM_ID.call(FactorialRequest(arg - 1)).await?.0
		}))
	}
}
