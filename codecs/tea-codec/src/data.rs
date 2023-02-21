use std::{
	borrow::Borrow,
	fmt::{Debug, Display},
	hash::Hash,
	ops::Deref,
};

use serde::{Deserialize, Serialize};

pub enum Soo<T>
where
	T: ToOwned + ?Sized + 'static,
{
	Static(&'static T),
	Owned(T::Owned),
}

impl<T> Serialize for Soo<T>
where
	T: ToOwned + ?Sized + 'static,
	T: Serialize,
{
	fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
	where
		S: serde::Serializer,
	{
		self.as_ref().serialize(serializer)
	}
}

impl<'a, T> Deserialize<'a> for Soo<T>
where
	T: ToOwned + ?Sized + 'static,
	&'a T: Deserialize<'a>,
{
	fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
	where
		D: serde::Deserializer<'a>,
	{
		Ok(Self::Owned(<&'a T>::deserialize(deserializer)?.to_owned()))
	}
}

impl<T> Clone for Soo<T>
where
	T: ToOwned + ?Sized + 'static,
{
	fn clone(&self) -> Self {
		match self {
			Self::Static(v) => Self::Static(v),
			Self::Owned(v) => Self::Owned(v.borrow().to_owned()),
		}
	}
}

impl<T> const Deref for Soo<T>
where
	T: ToOwned + ?Sized + 'static,
	T::Owned: ~const Borrow<T>,
{
	type Target = T;

	fn deref(&self) -> &Self::Target {
		match self {
			Soo::Static(v) => v,
			Soo::Owned(v) => v.borrow(),
		}
	}
}

impl<T> const AsRef<T> for Soo<T>
where
	T: ToOwned + ?Sized + 'static,
	T::Owned: ~const Borrow<T>,
{
	fn as_ref(&self) -> &T {
		self
	}
}

impl<T> const Borrow<T> for Soo<T>
where
	T: ToOwned + ?Sized + 'static,
	T::Owned: ~const Borrow<T>,
{
	fn borrow(&self) -> &T {
		self
	}
}

impl<T, Rhs, V> const PartialEq<Rhs> for Soo<T>
where
	T: ToOwned + ?Sized + ~const PartialEq<V> + 'static,
	T::Owned: ~const Borrow<T>,
	Rhs: ~const Deref<Target = V> + ?Sized,
	V: ?Sized,
{
	default fn eq(&self, other: &Rhs) -> bool {
		self.as_ref().eq(other)
	}
}

impl<T> Eq for Soo<T> where T: ToOwned + ?Sized + Eq + 'static {}

impl<T, Rhs, V> const PartialOrd<Rhs> for Soo<T>
where
	T: ToOwned + ?Sized + ~const PartialOrd<V> + 'static,
	T::Owned: ~const Borrow<T>,
	Rhs: ~const Deref<Target = V> + ?Sized,
	V: ?Sized,
{
	fn partial_cmp(&self, other: &Rhs) -> Option<std::cmp::Ordering> {
		self.as_ref().partial_cmp(other)
	}
}

impl<T> Ord for Soo<T>
where
	T: ToOwned + ?Sized + Ord + 'static,
	T::Owned: ~const Borrow<T>,
{
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.as_ref().cmp(other)
	}
}

impl<T> Debug for Soo<T>
where
	T: ToOwned + ?Sized + Debug + 'static,
	T::Owned: Debug,
{
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Static(v) => Debug::fmt(v, f),
			Self::Owned(v) => Debug::fmt(v, f),
		}
	}
}

impl<T> Display for Soo<T>
where
	T: ToOwned + ?Sized + Display + 'static,
	T::Owned: Display,
{
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		match self {
			Self::Static(v) => Display::fmt(v, f),
			Self::Owned(v) => Display::fmt(v, f),
		}
	}
}

impl<T> Default for Soo<T>
where
	T: ToOwned + ?Sized + 'static,
	T::Owned: Default,
{
	fn default() -> Self {
		Self::Owned(Default::default())
	}
}

impl<T> Hash for Soo<T>
where
	T: ToOwned + Hash + ?Sized + 'static,
{
	fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
		self.as_ref().hash(state)
	}
}

impl<T> From<&'static T> for Soo<T>
where
	T: ToOwned + ?Sized + 'static,
{
	fn from(value: &'static T) -> Self {
		Self::Static(value)
	}
}

impl<T> From<Box<T>> for Soo<T>
where
	T: ToOwned<Owned = Box<T>> + ?Sized + 'static,
{
	fn from(value: Box<T>) -> Self {
		Self::Owned(value)
	}
}

impl<T> From<Vec<T>> for Soo<[T]>
where
	T: 'static,
	[T]: ToOwned<Owned = Vec<T>>,
{
	fn from(value: Vec<T>) -> Self {
		Self::Owned(value)
	}
}

impl From<String> for Soo<str> {
	fn from(value: String) -> Self {
		Self::Owned(value)
	}
}

pub trait AsRefIntoVec {
	#[allow(clippy::wrong_self_convention)]
	fn as_ref_into_vec(self) -> Vec<u8>;
}

impl<T> AsRefIntoVec for T
where
	T: AsRef<[u8]>,
{
	default fn as_ref_into_vec(self) -> Vec<u8> {
		self.as_ref().to_vec()
	}
}

impl AsRefIntoVec for Vec<u8> {
	fn as_ref_into_vec(self) -> Vec<u8> {
		self
	}
}

pub trait AsRefToString {
	#[allow(clippy::wrong_self_convention)]
	fn as_ref_to_string(self) -> Soo<str>;
}

impl<T> AsRefToString for T
where
	T: AsRef<str>,
{
	default fn as_ref_to_string(self) -> Soo<str> {
		self.as_ref().to_owned().into()
	}
}

impl AsRefToString for String {
	fn as_ref_to_string(self) -> Soo<str> {
		self.into()
	}
}

impl AsRefToString for Soo<str> {
	fn as_ref_to_string(self) -> Soo<str> {
		self
	}
}
