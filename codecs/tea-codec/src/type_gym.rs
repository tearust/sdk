use std::{
	borrow::{Borrow, BorrowMut},
	fmt::{Debug, Display},
	ops::{Deref, DerefMut},
};

use serde::{Deserialize, Serialize};

use crate::errorx::NotError;

#[rustc_unsafe_specialization_marker]
pub trait Is<T> {}

impl<T> Is<T> for T {}

pub trait Irrelative {
	type Type;
}

impl<T> Irrelative for T
where
	T: ?Sized,
{
	type Type = !;
}

pub struct Equality<X, Y>(<X as Irrelative>::Type, <Y as Irrelative>::Type);
#[rustc_unsafe_specialization_marker]
pub auto trait NotEqual {}
impl<X> !NotEqual for Equality<X, X> {}

pub struct TypeMark<T>(<T as Irrelative>::Type)
where
	T: ?Sized;

#[rustc_specialization_trait]
trait SpecializedDebug = Debug;

trait DebugOrDefaultTrait {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl<T> DebugOrDefaultTrait for T
where
	T: ?Sized,
{
	default fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Ok(())
	}
}

impl<T> DebugOrDefaultTrait for T
where
	T: SpecializedDebug + ?Sized,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Debug::fmt(&self, f)
	}
}

#[rustc_specialization_trait]
trait SpecializedDisplay = Display;

trait DisplayOrDefaultTrait {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
}

impl<T> DisplayOrDefaultTrait for T
where
	T: ?Sized,
{
	default fn fmt(&self, _: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Ok(())
	}
}

impl<T> DisplayOrDefaultTrait for T
where
	T: SpecializedDisplay + ?Sized,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		Display::fmt(&self, f)
	}
}

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
#[repr(transparent)]
#[doc(hidden)]
pub struct ImplDefault<T>(pub T)
where
	T: ?Sized;

impl<T> Debug for ImplDefault<T>
where
	T: ?Sized,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		DebugOrDefaultTrait::fmt(&self.0, f)
	}
}

impl<T> Display for ImplDefault<T>
where
	T: ?Sized,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		DisplayOrDefaultTrait::fmt(&self.0, f)
	}
}

pub auto trait NotImplDefault {}
impl<T> !NotImplDefault for TypeMark<ImplDefault<T>> {}

impl<T> From<T> for ImplDefault<T>
where
	TypeMark<T>: NotImplDefault,
{
	fn from(value: T) -> Self {
		Self(value)
	}
}

#[allow(clippy::from_over_into)]
impl<T> Into<T> for ImplDefault<T>
where
	TypeMark<T>: NotImplDefault,
	T: NotError,
{
	fn into(self) -> T {
		todo!()
	}
}

impl<T> Deref for ImplDefault<T>
where
	T: ?Sized,
{
	type Target = T;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl<T> DerefMut for ImplDefault<T>
where
	T: ?Sized,
{
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.0
	}
}

impl<T> AsRef<T> for ImplDefault<T>
where
	T: ?Sized,
{
	fn as_ref(&self) -> &T {
		self
	}
}

impl<T> AsMut<T> for ImplDefault<T>
where
	T: ?Sized,
{
	fn as_mut(&mut self) -> &mut T {
		self
	}
}

impl<T> Borrow<T> for ImplDefault<T>
where
	T: ?Sized,
{
	fn borrow(&self) -> &T {
		self
	}
}

impl<T> BorrowMut<T> for ImplDefault<T>
where
	T: ?Sized,
{
	fn borrow_mut(&mut self) -> &mut T {
		self
	}
}
