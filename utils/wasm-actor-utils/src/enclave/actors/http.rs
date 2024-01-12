use tea_actorx::ActorId;
use tea_runtime_codec::runtime::http::{FromHttpBytes, IntoHttpBytes};
use tea_sdk::ResultExt;

use crate::enclave::error::{Error, Result};

pub use http::{request, HeaderName, HeaderValue, Request, Response};

#[allow(async_fn_in_trait)]
#[doc(hidden)]
pub trait RequestExt {
	#[allow(async_fn_in_trait)]
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
		self.body(Vec::new())
			.map_err(|e| Error::HttpRequest(e.to_string()))?
			.request()
			.await
	}

	async fn request_result<T, E>(self) -> Result<Response<Result<T, E>>>
	where
		T: FromHttpBytes,
		E: FromHttpBytes,
	{
		self.body(Vec::new())
			.map_err(|e| Error::HttpRequest(e.to_string()))?
			.request_result()
			.await
	}
}
