#![feature(min_specialization)]
#![feature(associated_type_defaults)]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]

pub mod prelude {
    pub use gluesql_core::prelude::{DataType, Key, Payload, Value};
}

#[macro_use]
extern crate log;
extern crate tea_codec as tea_sdk;

pub mod action;
pub mod actors;
pub mod error;
pub mod logging;
