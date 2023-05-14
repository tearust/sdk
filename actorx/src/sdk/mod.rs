pub use crate::core::{actor::ActorId, metadata};
pub mod actor;

#[cfg(any(feature = "host", feature = "wasm"))]
pub mod context;
#[cfg(any(feature = "host", feature = "wasm"))]
pub use context::{caller, calling_stack, current};
#[cfg(feature = "host")]
pub use context::{cost, get_gas, set_gas};

pub mod hooks;
#[cfg(any(feature = "host", feature = "wasm"))]
#[doc(hidden)]
pub mod invoke;

mod calling_stack;
pub use calling_stack::CallingStack;
