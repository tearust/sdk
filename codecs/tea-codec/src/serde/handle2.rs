use futures::Future;

use crate::{errorx::Scope, ResultExt};

use super::{
	error::{Error, Result, Serde, UnexpectedType},
	get_type_id,
	handle::{Fail, Request, With},
	FromBytes, ToBytes,
};

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
