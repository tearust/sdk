#![feature(min_specialization)]
#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]

pub mod error;
mod time_actor;

use error::Result;
use tea_actorx2_examples_codec::{AddRequest, AddResponse, GreetingsRequest, WASM_ID};
use tea_sdk::actorx2::{get_gas, set_gas, ActorExt, WasmActor, WithActorHost};

#[tokio::main]
async fn main() -> Result<()> {
	run().with_actor_host().await
}

async fn init() -> Result<()> {
	WasmActor::from_binary(include_bytes!(concat!(
		env!("OUT_DIR"),
		"/wasm32-unknown-unknown/release/tea_actorx2_examples_actor.wasm"
	)))
	.await?
	.register()
	.await?;

	time_actor::Actor.register().await?;
	Ok(())
}

async fn run() -> Result<()> {
	init().await?;

	set_gas(1000000);
	println!("gas: {}", get_gas());

	WASM_ID.call(GreetingsRequest("Alice".to_string())).await?;
	println!("gas: {}", get_gas());

	let AddResponse(r) = WASM_ID.call(AddRequest(123, 456)).await?;
	println!("r = {r}");
	println!("gas: {}", get_gas());

	Ok(())
}
