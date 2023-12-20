#![feature(min_specialization)]
#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]

use std::pin::Pin;

use crate::error::Result;
use tea_sdk::{
	actorx::{actor, hooks::Activate, println, ActorId, HandlerActor},
	serde::handle::handles,
};
use test_examples_codec::{
	native_a::{GetSystemTimeRequest, GetSystemTimeResponse, WaitingForRequest, NATIVE_ID},
	wasm_a::*,
	wasm_b::{PongRequest, PongResponse},
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

	async fn handle(&self, GreetingsRequest(name): _) -> Result<_> {
		let GetSystemTimeResponse(time) = NATIVE_ID.call(GetSystemTimeRequest).await?;
		println!("Hello {name}, the system time is {time}.");
		Ok(())
	}

	async fn handle(&self, FactorialRequest(arg): _) -> Result<_> {
		Ok(FactorialResponse(if arg <= 2 {
			arg
		} else {
			arg * WASM_ID.call(FactorialRequest(arg - 1)).await?.0
		}))
	}

	async fn handle(&self, WasmSleep(ms): WasmSleep) -> Result<()> {
		NATIVE_ID.call(WaitingForRequest(ms)).await?;
		Ok(())
	}

	async fn handle(
		&self,
		PingRequest {
			left_count,
			sleep_ms,
		}: PingRequest,
	) -> Result<_> {
		println!("PingRequest: left_count={}", left_count);

		if let Some(ms) = sleep_ms {
			NATIVE_ID.call(WaitingForRequest(ms)).await?;
		}

		let PongResponse { total_ticks } = test_examples_codec::wasm_b::WASM_ID
			.call(PongRequest {
				left_count,
				sleep_ms,
			})
			.await?;

		Ok(PingResponse {
			total_ticks: total_ticks + 1,
		})
	}
}
