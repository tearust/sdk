use crate::error::Result;

pub fn encode_protobuf<T>(protobuf_type: T) -> Result<Vec<u8>>
where
	T: prost::Message,
{
	let mut buf: Vec<u8> = Vec::with_capacity(protobuf_type.encoded_len());
	protobuf_type.encode(&mut buf)?;
	Ok(buf)
}

pub mod libp2p;
pub mod structs_proto;
