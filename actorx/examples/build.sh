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
cd debug
../release/tas actorx_example_actor.wasm -k ../../actorx/examples/actorx-example-actor/key.pem -i ../../actorx/examples/actorx-example-actor/module_id.txt -a com.tea.time-actor -t 0000000000000000000000000000000000000000 