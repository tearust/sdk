#![feature(min_specialization)]
#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]

use crate::error::Result;
use tea_sdk::{
	actorx::{actor, hooks::Activate, println, ActorId, HandlerActor},
	serde::handle::handles,
};
use test_examples_codec::{
	native_a::{WaitingForRequest, NATIVE_ID},
	wasm_b::*,
	WasmSleep,
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
	async fn handle(&self, _: Activate) -> Result<_> {
		println!("Activate!");
		Ok(())
	}

	async fn handle(&self, AddRequest(lhs, rhs): _) -> Result<_> {
		Ok(AddResponse(lhs + rhs))
	}

	async fn handle(&self, SubRequest(lhs, rhs): _) -> Result<_> {
		Ok(SubResponse(lhs - rhs))
	}

	async fn handle(&self, WasmSleep(ms): WasmSleep) -> Result<()> {
		NATIVE_ID.call(WaitingForRequest(ms)).await?;
		Ok(())
	}
}
