//! The error messege marshalling system for tea project

mod aggregate;
mod global;
mod scope;
mod serde;
mod sync_error;

pub use global::{BadBinaryFormat, CannotBeNone, Global, RoutineTimeout};
pub use scope::*;
pub use sync_error::*;
pub use tea_codec_macros::define_scope;

pub use smallvec::{smallvec, SmallVec};

#[cfg(feature = "backtrace")]
use std::backtrace::Backtrace;
use std::{
	any::TypeId,
	borrow::Cow,
	boxed::ThinBox,
	fmt::{Debug, Display, Formatter},
	marker::PhantomData,
	mem::{self, ManuallyDrop},
};

use crate::type_gym::{Equality, NotEqual, TypeMark};

pub struct Error<S = ()> {
	data: ThinBox<ErrorData<dyn Descriptee>>,
	_p: PhantomData<S>,
}

#[cfg(feature = "backtrace")]
enum ErrorLocation {
	Native(Backtrace),
	Serialized(Cow<'static, str>),
}

struct ErrorData<T>
where
	T: ?Sized,
{
	#[cfg(feature = "backtrace")]
	location: ErrorLocation,
	source: T,
}

auto trait NotWrappedError {}
impl<S> !NotWrappedError for TypeMark<Error<S>> {}
impl<S> NotWrappedError for Error<S> {}
pub trait NotError {}
impl<T> NotError for T where TypeMark<T>: NotWrappedError {}

impl<S> Error<S>
where
	S: Scope,
{
	fn new<T>(data: T, #[cfg(feature = "backtrace")] location: ErrorLocation) -> Self
	where
		T: NotError + Send + Sync + 'static,
	{
		Self {
			data: ThinBox::new_unsize(ErrorData {
				#[cfg(feature = "backtrace")]
				location,
				source: Dispatcher::<_, S>::new(data),
			}),
			_p: PhantomData,
		}
	}
}

#[cfg(not(feature = "checked"))]
impl<T, S> From<T> for Error<S>
where
	T: NotError + Send + Sync + 'static,
	S: Scope,
{
	default fn from(data: T) -> Self {
		Self::new(
			data,
			#[cfg(feature = "backtrace")]
			ErrorLocation::Native(Backtrace::capture()),
		)
	}
}

#[cfg(feature = "checked")]
impl<T, S> From<T> for Error<S>
where
	T: NotError + Send + Sync + 'static,
	S: Scope + DescriptableMark<T>,
{
	default fn from(data: T) -> Self {
		Self::new(
			data,
			#[cfg(feature = "backtrace")]
			ErrorLocation::Native(Backtrace::capture()),
		)
	}
}

impl<X, Y> From<Error<X>> for Error<Y>
where
	Equality<X, Y>: NotEqual,
{
	fn from(source: Error<X>) -> Self {
		Self {
			data: source.data,
			_p: PhantomData,
		}
	}
}

impl<'a, X, Y> From<&'a Error<X>> for &'a Error<Y>
where
	Equality<X, Y>: NotEqual,
{
	fn from(scope: &'a Error<X>) -> Self {
		scope.as_scope()
	}
}

impl<S> Error<S> {
	pub fn name(&self) -> Cow<str> {
		if let Some(name) = self.data.source.name() {
			name
		} else {
			//panic!("Converting an error that is undefined.",);
			"".into()
		}
	}

	pub fn summary(&self) -> Option<Cow<str>> {
		self.data.source.summary()
	}

	pub fn human(&self) -> Option<String> {
		if let Some(detail) = self.data.source.detail() {
			let reg = regex::Regex::new(r"___([\s\S]*)___").unwrap();
			if let Some(r) = reg.captures(&detail) {
				if let Some(mm) = r.get(1) {
					return Some(mm.as_str().to_string());
				}
			}
		}
		None
	}

	pub fn detail(&self) -> Option<Cow<str>> {
		self.data.source.detail()
	}

	pub fn inner(&self) -> SmallVec<[&Error; 1]> {
		self.data.source.inner().unwrap_or_default()
	}

	#[cfg(feature = "backtrace")]
	pub fn backtrace(&self) -> Cow<str> {
		match &self.data.location {
			ErrorLocation::Native(location) => Cow::Owned(location.to_string()),
			ErrorLocation::Serialized(location) => Cow::Borrowed(location),
		}
	}

	pub fn into_scope<T>(self) -> Error<T> {
		Error {
			data: self.data,
			_p: PhantomData,
		}
	}

	pub fn as_scope<T>(&self) -> &Error<T> {
		unsafe { mem::transmute(self) }
	}

	pub fn back_cast<T>(self) -> Result<T, Self>
	where
		T: Send + Sync + 'static,
	{
		if self.data.source.type_id() == Some(TypeId::of::<T>()) {
			let mut data = self.data;
			unsafe {
				let result = (&mut (*(&mut data.source as *mut _ as *mut Dispatcher<T, S>)).data
					as *mut T)
					.read();
				mem::transmute::<_, ThinBox<ErrorData<ManuallyDrop<dyn Descriptee>>>>(data);
				Ok(result)
			}
		} else {
			Err(self)
		}
	}

	pub fn back_cast_ref<T>(&self) -> Option<&T>
	where
		T: Send + Sync + 'static,
	{
		if self.data.source.type_id() == Some(TypeId::of::<T>()) {
			unsafe { Some(&(*(&self.data.source as *const _ as *const Dispatcher<T, S>)).data) }
		} else {
			None
		}
	}

	pub fn is_name_of<T>(&self) -> bool
	where
		T: Send + Sync + Default + 'static,
		S: Scope,
	{
		Dispatcher::<T, S>::new(Default::default()).name() == Some(self.name())
	}

	pub fn name_of<T>() -> Option<String>
	where
		T: Send + Sync + Default + 'static,
		S: Scope,
	{
		Dispatcher::<T, S>::new(Default::default())
			.name()
			.map(Into::into)
	}
}

impl<S> Display for Error<S> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		if let Some(value) = self.summary() {
			f.write_str(value.as_ref())?;
		}
		Ok(())
	}
}

impl<S> Debug for Error<S> {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		f.write_fmt(format_args!("[{}]", self.name().as_ref()))?;
		if let Some(value) = self.summary() {
			f.write_str(value.as_ref())?;
		}
		if let Some(value) = self.detail() {
			f.write_fmt(format_args!("\n{value}"))?;
		}
		let inner = self.inner();
		for (index, &inner) in inner.iter().enumerate() {
			f.write_fmt(format_args!("\nInner Error {index}: {{{inner:?}}}"))?;
		}
		#[cfg(feature = "backtrace")]
		f.write_fmt(format_args!("\n{}", self.backtrace()))?;
		Ok(())
	}
}

impl<S> std::error::Error for Error<S> {}

pub trait ResultXExt {
	fn assume_error_into_backcast<E>(self) -> Option<E>
	where
		E: Send + Sync + 'static;
}

impl<T, S> ResultXExt for Result<T, Error<S>> {
	fn assume_error_into_backcast<E>(self) -> Option<E>
	where
		E: Send + Sync + 'static,
	{
		if let Err(e) = self {
			if let Ok(e) = e.back_cast() {
				return Some(e);
			}
		}
		None
	}
}

impl<S> PartialEq for Error<S> {
	fn eq(&self, other: &Self) -> bool {
		self.name() == other.name()
			&& self.summary() == other.summary()
			&& self.detail() == other.detail()
			&& self.inner() == other.inner()
	}
}

impl<S> Eq for Error<S> {}
