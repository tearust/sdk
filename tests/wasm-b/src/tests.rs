use crate::{Actor, Result};
use tea_sdk::actorx::ActorExt;

async fn init() -> Result<()> {
	Actor::default().register().await?;
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use tea_sdk::actorx::WithActorHost;
	use test_examples_codec::wasm_b::*;

	#[tokio::test]
	async fn add_test() -> Result<()> {
		async {
			init().await?;

			let AddResponse(result) = WASM_ID.call(AddRequest(1, 2)).await?;
			assert_eq!(result, 3);

			Ok(())
		}
		.with_actor_host()
		.await
	}

	#[tokio::test]
	async fn sub_test() -> Result<()> {
		async {
			init().await?;

			let SubResponse(result) = WASM_ID.call(SubRequest(5, 2)).await?;
			assert_eq!(result, 3);

			Ok(())
		}
		.with_actor_host()
		.await
	}
}
