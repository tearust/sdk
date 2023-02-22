#![feature(min_specialization)]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]
#![feature(return_position_impl_trait_in_trait)]
#![feature(type_changing_struct_update)]
#![feature(new_uninit)]
#![feature(associated_type_defaults)]
#![feature(iterator_try_collect)]
#![feature(auto_traits)]
#![feature(negative_impls)]

#[macro_use]
extern crate tracing;

pub mod actor;
pub mod billing;
pub mod error;
mod host;
mod registry;
pub use host::*;

extern crate tea_codec as tea_sdk;
