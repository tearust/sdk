#![feature(min_specialization)]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]

use std::env::current_exe;

use actorx_example_codec::{AddRequest, AddResponse, HelloWorldRequest, WASM_ACTOR_NAME};
use actorx_example_host::{error::Result, time_actor::TimeActor};
use tea_sdk::actorx::{host::ActorHost, InstanceId};

use wasmer::wasmparser::Operator;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    let mut path = current_exe()?;
    path.pop();
    path.push("actorx_example_actor.wasm");
    let wasm = std::fs::read(path)?;

    let host = ActorHost::new();

    host.register_wasm(wasm, |op| match op {
        Operator::Call { function_index: _ } => 2,
        _ => 1,
    })?;
    host.register_native(|context| Ok(TimeActor::new(context)))?;

    host.multicast_0().await?.activate().await?;

    let example = host
        .registry(WASM_ACTOR_NAME)?
        .actor(&InstanceId::ZERO)
        .await?;
    let r1 = example.call(HelloWorldRequest("Alice".to_string())).await?;

    println!("Result: {r1:?}");

    let AddResponse(r2, test) = example.call(AddRequest(1, 2)).await?;

    println!("Result: {:?}, {}", r2, test.len());

    Ok(())
}
