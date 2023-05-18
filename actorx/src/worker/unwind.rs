use std::{
	future::Future,
	panic::catch_unwind,
	pin::Pin,
	task::{Context, Poll},
};

use tea_sdk::errorx::{Scope, SyncResultExt};

use crate::error::Error;

pub struct Unwind<F>(Option<F>);

impl<F, T, S> Future for Unwind<F>
where
	F: Future<Output = Result<T, Error<S>>>,
	S: Scope,
{
	type Output = Result<T, Error<S>>;

	fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
		let slf = unsafe { self.get_unchecked_mut() };
		let inner = slf.0.as_mut().expect("Polling after completion.") as *mut F as *mut ();
		let cx = cx as *mut Context<'_>;
		match catch_unwind(move || {
			let inner = unsafe { Pin::new_unchecked(&mut *(inner as *mut F)) };
			let cx = unsafe { &mut *cx };
			inner.poll(cx)
		})
		.sync_err_into()
		{
			Ok(r) => r,
			Err(e) => {
				std::mem::forget(slf.0.take());
				Poll::Ready(Err(e))
			}
		}
	}
}

pub trait FutureExt: Future {
	fn force_unwind(self) -> Unwind<Self>
	where
		Self: Sized;
}

impl<T> FutureExt for T
where
	T: Future,
{
	fn force_unwind(self) -> Unwind<Self>
	where
		Self: Sized,
	{
		Unwind(Some(self))
	}
}
