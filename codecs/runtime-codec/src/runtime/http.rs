use std::{borrow::Cow, rc::Rc, sync::Arc};

use super::error::{Error, Result};
use http::{request, response, HeaderMap, HeaderValue, Method, StatusCode, Uri, Version};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tea_sdk::serde::TypeId;

#[derive(Clone, Debug, TypeId, Serialize, Deserialize)]
pub struct HttpRequest {
	#[serde(with = "http_serde::method")]
	pub method: Method,

	#[serde(with = "http_serde::uri")]
	pub uri: Uri,

	#[serde(with = "http_serde::version")]
	pub version: Version,

	#[serde(with = "http_serde::header_map")]
	pub headers: HeaderMap<HeaderValue>,

	pub body: Vec<u8>,
}

impl<B> TryFrom<http::Request<B>> for HttpRequest
where
	B: IntoHttpBytes,
{
	type Error = Error;
	fn try_from(value: http::Request<B>) -> std::result::Result<Self, Self::Error> {
		let (
			request::Parts {
				method,
				uri,
				version,
				headers,
				..
			},
			body,
		) = value.into_parts();

		body.into_http_bytes().map(|body| Self {
			method,
			uri,
			version,
			headers,
			body,
		})
	}
}

impl<B> TryFrom<HttpRequest> for http::Request<B>
where
	B: FromHttpBytes,
{
	type Error = Error;
	fn try_from(value: HttpRequest) -> std::result::Result<Self, Self::Error> {
		http::Request::builder()
			.method(value.method)
			.uri(value.uri)
			.version(value.version)
			.body(B::from_http_bytes(value.body)?)
			.map(|mut r| {
				(*r.headers_mut()) = value.headers;
				r
			})
			.map_err(|e| Error::HttpError(e.to_string()))
	}
}

#[derive(Clone, Debug, TypeId, Serialize, Deserialize)]
pub struct HttpResponse {
	#[serde(with = "http_serde::status_code")]
	pub status: StatusCode,

	#[serde(with = "http_serde::version")]
	pub version: Version,

	#[serde(with = "http_serde::header_map")]
	pub headers: HeaderMap<HeaderValue>,

	pub body: Vec<u8>,
}

impl<B> TryFrom<http::Response<B>> for HttpResponse
where
	B: IntoHttpBytes,
{
	type Error = Error;
	fn try_from(value: http::Response<B>) -> std::result::Result<Self, Self::Error> {
		let (
			response::Parts {
				status,
				version,
				headers,
				..
			},
			body,
		) = value.into_parts();

		body.into_http_bytes().map(|body| Self {
			status,
			version,
			headers,
			body,
		})
	}
}

impl<B> TryFrom<HttpResponse> for http::Response<B>
where
	B: FromHttpBytes,
{
	type Error = Error;
	default fn try_from(value: HttpResponse) -> std::result::Result<Self, Self::Error> {
		http::Response::builder()
			.status(value.status)
			.version(value.version)
			.body(B::from_http_bytes(value.body)?)
			.map(|mut r| {
				(*r.headers_mut()) = value.headers;
				r
			})
			.map_err(|e| Error::HttpError(e.to_string()))
	}
}

pub trait IntoHttpBytes {
	fn into_http_bytes(self) -> std::result::Result<Vec<u8>, Error>;
}

pub trait FromHttpBytes: Sized {
	fn from_http_bytes(input: Vec<u8>) -> std::result::Result<Self, Error>;
}

struct NotBytesWrapper<T>(fn() -> T);
auto trait NotBytes {}

impl<T> IntoHttpBytes for T
where
	T: Serialize,
	NotBytesWrapper<T>: NotBytes,
{
	default fn into_http_bytes(self) -> Result<Vec<u8>> {
		Ok(serde_json::to_vec(&self).map_err(|e| Error::SerdeJsonError(e.to_string()))?)
	}
}
impl<T> FromHttpBytes for T
where
	T: DeserializeOwned,
	NotBytesWrapper<T>: NotBytes,
{
	default fn from_http_bytes(input: Vec<u8>) -> Result<Self> {
		serde_json::from_slice(&input).map_err(|e| Error::SerdeJsonError(e.to_string()))
	}
}

