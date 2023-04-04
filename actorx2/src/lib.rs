#![feature(min_specialization)]
#![feature(const_trait_impl)]
#![feature(auto_traits)]
#![feature(negative_impls)]
#![feature(type_alias_impl_trait)]
#![feature(downcast_unchecked)]
#![feature(return_position_impl_trait_in_trait)]
#![feature(async_fn_in_trait)]
#![allow(incomplete_features)]

extern crate tea_codec as tea_sdk;
#[cfg(feature = "host")]
#[macro_use]
extern crate tracing;

pub use tea_actorx2_core::{actor::ActorId, metadata};
mod actor;
pub mod error;
pub use actor::{Actor, ActorSend, HandlerActor};

#[cfg(any(feature = "host", feature = "wasm"))]
mod context;
#[cfg(any(feature = "host", feature = "wasm"))]
pub use context::{caller, calling_stack, current, CallingStack};
#[cfg(feature = "host")]
pub use context::{cost, get_gas, set_gas};

pub mod hooks;
#[cfg(any(feature = "host", feature = "wasm"))]
mod invoke;

#[cfg(feature = "host")]
mod host;
#[cfg(feature = "host")]
pub use host::{spawn, ActorExt, HostActorIdExt, WasmActor, WithActorHost};

#[cfg(feature = "wasm")]
mod wasm;

#[cfg(feature = "wasm")]
pub use wasm::abi;
