//! The error messege marshalling system for tea project

mod actorx;
mod aggregate;
mod global;

pub use actorx::*;
pub use global::{BadBinaryFormat, CannotBeNone, Global, RoutineTimeout};

pub use smallvec::{smallvec, SmallVec};

#[cfg(feature = "backtrace")]
use std::backtrace::Backtrace;
