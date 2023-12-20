#![feature(min_specialization)]

pub mod native_a;
pub mod wasm_a;
pub mod wasm_b;

use serde::{Deserialize, Serialize};
use tea_sdk::serde::TypeId;

#[derive(Serialize, Deserialize, TypeId)]
#[response(())]
pub struct WasmSleep(pub u64);
