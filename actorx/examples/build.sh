#!/bin/sh

cd $(dirname $0)
cd actorx-example-actor
cargo build --release --target wasm32-unknown-unknown
cd ../actorx-example-host
cargo build
cd ../../actorx-signer
cargo build --release --bins
cd ../../target
cp wasm32-unknown-unknown/release/actorx_example_actor.wasm debug/
cd ../actorx/examples/actorx-example-actor
../../../target/release/tas ../../../target/debug/actorx_example_actor.wasm