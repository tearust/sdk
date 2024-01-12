#![feature(min_specialization)]
#![feature(const_trait_impl)]
#![feature(auto_traits)]
#![feature(negative_impls)]
#![feature(type_alias_impl_trait)]
#![feature(downcast_unchecked)]
#![cfg_attr(feature = "sign", feature(iterator_try_collect))]
#![cfg_attr(
	feature = "worker",
	feature(new_uninit, result_flattening, mutex_unpoison)
)]
#![cfg_attr(feature = "host", feature(unix_chown, map_try_insert))]
#![feature(return_position_impl_trait_in_trait)]
#![feature(async_fn_in_trait)]
#![feature(allow_internal_unstable)]
#![allow(incomplete_features)]
#![allow(stable_features)]
#![allow(internal_features)]
#![feature(impl_trait_in_assoc_type)]

extern crate tea_codec as tea_sdk;
#[cfg(any(feature = "host", feature = "worker"))]
#[macro_use]
extern crate tracing;

#[cfg(any(feature = "sdk", feature = "sign", feature = "worker"))]
mod core;

#[cfg(feature = "wasm")]
mod wasm;

#[cfg(feature = "host")]
mod host;

#[cfg(feature = "sign")]
pub mod sign;

#[cfg(feature = "worker")]
pub mod worker;

#[cfg(feature = "sdk")]
mod sdk;
#[cfg(feature = "sdk")]
pub use sdk::*;

#[cfg(any(feature = "sdk", feature = "sign", feature = "worker",))]
pub mod error;

mod export {
	#[allow(unused_imports)]
	#[cfg(any(feature = "sdk", feature = "sign", feature = "worker"))]
	pub use crate::core::{actor::ActorId, metadata};

	#[cfg(all(feature = "host", feature = "__test"))]
	pub use crate::host::invoke_timeout_ms;
	#[cfg(feature = "host")]
	pub use crate::host::{
		set_wasm_output_handler, spawn,
		sys::{dump_sys_usages, get_memory_usage},
		ActorExt, WasmActor, WithActorHost,
	};

	#[cfg(feature = "wasm")]
	pub use crate::wasm::abi;

	#[cfg(any(feature = "host", feature = "wasm"))]
	pub use crate::sdk::actor::*;
}

pub use export::*;
