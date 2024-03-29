use tea_actorx_examples_codec::{AddRequest, AddResponse, WASM_ID};
use tea_sdk::actorx::{ActorExt, WithActorHost};

use crate::Actor;

async fn init() -> anyhow::Result<()> {
	Actor::default().register().await?;
	Ok(())
}

#[tokio::test]
async fn add_test() -> anyhow::Result<()> {
	async {
		init().await?;

		let AddResponse(result) = WASM_ID.call(AddRequest(1, 2)).await?;
		assert_eq!(result, 3);

		Ok(())
	}
	.with_actor_host()
	.await
}
