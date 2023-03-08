#![feature(min_specialization)]
#![feature(associated_type_defaults)]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]

#[macro_use]
extern crate log;
extern crate tea_codec as tea_sdk;

pub mod api;
mod error;
pub mod help;
mod query_cb;
pub mod request;
pub mod types;
pub mod utility;

pub use api::user::check_auth;
pub use error::{Errors, Result};

pub const CLIENT_DEFAULT_GAS_LIMIT: u64 = 100_000_000_u64;
