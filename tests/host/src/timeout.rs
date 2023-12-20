use crate::{init, Result};
use tea_sdk::actorx::WithActorHost;
use test_examples_codec::{
	wasm_a::{WASM_ID as WASM_A, *},
	WasmSleep,
};
use tokio::sync::oneshot::channel;

#[tokio::test]
async fn waiting_for_instance_works() -> Result<()> {
	tracing_subscriber::fmt().init();
	async {
		init(2, 1).await?;
		set_gas();

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
			WASM_A.call(WasmSleep(200)).await.unwrap();
			tx.send(()).unwrap();
		});
		let r = WASM_A.call(FactorialRequest(3)).await;
		rx.await.unwrap();
		assert!(r.is_err()); // should be timeout error
		if let Err(e) = r {
			assert_eq!(e.name(), tea_actorx::error::ActorX::ChannelReceivingTimeout);
		}

		Ok(())
	}
	.with_actor_host()
	.await
}

fn set_gas() {
	tea_sdk::actorx::set_gas(1000000);
}
