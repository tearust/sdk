pub mod error;

pub const NAME: &[u8] = b"tea:http";

pub use tea_runtime_codec::http::{HttpRequest, HttpResponse};
