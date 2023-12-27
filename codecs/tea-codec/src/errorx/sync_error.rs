use std::{
	fmt::{Debug, Display, Formatter},
	sync::Mutex,
};

use super::{DescriptableMark, Error, Scope};

pub struct SyncError<T>(Mutex<T>)
where
	T: ?Sized + Send + 'static;

impl<S, T> DescriptableMark<SyncError<T>> for S
where
	T: ?Sized + Send + 'static,
	S: Scope + DescriptableMark<T>,
{
}

impl<T> Display for SyncError<T>
where
	T: ?Sized + Display + Send + 'static,
{
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		let item = self.0.lock().unwrap();
		Display::fmt(&*item, f)
	}
}

impl<T> Debug for SyncError<T>
where
	T: ?Sized + Debug + Send + 'static,
{
	fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
		let item = self.0.lock().unwrap();
		Debug::fmt(&*item, f)
	}
}

impl<T> std::error::Error for SyncError<T> where T: ?Sized + std::error::Error + Send + 'static {}

#[cfg(feature = "checked")]
pub trait SyncErrorExt {
	fn sync_into<S>(self) -> Error<S>
	where
		S: Scope + DescriptableMark<Self>;
}

#[cfg(not(feature = "checked"))]
pub trait SyncErrorExt {
	fn sync_into(self) -> Error;
}

#[cfg(feature = "checked")]
impl<T> SyncErrorExt for T
where
	T: Send + 'static,
{
	fn sync_into(self) -> Error {
		SyncError(Mutex::new(self)).into()
	}
}

#[cfg(not(feature = "checked"))]
impl<T> SyncErrorExt for T
where
	T: Send + 'static,
{
	fn sync_into(self) -> Error {
		SyncError(Mutex::new(self)).into()
	}
}

#[cfg(feature = "checked")]
pub trait SyncResultExt {
	type Value;
	type Error;
	fn sync_err_into<S>(self) -> Result<Self::Value, Error<S>>
	where
		S: Scope + DescriptableMark<Self::Error>;
}

#[cfg(not(feature = "checked"))]
pub trait SyncResultExt {
	type Value;
	fn sync_err_into(self) -> Result<Self::Value, Error>;
}

#[cfg(feature = "checked")]
impl<T, E> SyncResultExt for Result<T, E>
where
	E: Send + 'static,
{
	type Value = T;
	type Error = E;
	fn sync_err_into<S>(self) -> Result<T, Error<S>>
	where
		S: Scope + DescriptableMark<E>,
	{
		self.map_err(|e| SyncError(Mutex::new(e)).into())
	}
}

#[cfg(not(feature = "checked"))]
impl<T, E> SyncResultExt for Result<T, E>
where
	E: Send + 'static,
{
	type Value = T;
	fn sync_err_into(self) -> Result<T, Error> {
		self.map_err(|e| SyncError(Mutex::new(e)).into())
	}
}
