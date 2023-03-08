#![feature(min_specialization)]
#![feature(associated_type_defaults)]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]

pub mod client;
pub mod enclave;

#[macro_use]
extern crate log;
extern crate tea_codec as tea_sdk;
