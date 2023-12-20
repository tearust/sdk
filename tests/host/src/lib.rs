#![feature(min_specialization)]
#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]

pub mod error;
mod time_actor;

#[cfg(all(test, feature = "__test"))]
mod timeout;

use error::Result;
use tea_sdk::actorx::{ActorExt, WasmActor};
#[cfg(feature = "timeout")]
use ::{std::time::Duration, tea_sdk::actorx::context::tracker};

#[allow(dead_code)]
async fn init(instance_a: u8, instance_b: u8) -> Result<()> {
	WasmActor::from_binary(
		include_bytes!(concat!(
			env!("OUT_DIR"),
			"/wasm32-unknown-unknown/release/wasm_a_actor.wasm"
		)),
		instance_a,
	)
	.await?
	.register()
	.await?;

	WasmActor::from_binary(
		include_bytes!(concat!(
			env!("OUT_DIR"),
			"/wasm32-unknown-unknown/release/wasm_b_actor.wasm"
		)),
		instance_b,
	)
	.await?
	.register()
	.await?;

	time_actor::Actor.register().await?;
	Ok(())
}

#[cfg(all(test, feature = "__test"))]
mod tests {
	use super::*;
	use tea_sdk::actorx::{set_gas, WithActorHost};
	use test_examples_codec::{
		wasm_a::{WASM_ID as WASM_A, *},
		wasm_b::{WASM_ID as WASM_B, *},
	};

	#[tokio::test]
	async fn basic_test() -> Result<()> {
		tracing_subscriber::fmt().init();
		async {
			init(5, 1).await?;
			set_gas(1000000);

			WASM_A.call(GreetingsRequest("Alice".to_string())).await?;

			let AddResponse(r) = WASM_B.call(AddRequest(123, 456)).await?;
			assert_eq!(r, 579);

			let FactorialResponse(r) = WASM_A.call(FactorialRequest(5)).await?;
			assert_eq!(r, 120);
			Ok(())
		}
		.with_actor_host()
		.await
	}
}