impl IntoHttpBytes for Vec<u8> {
	fn into_http_bytes(self) -> Result<Vec<u8>> {
		Ok(self)
	}
}

impl FromHttpBytes for Vec<u8> {
	fn from_http_bytes(input: Vec<u8>) -> Result<Self> {
		Ok(input)
	}
}

impl IntoHttpBytes for &[u8] {
	fn into_http_bytes(self) -> Result<Vec<u8>> {
		Ok(self.to_vec())
	}
}

impl IntoHttpBytes for Box<[u8]> {
	fn into_http_bytes(self) -> Result<Vec<u8>> {
		Ok(self.into_vec())
	}
}

impl FromHttpBytes for Box<[u8]> {
	fn from_http_bytes(input: Vec<u8>) -> Result<Self> {
		Ok(input.into_boxed_slice())
	}
}

impl IntoHttpBytes for Rc<[u8]> {
	fn into_http_bytes(self) -> Result<Vec<u8>> {
		Ok(self.to_vec())
	}
}

impl FromHttpBytes for Rc<[u8]> {
	fn from_http_bytes(input: Vec<u8>) -> Result<Self> {
		Ok(input.into())
	}
}

impl IntoHttpBytes for Arc<[u8]> {
	fn into_http_bytes(self) -> Result<Vec<u8>> {
		Ok(self.to_vec())
	}
}

impl FromHttpBytes for Arc<[u8]> {
	fn from_http_bytes(input: Vec<u8>) -> Result<Self> {
		Ok(input.into())
	}
}

impl IntoHttpBytes for Cow<'_, [u8]> {
	fn into_http_bytes(self) -> Result<Vec<u8>> {
		Ok(self.to_vec())
	}
}

impl FromHttpBytes for Cow<'_, [u8]> {
	fn from_http_bytes(input: Vec<u8>) -> Result<Self> {
		Ok(input.into())
	}
}

impl IntoHttpBytes for String {
	fn into_http_bytes(self) -> Result<Vec<u8>> {
		Ok(self.into_bytes())
	}
}

impl FromHttpBytes for String {
	fn from_http_bytes(input: Vec<u8>) -> Result<Self> {
		Ok(String::from_utf8(input).map_err(|e| Error::Utf8Error(e.to_string()))?)
	}
}

impl IntoHttpBytes for &str {
	fn into_http_bytes(self) -> Result<Vec<u8>> {
		Ok(self.to_string().into_bytes())
	}
}

impl IntoHttpBytes for Box<str> {
	fn into_http_bytes(self) -> Result<Vec<u8>> {
		Ok(self.into_string().into_bytes())
	}
}

impl FromHttpBytes for Box<str> {
	fn from_http_bytes(input: Vec<u8>) -> Result<Self> {
		Ok(String::from_utf8(input)
			.map_err(|e| Error::Utf8Error(e.to_string()))?
			.into_boxed_str())
	}
}

impl IntoHttpBytes for Rc<str> {
	fn into_http_bytes(self) -> Result<Vec<u8>> {
		Ok(self.to_string().into_bytes())
	}
}

impl FromHttpBytes for Rc<str> {
	fn from_http_bytes(input: Vec<u8>) -> Result<Self> {
		Ok(std::str::from_utf8(&input)
			.map_err(|e| Error::Utf8Error(e.to_string()))?
			.into())
	}
}

impl IntoHttpBytes for Arc<str> {
	fn into_http_bytes(self) -> Result<Vec<u8>> {
		Ok(self.to_string().into_bytes())
	}
}

impl FromHttpBytes for Arc<str> {
	fn from_http_bytes(input: Vec<u8>) -> Result<Self> {
		Ok(std::str::from_utf8(&input)
			.map_err(|e| Error::Utf8Error(e.to_string()))?
			.into())
	}
}

impl IntoHttpBytes for () {
	fn into_http_bytes(self) -> Result<Vec<u8>> {
		Ok(Vec::new())
	}
}

impl FromHttpBytes for () {
	fn from_http_bytes(_: Vec<u8>) -> Result<Self> {
		Ok(())
	}
}
