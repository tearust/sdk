use tea_actorx::ActorId;
use tea_runtime_codec::runtime::http::{FromHttpBytes, IntoHttpBytes};
use tea_sdk::ResultExt;

use crate::enclave::error::Result;

pub use http::*;

pub trait RequestExt {
	async fn request<T>(self) -> Result<Response<T>>
	where
		T: FromHttpBytes;

	async fn request_result<T, E>(self) -> Result<Response<Result<T, E>>>
	where
		T: FromHttpBytes,
		E: FromHttpBytes;
}

impl<B> RequestExt for Request<B>
where
	B: IntoHttpBytes,
{
	async fn request<T>(self) -> Result<Response<T>>
	where
		T: FromHttpBytes,
	{
		ActorId::Static(tea_system_actors::http::NAME)
			.call(tea_system_actors::http::HttpRequest::try_from(self)?)
			.await?
			.try_into()
			.err_into()
	}

	async fn request_result<T, E>(self) -> Result<Response<Result<T, E>>>
	where
		T: FromHttpBytes,
		E: FromHttpBytes,
	{
		let resp = ActorId::Static(tea_system_actors::http::NAME)
			.call(tea_system_actors::http::HttpRequest::try_from(self)?)
			.await?;

		Ok(if resp.status.is_success() {
			Response::try_from(resp)?.map(Ok)
		} else {
			Response::try_from(resp)?.map(Err)
		})
	}
}

impl RequestExt for request::Builder {
	async fn request<T>(self) -> Result<Response<T>>
	where
		T: FromHttpBytes,
	{
		self.body(Vec::new())?.request().await
	}

	async fn request_result<T, E>(self) -> Result<Response<Result<T, E>>>
	where
		T: FromHttpBytes,
		E: FromHttpBytes,
	{
		self.body(Vec::new())?.request_result().await
	}
}
