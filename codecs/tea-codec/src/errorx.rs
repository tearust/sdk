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
	mem::{self, ManuallyDrop},
};

use crate::type_gym::{Equality, NotEqual, TypeMark};

pub struct Error {
	data: ThinBox<ErrorData<dyn Descriptee>>,
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
impl !NotWrappedError for TypeMark<Error> {}
impl NotWrappedError for Error {}
pub trait NotError {}
impl<T> NotError for T where TypeMark<T>: NotWrappedError {}

impl Error {
	fn new<T, S>(data: T, #[cfg(feature = "backtrace")] location: ErrorLocation) -> Self
	where
		T: NotError + Send + Sync + 'static,
		S: Scope,
	{
		Self {
			data: ThinBox::new_unsize(ErrorData {
				#[cfg(feature = "backtrace")]
				location,
				source: Dispatcher::<_, S>::new(data),
			}),
		}
	}
}

impl<T> From<T> for Error
where
	T: NotError + Send + Sync + 'static,
{
	default fn from(data: T) -> Self {
		Self::new(
			data,
			#[cfg(feature = "backtrace")]
			ErrorLocation::Native(Backtrace::capture()),
		)
	}
}

// impl<X, Y> From<Error> for Error
// where
// 	Equality<X, Y>: NotEqual,
// {
// 	fn from(source: Error) -> Self {
// 		Self { data: source.data }
// 	}
// }

// impl<'a, X, Y> From<&'a Error> for &'a Error
// where
// 	Equality<X, Y>: NotEqual,
// {
// 	fn from(scope: &'a Error) -> Self {
// 		scope.as_scope()
// 	}
// }

impl Error {
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

	pub fn into_scope(self) -> Error {
		Error { data: self.data }
	}

	pub fn as_scope(&self) -> &Error {
		unsafe { mem::transmute(self) }
	}

	pub fn back_cast<T, S>(self) -> Result<T, Self>
	where
		T: Send + Sync + 'static,
		S: Scope,
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

	pub fn back_cast_ref<T, S>(&self) -> Option<&T>
	where
		T: Send + Sync + 'static,
		S: Scope,
	{
		if self.data.source.type_id() == Some(TypeId::of::<T>()) {
			unsafe { Some(&(*(&self.data.source as *const _ as *const Dispatcher<T, S>)).data) }
		} else {
			None
		}
	}

	pub fn is_name_of<T, S>(&self) -> bool
	where
		T: Send + Sync + Default + 'static,
		S: Scope,
	{
		Dispatcher::<T, S>::new(Default::default()).name() == Some(self.name())
	}

	pub fn name_of<T, S>() -> Option<String>
	where
		T: Send + Sync + Default + 'static,
		S: Scope,
	{
		Dispatcher::<T, S>::new(Default::default())
			.name()
			.map(Into::into)
	}
}

impl Display for Error {
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		if let Some(value) = self.summary() {
			f.write_str(value.as_ref())?;
		}
		Ok(())
	}
}

impl Debug for Error {
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

impl std::error::Error for Error {}

pub trait ResultXExt {
	fn assume_error_into_backcast<E>(self) -> Option<E>
	where
		E: Send + Sync + 'static;
}

impl<T> ResultXExt for Result<T, Error> {
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

impl PartialEq for Error {
	fn eq(&self, other: &Self) -> bool {
		self.name() == other.name()
			&& self.summary() == other.summary()
			&& self.detail() == other.detail()
			&& self.inner() == other.inner()
	}
}

impl Eq for Error {}
