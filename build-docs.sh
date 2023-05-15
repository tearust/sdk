#!/bin/bash

cd $(dirname $0)

rm -rf ./target/doc
cargo doc --features wasm --release --no-deps