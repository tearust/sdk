use bytes::{Buf, BufMut, Bytes};

use http::header::{HeaderName, HOST};
use serde::de::DeserializeOwned;
use tea_codec::{errorx::BadBinaryFormat, OptionExt};

use self::error::{Error, HttpErrorStatus, Result};

mod buf;
pub mod error;

pub use http::*;
pub use http_body::*;

pub trait RequestExt {
	async fn request_raw(self) -> Result<Response<Bytes>>;
	async fn request<T>(self) -> Result<T>
	where
		T: DeserializeOwned;
}

impl<B> RequestExt for Request<B>
where
	B: Body,
	<B as Body>::Error: Into<Error> + Send + Sync + 'static,
{
	async fn request_raw(self) -> Result<Response<Bytes>> {
		let uri = self.uri().to_string();
		let data = request_to_bytes(self).await?;
		let tea_system_actors::http::Response(result) = tea_actorx_runtime::call(
			tea_actorx_core::RegId::Static(tea_system_actors::http::NAME).inst(1),
			tea_system_actors::http::Request(uri.to_string(), data),
		)
		.await?;
		let result = Bytes::from(result);
		let mut headers = [httparse::EMPTY_HEADER; 32];
		let mut response = httparse::Response::new(&mut headers);
		let len_head = match response.parse(&result)? {
			httparse::Status::Complete(size) => size,
			httparse::Status::Partial => return Err(BadBinaryFormat.into()),
		};
		let mut r = Response::new(result.slice(len_head..));
		*r.status_mut() = StatusCode::from_u16(response.code.ok_or_err("status code")?)?;
		for header in response.headers {
			let name = header.name.parse::<HeaderName>()?;
			let value = HeaderValue::from_bytes(header.value)?;
			r.headers_mut().append(name, value);
		}
		Ok(r)
	}

	async fn request<T>(self) -> Result<T>
	where
		T: DeserializeOwned,
	{
		let uri = self.uri().to_string();
		let data = request_to_bytes(self).await?;
		let tea_system_actors::http::Response(result) = tea_actorx_runtime::call(
			tea_actorx_core::RegId::Static(tea_system_actors::http::NAME).inst(1),
			tea_system_actors::http::Request(uri.to_string(), data),
		)
		.await?;
		let mut headers = [httparse::EMPTY_HEADER; 32];
		let mut response = httparse::Response::new(&mut headers);
		let len_head = match response.parse(&result)? {
			httparse::Status::Complete(size) => size,
			httparse::Status::Partial => return Err(BadBinaryFormat.into()),
		};
		let status = StatusCode::from_u16(response.code.ok_or_err("status code")?)?;

		if !status.is_success() {
			return Err(HttpErrorStatus(status).into());
		}

		let body = &result[len_head..];

		if body.is_empty() {
			return Ok(serde_json::from_slice(b"null")?);
		}

		Ok(serde_json::from_slice(body)?)
	}
}

impl RequestExt for request::Builder {
	async fn request_raw(self) -> Result<Response<Bytes>> {
		self.body(Empty::<&[u8]>::new())?.request_raw().await
	}

	async fn request<T>(self) -> Result<T>
	where
		T: DeserializeOwned,
	{
		self.body(Empty::<&[u8]>::new())?.request().await
	}
}

async fn request_to_bytes<B>(req: Request<B>) -> Result<Vec<u8>>
where
	B: Body,
	<B as Body>::Error: Into<Error> + Send + Sync + 'static,
{
	let (mut head, body) = req.into_parts();
	if let (Some(host), false) = (head.uri.host(), head.headers.contains_key(HOST)) {
		if let Some(port) = head.uri.port() {
			head.headers.append(HOST, format!("{host}:{port}").parse()?);
		} else {
			head.headers.append(HOST, host.parse()?);
		}
	}
	let body = aggregate(body).await?;
	let head = format!(
		"{} {} {:?}\r\n",
		head.method.as_str(),
		head.uri.path_and_query().ok_or_err("uri")?.as_str(),
		head.version
	)
	.into_bytes()
	.into_iter()
	.chain(head.headers.iter().flat_map(|(k, v)| {
		k.as_str()
			.bytes()
			.chain(*b": ")
			.chain(v.as_bytes().iter().copied())
			.chain(*b"\r\n")
	}))
	.chain(*b"\r\n");
	let hint = {
		let (lower, upper) = head.size_hint();
		upper.unwrap_or(lower)
	} + body.remaining();
	let mut result = Vec::with_capacity(hint);
	result.extend(head);
	result.put(body);

	Ok(result)
}

async fn aggregate<T>(body: T) -> Result<impl Buf, T::Error>
where
	T: Body,
{
	let mut bufs = buf::BufList::new();

	tokio::pin!(body);
	while let Some(buf) = body.data().await {
		let buf = buf?;
		if buf.has_remaining() {
			bufs.push(buf);
		}
	}

	Ok(bufs)
}
