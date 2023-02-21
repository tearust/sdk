use std::{
	any::{type_name, Any},
	future::Future,
	mem::size_of,
	pin::Pin,
};

use tea_codec::{
	errorx::Scope,
	serde::{FromBytes, ToBytes},
	ResultExt,
};

use crate::error::{ArgsTypeMismatch, Error, Result};

const LEN_FPTR: usize = size_of::<fn(&[u8]) -> Result<()>>();

pub auto trait NoPtr {}
impl<T> !NoPtr for *const T {}
impl<T> !NoPtr for *mut T {}

pub fn erase_callback<F, S, T, A>(v: T, f: F) -> Result<Vec<u8>>
where
	F: FnOnce(T, &mut A) -> Result<(), Error<S>> + NoPtr + 'static,
	S: Scope,
	T: for<'a> FromBytes<'a> + ToBytes,
	A: Any,
{
	let mut data = vec![0u8; LEN_FPTR + size_of::<usize>() + size_of::<F>()];
	unsafe {
		let (data, rest) = data.split_at_mut(LEN_FPTR);
		(data as *mut [u8] as *mut u8 as *mut fn(&[u8], &[u8], &mut dyn Any) -> Result<()>)
			.write(dispatch::<F, S, T, A>);
		let (data, state) = rest.split_at_mut(size_of::<usize>());
		(data as *mut [u8] as *mut u8 as *mut usize).write(size_of::<F>());
		(state as *mut [u8] as *mut u8 as *mut F).write(f);
	}
	v.write_to(&mut data)?;
	Ok(data)
}

pub fn apply_callback<A>(data: Vec<u8>, a: &mut A) -> Result<()>
where
	A: Any,
{
	unsafe {
		let (data, rest) = data.split_at(LEN_FPTR);
		let fptr = (data as *const [u8] as *const u8
			as *const fn(&[u8], &[u8], &mut dyn Any) -> Result<()>)
			.read();
		let (data, rest) = rest.split_at(size_of::<usize>());
		let len = (data as *const [u8] as *const u8 as *const usize).read();
		let (state, payload) = rest.split_at(len);
		fptr(state, payload, a)
	}
}

fn dispatch<F, S, T, A>(f: &[u8], p: &[u8], a: &mut dyn Any) -> Result<()>
where
	F: FnOnce(T, &mut A) -> Result<(), Error<S>>,
	S: Scope,
	T: for<'a> FromBytes<'a> + ToBytes,
	A: Any,
{
	let f = unsafe { (f as *const [u8] as *const u8 as *const F).read() };
	let v = T::from_bytes(p)?;
	let a = if let Some(a) = a.downcast_mut() {
		a
	} else {
		return Err(ArgsTypeMismatch(type_name::<A>()).into());
	};
	f(v, a).err_into()
}

pub trait AsyncCallback<'a, T, A, S> {
	async fn call(self, cap: T, args: &'a mut A) -> Result<(), Error<S>>;
}

pub fn erase_callback_async<F, S, T, A>(v: T, f: F) -> Result<Vec<u8>>
where
	F: for<'a> AsyncCallback<'a, T, A, S> + NoPtr + 'static,
	S: Scope,
	T: for<'a> FromBytes<'a> + ToBytes,
	A: Any,
{
	let mut data = vec![0u8; LEN_FPTR + size_of::<usize>() + size_of::<F>()];
	unsafe {
		let (data, rest) = data.split_at_mut(LEN_FPTR);
		(data as *mut [u8] as *mut u8
			as *mut for<'a> fn(
				&'a [u8],
				&'a [u8],
				&'a mut dyn Any,
			) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>>)
			.write(dispatch_async::<F, S, T, A>);
		let (data, state) = rest.split_at_mut(size_of::<usize>());
		(data as *mut [u8] as *mut u8 as *mut usize).write(size_of::<F>());
		(state as *mut [u8] as *mut u8 as *mut F).write(f);
	}
	v.write_to(&mut data)?;
	Ok(data)
}

pub async fn apply_callback_async<A>(data: Vec<u8>, a: &mut A) -> Result<()>
where
	A: Any,
{
	unsafe {
		let (data, rest) = data.split_at(LEN_FPTR);
		let fptr = (data as *const [u8] as *const u8
			as *const for<'a> fn(
				&'a [u8],
				&'a [u8],
				&'a mut dyn Any,
			) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>>)
			.read();
		let (data, rest) = rest.split_at(size_of::<usize>());
		let len = (data as *const [u8] as *const u8 as *const usize).read();
		let (state, payload) = rest.split_at(len);
		fptr(state, payload, a).await
	}
}

impl<'a, X, Fut, T, A, S> AsyncCallback<'a, T, A, S> for X
where
	X: FnOnce(T, &'a mut A) -> Fut,
	Fut: Future<Output = Result<(), Error<S>>> + 'a,
	T: for<'x> FromBytes<'x> + ToBytes,
	A: Any,
	S: Scope,
{
	async fn call(self, cap: T, args: &'a mut A) -> Result<(), Error<S>> {
		self(cap, args).await
	}
}

fn dispatch_async<'a, F, S, T, A>(
	f: &'a [u8],
	p: &'a [u8],
	a: &'a mut dyn Any,
) -> Pin<Box<dyn Future<Output = Result<()>> + 'a>>
where
	F: for<'x> AsyncCallback<'x, T, A, S>,
	S: Scope,
	T: for<'x> FromBytes<'x> + ToBytes,
	A: Any,
{
	Box::pin(async move {
		let f = unsafe { (f as *const [u8] as *const u8 as *const F).read() };
		let v = T::from_bytes(p)?;
		let a = if let Some(a) = a.downcast_mut() {
			a
		} else {
			return Err(ArgsTypeMismatch(type_name::<A>()).into());
		};
		f.call(v, a).await.err_into()
	})
}

#[cfg(test)]
mod test_callback {
	use crate::{apply_callback_async, erase_callback_async};
	use tea_actorx_core::error::Result;

	#[tokio::test]
	async fn test() -> Result<()> {
		let callback = erase_callback_async(("abc".to_string(), "def".to_string()), callback_fn)?;

		async fn callback_fn(
			(a, b): (String, String),
			(x, y): &mut (&'static str, &'static str),
		) -> Result<()> {
			assert_eq!(a, *x);
			assert_eq!(b, *y);
			Ok(())
		}

		apply_callback_async(callback, &mut ("abc", "def")).await?;

		Ok(())
	}
}
