use std::marker::PhantomData;

use futures::Future;

use crate::{errorx::Scope, ResultExt};

use super::{
	error::{Error, Result, Serde, UnexpectedType},
	get_type_id, FromBytes, ToBytes,
};

pub use tea_codec_macros::handles;

pub trait Request {
	type Response;
}

impl Request for &[u8] {
	type Response = Vec<u8>;
}

pub struct Fail;

pub struct With<Req, Prev>(PhantomData<dyn Fn(Prev, Req)>);

#[doc(hidden)]
#[macro_export]
macro_rules! Handle {
	[] => { $crate::serde::handle::Fail };
	[$t0:ty] => { $crate::serde::handle::With<$t0, $crate::serde::handle::Fail> };
	[$t0:ty; $other:ty] => { $crate::serde::handle::With<$t0, $other> };
	[$t0:ty, $($tn:ty),*] => { $crate::serde::handle::With<$t0, $crate::Handle![$($tn),*]> };
	[$t0:ty, $($tn:ty),*; $other:ty] => { $crate::serde::handle::With<$t0, $crate::Handle![$($tn),*; $other]> };
}

pub trait Handles: Sized {
	type List: HandleList<Self>;
}

pub trait Handle<Req>
where
	Req: Request,
{
	async fn handle(&self, req: Req) -> Result<Req::Response, Error<impl Scope>>;
}

pub trait HandleBytes {
	type Handle<'a>: Future<Output = Result<Vec<u8>>> + 'a
	where
		Self: 'a;
	fn handle_bytes<'a>(&'a self, req: &'a [u8]) -> Self::Handle<'a>;
}

pub trait HandleList<H> {
	async fn handle(handler: &H, req: &[u8]) -> Option<Result<Vec<u8>>>;
}

impl<T> HandleBytes for T
where
	T: Handles,
{
	type Handle<'a> = impl Future<Output = Result<Vec<u8>>> + 'a
	where
		Self: 'a;

	fn handle_bytes<'a>(&'a self, req: &'a [u8]) -> Self::Handle<'a> {
		async move {
			T::List::handle(self, req).await.ok_or_else(|| {
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
}

impl<H> HandleList<H> for Fail {
	async fn handle(_: &H, _: &[u8]) -> Option<Result<Vec<u8>>> {
		None
	}
}

impl<H, Req, Prev> HandleList<H> for With<Req, Prev>
where
	H: Handle<Req>,
	Req: Request + for<'a> FromBytes<'a>,
	Prev: HandleList<H>,
	Req::Response: ToBytes,
{
	async fn handle(handler: &H, req: &[u8]) -> Option<Result<Vec<u8>>> {
		match Req::from_bytes(req) {
			Ok(req) => Some(
				handler
					.handle(req)
					.await
					.err_into()
					.and_then(|x| x.to_bytes().err_into()),
			),
			Err(_) => Prev::handle(handler, req).await,
		}
	}
}
