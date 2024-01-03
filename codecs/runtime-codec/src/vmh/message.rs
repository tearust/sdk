use tea_sdk::IntoGlobal;

use crate::vmh::error::{Error, Result};

pub fn encode_protobuf<T>(protobuf_type: T) -> Result<Vec<u8>>
where
	T: prost::Message,
{
	let mut buf: Vec<u8> = Vec::with_capacity(protobuf_type.encoded_len());
	protobuf_type.encode(&mut buf).into_g::<Error>()?;
	Ok(buf)
}

pub mod libp2p;
pub mod structs_proto;
