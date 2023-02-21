use std::marker::PhantomData;

use futures::Future;

use crate::{errorx::Scope, ResultExt};

use super::{
	error::{Error, Serde, UnexpectedType},
	get_type_id, FromBytes, ToBytes,
};

pub trait Request {
	type Response;
}

impl Request for &[u8] {
	type Response = Vec<u8>;
}

pub trait Handles<Cx>: Sized {
	type List: HandleList<Self, Cx>;
}

pub trait HandlesSend<Cx>: Handles<Cx> {
	fn handle<'a>(
		self,
		req: &'a [u8],
		cx: Cx,
	) -> impl Future<Output = Result<Vec<u8>, Error<impl Scope + 'a>>> + Send + 'a
	where
		Cx: 'a;
}

mod send {
	use futures::Future;

	use crate::{serde::error::Error, FixSend};

	use super::{Handles, HandlesSend};

	type Scope<'a, T, Cx>
	where
		T: super::Handle<Cx, &'a [u8]> + 'a,
		Cx: 'a,
	= impl crate::errorx::Scope + 'a;
	pub type HandleSend<'a, T, Cx>
	where
		T: super::Handle<Cx, &'a [u8]> + 'a,
		Cx: 'a,
	= impl Future<Output = Result<Vec<u8>, Error<Scope<'a, T, Cx>>>> + 'a;
	pub fn handle_send<'a, T, Cx>(v: T, req: &'a [u8], cx: Cx) -> HandleSend<'a, T, Cx>
	where
		T: super::Handle<Cx, &'a [u8]> + 'a,
		Cx: 'a,
	{
		super::Handle::handle(v, req, cx)
	}

	impl<T, Cx> HandlesSend<Cx> for T
	where
		for<'a> T: Handles<Cx> + super::Handle<Cx, &'a [u8]> + Send + 'a,
		for<'a> HandleSend<'a, T, Cx>: Send,
		Cx: Send,
	{
		fn handle<'a>(
			self,
			req: &'a [u8],
			cx: Cx,
		) -> impl Future<Output = Result<Vec<u8>, Error<impl crate::errorx::Scope + 'a>>> + Send + 'a
		where
			Cx: 'a,
		{
			FixSend(async move { super::Handle::handle(self, req, cx).await })
		}
	}
}

pub use send::*;

pub trait Handle<Cx, Req>
where
	Req: Request,
{
	async fn handle(self, req: Req, cx: Cx) -> Result<Req::Response, Error<impl Scope>>;
}

pub trait HandleList<H, Cx> {
	async fn handle(handler: H, req: &[u8], cx: Cx) -> Result<Result<Vec<u8>, Error>, (H, Cx)>;
}

impl<T, Cx> Handle<Cx, &[u8]> for T
where
	T: Handles<Cx>,
{
	async fn handle(
		self,
		req: &[u8],
		cx: Cx,
	) -> Result<<&[u8] as Request>::Response, Error<impl Scope>> {
		T::List::handle(self, req, cx).await.map_err(|_| {
			Error::<Serde>::from(UnexpectedType(
				match get_type_id(req) {
					Ok(id) => id,
					Err(e) => return e,
				}
				.to_string(),
			))
		})?
	}
}

pub struct Fail;

impl<H, Cx> HandleList<H, Cx> for Fail {
	async fn handle(handler: H, _: &[u8], cx: Cx) -> Result<Result<Vec<u8>, Error>, (H, Cx)> {
		Err((handler, cx))
	}
}

pub struct With<Req, Prev>(PhantomData<dyn Fn(Prev, Req)>);

impl<H, Cx, Req, Prev> HandleList<H, Cx> for With<Req, Prev>
where
	H: Handle<Cx, Req>,
	Req: Request + for<'a> FromBytes<'a>,
	Prev: HandleList<H, Cx>,
	Req::Response: ToBytes,
{
	async fn handle(handler: H, req: &[u8], cx: Cx) -> Result<Result<Vec<u8>, Error>, (H, Cx)> {
		match Req::from_bytes(req) {
			Ok(req) => Ok(handler
				.handle(req, cx)
				.await
				.err_into()
				.and_then(|x| x.to_bytes().err_into())),
			Err(_) => Prev::handle(handler, req, cx).await,
		}
	}
}

#[macro_export]
macro_rules! Handle {
	[] => { $crate::serde::handle::Fail };
	[$t0:ty] => { $crate::serde::handle::With<$t0, $crate::serde::handle::Fail> };
	[$t0:ty; $other:ty] => { $crate::serde::handle::With<$t0, $other> };
	[$t0:ty, $($tn:ty),*] => { $crate::serde::handle::With<$t0, $crate::Handle![$($tn),*]> };
	[$t0:ty, $($tn:ty),*; $other:ty] => { $crate::serde::handle::With<$t0, $crate::Handle![$($tn),*; $other]> };
}
