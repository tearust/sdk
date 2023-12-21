use crate::{init, set_gas, Result};
use tea_sdk::actorx::WithActorHost;
use test_examples_codec::wasm_b::{WASM_ID as WASM_B, *};

#[tokio::test]
async fn cache_worker_works() -> Result<()> {
	async {
		init(1, false, 1, false).await?;

		for i in 0..128 {
			set_gas();
			let AddResponse(r) = WASM_B.call(AddRequest(i, 1)).await?;
			assert_eq!(r, i + 1);
		}
		Ok(())
	}
	.with_actor_host()
	.await
}

#[cfg(feature = "integration")]
#[tokio::test]
async fn predicted_cache_worker_works() -> Result<()> {
	async {
		init(1, false, 1, false).await?;

		for i in 0..128 {
			set_gas();
			WASM_B.call(WasmSleep(1000)).await?;
			println!("call {} times", i);
		}
		Ok(())
	}
	.with_actor_host()
	.await
}
