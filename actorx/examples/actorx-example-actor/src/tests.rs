use actorx_example_codec::{
    AddRequest, AddResponse, HelloWorldRequest, HelloWorldResponse, WASM_ACTOR_NAME,
};
use actorx_example_host::time_actor::TimeActor;
use tea_sdk::actorx::runtime::{call, ActorHost, MockedActorName, RegId, RegisterMocked};

use crate::{error::Result, ActorHandler};

impl MockedActorName for ActorHandler {
    const NAME: &'static [u8] = b"actorx-example-actor";
}

#[tokio::test]
async fn actor_test() -> Result<()> {
    tracing_subscriber::fmt().init();
    let host = ActorHost::new();
    host.register_mocked(ActorHandler)?;
    host.register_native(|context| Ok(TimeActor::new(context)))?;
    tea_sdk::actorx::runtime::init_host(host);

    let HelloWorldResponse(r1) = call(
        RegId::from(WASM_ACTOR_NAME).inst(0),
        HelloWorldRequest("Alice".to_string()),
    )
    .await?;
    assert!(r1.starts_with("Hello, Alice!"));
    let AddResponse(r2, test) =
        call(RegId::from(WASM_ACTOR_NAME).inst(0), AddRequest(1, 2)).await?;

    assert_eq!(r2, 3);
    assert_eq!(test.len(), 65537);
    Ok(())
}
