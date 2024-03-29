//! tea-codec
//!
//! # About the Tea Project (teaproject.org)
//!
//! Tea Project (Trusted Execution & Attestation) is a Wasm runtime build on top of RoT(Root of Trust)
//! from both trusted hardware environment ,GPS, and blockchain technologies. Developer, Host and Consumer
//! do not have to trust any others to not only protecting privacy but also preventing cyber attacks.
//! The execution environment under remoted attestation can be verified by blockchain consensys.
//! Crypto economy is used as motivation that hosts are willing run trusted computing nodes.
//!

//!

#![feature(thin_box)]
#![feature(auto_traits, negative_impls, never_type)]
#![feature(min_specialization)]
#![allow(incomplete_features)]
#![allow(stable_features)]
#![allow(internal_features)]
#![feature(async_fn_in_trait)]
#![feature(impl_trait_projections)]
#![feature(return_position_impl_trait_in_trait)]
#![feature(type_alias_impl_trait)]
#![feature(const_trait_impl)]
#![feature(pointer_byte_offsets)]
#![feature(rustc_attrs)]
#![feature(trait_alias)]
#![feature(marker_trait_attr)]
#![feature(adt_const_params)]
#![feature(generic_const_exprs)]
#![feature(associated_type_bounds)]
#![feature(impl_trait_in_assoc_type)]
#![feature(type_name_of_val)]
#![feature(associated_type_defaults)]

extern crate self as tea_sdk;

/// The version of the codec as seen on crates.io
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

use std::marker::PhantomData;
use std::ops::Deref;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use ::serde::{de::DeserializeOwned, Serialize};
use bincode::Options as _;
use errorx::{CannotBeNone, Global};
use futures::Future;
#[doc(hidden)]
pub type Result<T, E = Global> = std::result::Result<T, E>;

#[inline(always)]
fn bincode_options() -> impl bincode::Options {
	bincode::DefaultOptions::new()
		.with_limit(1048576 * 150)
		.with_fixint_encoding()
}

/// The standard function for serializing codec structs into a format that can be
/// used for message exchange between actor and host. Use of any other function to
/// serialize could result in breaking incompatibilities.
pub fn serialize<T>(item: &T) -> Result<Vec<u8>>
where
	T: Serialize,
{
	let buf = bincode_options().serialize(item)?;
	Ok(buf)
}

/// The standard function for de-serializing codec structs from a format suitable
/// for message exchange between actor and host. Use of any other function to
/// deserialize could result in breaking incompatibilities.
pub fn deserialize<T, B>(buf: B) -> Result<T>
where
	T: DeserializeOwned,
	B: AsRef<[u8]>,
{
	Ok(bincode_options().deserialize(buf.as_ref())?)
}

/// A helper trait to map the error variant of a `Result` to inferred type with `From`/`Into`
pub trait ResultExt {
	type Value;
	type Error;
	/// Map the error variant of a `Result` to inferred type with `From`/`Into`
	fn err_into<E>(self) -> Result<Self::Value, E>
	where
		E: From<Self::Error>;
}

impl<T, E> ResultExt for std::result::Result<T, E> {
	type Value = T;
	type Error = E;
	fn err_into<E2>(self) -> Result<Self::Value, E2>
	where
		E2: From<E>,
	{
		self.map_err(From::from)
	}
}

pub trait IntoGlobal {
	type Value;
	fn into_g<E>(self) -> Result<Self::Value, E>
	where
		E: From<Global>;
}

impl<T, E> IntoGlobal for std::result::Result<T, E>
where
	Global: From<E>,
{
	type Value = T;
	fn into_g<E2>(self) -> Result<Self::Value, E2>
	where
		E2: From<Global>,
	{
		Ok(self.map_err(|e| e.into())?)
	}
}

/// A helper trait to map `None` conditions to tea `Error`s
pub trait OptionExt {
	type Value;
	/// Map `None` condition to an error message with a const name of some value that is expected not to be `None`.
	fn ok_or_err(self, name: impl Into<String>) -> Result<Self::Value, Global>;

	/// Map `None` condition to an error message with a function generatred name of some value that is expected not to be `None`.
	fn ok_or_err_else<N, F>(self, name_factory: F) -> Result<Self::Value, Global>
	where
		N: Into<String>,
		F: FnOnce() -> N;
}

impl<T> OptionExt for Option<T> {
	type Value = T;
	fn ok_or_err(self, name: impl Into<String>) -> Result<Self::Value, Global> {
		self.ok_or_else(move || Global::from(CannotBeNone(name.into())).into())
	}

