use crate::Actor;
use tea_sdk::actorx::{ActorExt, WithActorHost};
use test_examples_codec::wasm_a::*;

async fn init() -> anyhow::Result<()> {
	Actor::default().register().await?;
	Ok(())
}

#[tokio::test]
async fn add_test() -> anyhow::Result<()> {
	async {
		init().await?;

		let FactorialResponse(result) = WASM_ID.call(FactorialRequest(1)).await?;
		assert_eq!(result, 1);

		let FactorialResponse(result) = WASM_ID.call(FactorialRequest(2)).await?;
		assert_eq!(result, 2);

		let FactorialResponse(result) = WASM_ID.call(FactorialRequest(5)).await?;
		assert_eq!(result, 120);

		Ok(())
	}
	.with_actor_host()
	.await
}
