use crate::{init, set_gas, Result};
use tea_sdk::actorx::WithActorHost;
use test_examples_codec::wasm_a::{WASM_ID as WASM_A, *};

#[tokio::test]
async fn ping_pong_1_time_works() -> Result<()> {
	async {
		init(1, 1).await?;
		set_gas();

		let r = WASM_A
			.call(PingRequest {
				left_count: 0,
				sleep_ms: None,
			})
			.await?;
		assert_eq!(r.total_ticks, 2);
		Ok(())
	}
	.with_actor_host()
	.await
}

#[tokio::test]
async fn ping_pong_2_times_works() -> Result<()> {
	async {
		init(2, 2).await?;
		set_gas();

		let r = WASM_A
			.call(PingRequest {
				left_count: 1,
				sleep_ms: None,
			})
			.await?;
		assert_eq!(r.total_ticks, 4);

		Ok(())
	}
	.with_actor_host()
	.await
}