	fn ok_or_err_else<N, F>(self, name_factory: F) -> Result<Self::Value, Global>
	where
		N: Into<String>,
		F: FnOnce() -> N,
	{
		self.ok_or_else(move || Global::from(CannotBeNone(name_factory().into())).into())
	}
}

#[doc(hidden)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NoOp<T>(PhantomData<dyn Fn() -> T + Send + Sync>);

impl Future for NoOp<()> {
	type Output = ();
	fn poll(self: Pin<&mut Self>, _: &mut Context) -> Poll<Self::Output> {
		Poll::Ready(())
	}
}

impl<E> Future for NoOp<Result<(), E>> {
	type Output = Result<(), E>;
	fn poll(self: Pin<&mut Self>, _: &mut Context) -> Poll<Self::Output> {
		Poll::Ready(Ok(()))
	}
}

impl<T> NoOp<T> {
	#[allow(clippy::should_implement_trait)]
	pub fn default() -> T {
		NoOpDefault::default()
	}

	pub fn is_no_op() -> bool {
		<T as NoOpDefault>::is_no_op()
	}
}

impl<Scope> Default for NoOp<Scope> {
	fn default() -> Self {
		Self(PhantomData)
	}
}

trait NoOpDefault {
	fn default() -> Self;
	fn is_no_op() -> bool;
}

impl<T> NoOpDefault for T {
	default fn default() -> Self {
		unreachable!()
	}

	default fn is_no_op() -> bool {
		false
	}
}

impl<T> NoOpDefault for NoOp<T> {
	fn default() -> Self {
		NoOp(PhantomData)
	}

	fn is_no_op() -> bool {
		true
	}
}

#[doc(hidden)]
pub struct FixSend<Fut>(pub Fut);

unsafe impl<Fut> Send for FixSend<Fut> {}

impl<Fut> Future for FixSend<Fut>
where
	Fut: Future,
{
	type Output = Fut::Output;
	fn poll(
		self: std::pin::Pin<&mut Self>,
		cx: &mut std::task::Context<'_>,
	) -> std::task::Poll<Self::Output> {
		unsafe { Pin::new_unchecked(&mut (self.get_unchecked_mut().0)) }.poll(cx)
	}
}

#[doc(hidden)]
pub struct ArcIter<'x, Owner, Iter>
where
	Owner: ?Sized,
	for<'a> &'a Iter: IntoIterator,
{
	_arc: Arc<Owner>,
	inner: <&'x Iter as IntoIterator>::IntoIter,
}

impl<'x, Owner, Iter> Iterator for ArcIter<'x, Owner, Iter>
where
	for<'a> &'a Iter: IntoIterator<Item: Deref<Target: Clone>>,
{
	type Item = <<&'x Iter as IntoIterator>::Item as Deref>::Target;
	fn next(&mut self) -> Option<Self::Item> {
		self.inner.next().map(|x| x.clone())
	}
}

#[doc(hidden)]
pub trait ArcIterExt<'x> {
	fn arc_iter<F, Iter>(self: &Arc<Self>, selector: F) -> ArcIter<'x, Self, Iter>
	where
		for<'a> &'a Iter: IntoIterator<Item: Deref<Target: Clone>>,
		F: FnOnce(&Self) -> &Iter;
}

impl<'x, T> ArcIterExt<'x> for T
where
	T: 'x,
{
	fn arc_iter<F, Iter>(self: &Arc<Self>, selector: F) -> ArcIter<'x, Self, Iter>
	where
		for<'a> &'a Iter: IntoIterator<Item: Deref<Target: Clone>>,
		F: FnOnce(&Self) -> &Iter,
	{
		let _arc = self.clone();
		let slf = unsafe { &*(&**self as *const T) };
		let inner = selector(slf).into_iter();
		ArcIter { _arc, inner }
	}
}

#[doc(hidden)]
pub mod const_concat;
#[doc(hidden)]
pub mod data;
pub mod defs;
pub mod errorx;
#[doc(hidden)]
pub mod pricing;
#[cfg(feature = "runtime")]
pub mod runtime;
#[cfg(feature = "runtime")]
pub use runtime::Timeout;
#[cfg(feature = "runtime")]
pub use tea_codec_macros::{timeout_retry, timeout_retry_worker};
pub mod serde;
mod type_gym;

pub use type_gym::ImplDefault;
