use http_actor_codec::error::HttpActor;
use tea_codec::define_scope;
use thiserror::Error;

use crate::error::Actor;

define_scope! {
	Http: pub Actor, HttpActor {
		httparse::Error => @HttpActor::BadRequestFormat, @Display, @Debug;
		http::Error => Http, @Display, @Debug;
		http::uri::InvalidUri => @HttpActor::BadRequestFormat, @Display, @Debug;
		http::header::InvalidHeaderName => @HttpActor::BadResponseFormat, @Display, @Debug;
		http::header::InvalidHeaderValue => @HttpActor::BadResponseFormat, @Display, @Debug;
		http::status::InvalidStatusCode => @HttpActor::BadResponseFormat, @Display, @Debug;
	}
}

#[derive(Debug, Error)]
#[error("Http responded an error code {0}")]
pub struct HttpErrorStatus(pub http::StatusCode);
