#![feature(min_specialization)]
#![feature(new_uninit)]
#![allow(incomplete_features)]
#![feature(async_fn_in_trait)]
#![feature(allow_internal_unstable)]
#![feature(return_position_impl_trait_in_trait)]
#![feature(closure_track_caller)]
#![feature(auto_traits)]
#![feature(negative_impls)]

mod abi;
mod callbacks;
pub mod error;
mod interface;
#[cfg_attr(
	all(not(feature = "no-mock"), feature = "mock"),
	path = "runtime.mock.rs"
)]
mod runtime;

pub use abi::{_print, print_bytes, RUNTIME};
pub use callbacks::*;
pub use interface::CallingCx;
#[cfg(any(not(feature = "mock"), feature = "no-mock"))]
pub use interface::{handle, NoCallingCxWrapper};
pub use runtime::*;
pub use tea_actorx_core::{hook::*, ActorId, InstanceId, RegId};

#[doc(hidden)]
pub mod __hidden {
	pub use tokio;
}

extern crate tea_codec as tea_sdk;
