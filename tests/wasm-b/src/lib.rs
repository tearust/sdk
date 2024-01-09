#![feature(min_specialization)]
#![allow(incomplete_features)]

use tea_sdk::{
	actorx::{actor, hooks::Activate, println, ActorId, HandlerActor},
	serde::handle::handles,
};
use test_examples_codec::{
	native_a::{WaitingForRequest, NATIVE_ID},
	wasm_a::{PingRequest, PingResponse},
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
	async fn handle(&self, _: Activate) -> tea_sdk::Result<_> {
		println!("Activate!");
		Ok(())
	}

	async fn handle(&self, AddRequest(lhs, rhs): _) -> tea_sdk::Result<_> {
		Ok(AddResponse(lhs + rhs))
	}

	async fn handle(&self, SubRequest(lhs, rhs): _) -> tea_sdk::Result<_> {
		Ok(SubResponse(lhs - rhs))
	}

	async fn handle(&self, AddWithWaitingRequest { lhs, rhs, sleep_ms }: _) -> tea_sdk::Result<_> {
		if let Some(ms) = sleep_ms {
			NATIVE_ID.call(WaitingForRequest(ms)).await?;
		}
		Ok(AddWithWaitingResponse(lhs + rhs))
	}

	async fn handle(&self, SubWithWaitingRequest { lhs, rhs, sleep_ms }: _) -> tea_sdk::Result<_> {
		if let Some(ms) = sleep_ms {
			NATIVE_ID.call(WaitingForRequest(ms)).await?;
		}
		Ok(SubWithWaitingResponse(lhs - rhs))
	}

	async fn handle(&self, WasmSleep(ms): WasmSleep) -> tea_sdk::Result<()> {
		NATIVE_ID.call(WaitingForRequest(ms)).await?;
		Ok(())
	}

	async fn handle(
		&self,
		PongRequest {
			left_count,
			sleep_ms,
		}: PongRequest,
	) -> tea_sdk::Result<_> {
		println!("PongRequest: left_count={}", left_count);

		if let Some(ms) = sleep_ms {
			NATIVE_ID.call(WaitingForRequest(ms)).await?;
		}

		if left_count == 0 {
			Ok(PongResponse { total_ticks: 1 })
		} else {
			let PingResponse { total_ticks } = test_examples_codec::wasm_a::WASM_ID
				.call(PingRequest {
					left_count: left_count - 1,
					sleep_ms,
				})
				.await?;
			Ok(PongResponse {
				total_ticks: total_ticks + 1,
			})
		}
	}
}
