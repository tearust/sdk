//! The error messege marshalling system for tea project

mod aggregate;
mod global;
mod serde;

pub use global::{BadBinaryFormat, CannotBeNone, Global, RoutineTimeout};

pub use smallvec::{smallvec, SmallVec};

#[cfg(feature = "backtrace")]
use std::backtrace::Backtrace;
