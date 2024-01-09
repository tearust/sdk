use crate::{init, set_gas, Result};
use mocktopus::mocking::{MockResult, Mockable};
use tea_actorx::{dump_sys_usages, error::ActorX, invoke_timeout_ms};
use tea_sdk::{actorx::WithActorHost, errorx::Global};
use test_examples_codec::{
	wasm_a::{WASM_ID as WASM_A, *},
	WasmSleep,
};
use tokio::sync::oneshot::channel;

#[tokio::test]
async fn waiting_for_instance_works() -> Result<()> {
	async {
		init(2, false, 1, false).await?;
		set_gas();

		// set invoke timeout to 100 mill-seconds
		invoke_timeout_ms.mock_safe(|| MockResult::Return(100));

		let FactorialResponse(r) = WASM_A.call(FactorialRequest(3)).await?;
		assert_eq!(r, 6);

		let (tx, rx) = channel();
		tea_sdk::actorx::spawn(async move {
			set_gas();
			// sleep 5 mill-seconds that is shorter than the invoke timeout of 100 mill-seconds
			WASM_A.call(WasmSleep(5)).await.unwrap();
			tx.send(()).unwrap();
		});
		let FactorialResponse(r) = WASM_A.call(FactorialRequest(3)).await?;
		rx.await.unwrap();
		assert_eq!(r, 6);

		let (tx, rx) = channel();
		tea_sdk::actorx::spawn(async move {
			set_gas();
			// sleep 200 mill-seconds that is longer than the invoke timeout of 100 mill-seconds
			WASM_A.call(WasmSleep(2000)).await.unwrap();
			tx.send(()).unwrap();
		});
		let r = WASM_A.call(FactorialRequest(3)).await;
		rx.await.unwrap();
		assert!(r.is_err()); // should be timeout error
		if let Err(e) = r {
			// assert_eq!(e.to_string(), "ActorX: Channel receiving timeout");
			assert!(matches!(
				e,
				ActorX::Global(Global::ChannelReceivingTimeout(_))
			));
		}

		Ok(())
	}
	.with_actor_host()
	.await
}

#[tokio::test]
async fn auto_increase_instance_works() -> Result<()> {
	async {
		init(1, true, 1, false).await?;
		set_gas();

		let FactorialResponse(r) = WASM_A.call(FactorialRequest(3)).await?; // auto increase to 2
		assert_eq!(r, 6);

		let FactorialResponse(r) = WASM_A.call(FactorialRequest(5)).await?; // auto increase to 4
		assert_eq!(r, 120);

		Ok(())
	}
	.with_actor_host()
	.await
}

#[tokio::test]
async fn auto_drop_instance_works() -> Result<()> {
	async {
		init(1, true, 1, false).await?;
		set_gas();

		print_memory("init")?;

		let FactorialResponse(r) = WASM_A.call(FactorialRequest(5)).await?; // auto increase to 4
		assert_eq!(r, 120);
		print_memory("after auto increase to 4")?;

		WASM_A.call(WasmSleep(1100)).await.unwrap();
		print_memory("after sleep 1100 mill-seconds")?;

		Ok(())
	}
	.with_actor_host()
	.await
}

fn print_memory(msg: &str) -> Result<()> {
	println!("{} - {}", msg, dump_sys_usages());
	Ok(())
}
