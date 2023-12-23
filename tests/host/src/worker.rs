use crate::{init, set_gas, Result};
use tea_sdk::actorx::WithActorHost;
use test_examples_codec::{
	wasm_a::{WASM_ID as WASM_A, *},
	wasm_b::{WASM_ID as WASM_B, *},
};

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

#[tokio::test]
async fn single_actor_concurrent_requests_works() -> Result<()> {
	async {
		const LOOP_COUNT: u32 = 127;
		init(1, false, 2, false).await?;

		let (add_tx, add_rx) = tokio::sync::oneshot::channel();
		tea_sdk::actorx::spawn(async move {
			if let Err(e) = async {
				for i in 0..LOOP_COUNT {
					set_gas();
					let AddWithWaitingResponse(r) = WASM_B
						.call(AddWithWaitingRequest {
							lhs: i,
							rhs: 1,
							sleep_ms: Some(1),
						})
						.await?;
					assert_eq!(r, i + 1);
				}
				Ok(()) as Result<()>
			}
			.await
			{
				println!("Add Response error: {:?}", e);
				add_tx.send(false).unwrap();
			} else {
				add_tx.send(true).unwrap();
			}
		});

		let (sub_tx, sub_rx) = tokio::sync::oneshot::channel();
		tea_sdk::actorx::spawn(async move {
			if let Err(e) = async {
				for i in 1..=LOOP_COUNT {
					set_gas();
					let SubWithWaitingResponse(r) = WASM_B
						.call(SubWithWaitingRequest {
							lhs: i,
							rhs: 1,
							sleep_ms: Some(2),
						})
						.await?;
					assert_eq!(r, i - 1);
				}
				Ok(()) as Result<()>
			}
			.await
			{
				println!("Sub Response error: {:?}", e);
				sub_tx.send(false).unwrap();
			} else {
				sub_tx.send(true).unwrap();
			}
		});

		assert!(add_rx.await.unwrap());
		assert!(sub_rx.await.unwrap());

		Ok(())
	}
	.with_actor_host()
	.await
}

#[tokio::test]
async fn multi_actors_concurrent_requests_works() -> Result<()> {
	async {
		const LOOP_COUNT: u32 = 127;
		init(1, false, 2, false).await?;

		let (add_tx, add_rx) = tokio::sync::oneshot::channel();
		tea_sdk::actorx::spawn(async move {
			if let Err(e) = async {
				for i in 0..LOOP_COUNT {
					set_gas();
					let AddWithWaitingResponse(r) = WASM_B
						.call(AddWithWaitingRequest {
							lhs: i,
							rhs: 1,
							sleep_ms: Some(1),
						})
						.await?;
					assert_eq!(r, i + 1);
				}
				Ok(()) as Result<()>
			}
			.await
			{
				println!("Add Response error: {:?}", e);
				add_tx.send(false).unwrap();
			} else {
				add_tx.send(true).unwrap();
			}
		});

		let (mul_tx, mul_rx) = tokio::sync::oneshot::channel();
		tea_sdk::actorx::spawn(async move {
			if let Err(e) = async {
				for i in 1..=LOOP_COUNT {
					set_gas();
					let MulWithWaitingResponse(r) = WASM_A
						.call(MulWithWaitingRequest {
							lhs: i,
							rhs: 2,
							sleep_ms: Some(1),
						})
						.await?;
					assert_eq!(r, i * 2);
				}
				Ok(()) as Result<()>
			}
			.await
			{
				println!("Mul Response error: {:?}", e);
				mul_tx.send(false).unwrap();
			} else {
				mul_tx.send(true).unwrap();
			}
		});

		assert!(add_rx.await.unwrap());
		assert!(mul_rx.await.unwrap());

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
