use crate::error::Result;
use prost::Message;
use tea_actorx_core::RegId;
use tea_actorx_runtime::call;
use tea_system_actors::crypto::*;
use tea_tapp_common::Account;
use tea_vmh_codec::message::{encode_protobuf, structs_proto::crypto};

pub async fn sha256(content: Vec<u8>) -> Result<Vec<u8>> {
	let req = crypto::ShaRequest {
		sha_type: "sha256".to_string(),
		content,
	};
	let r = call(
		RegId::Static(NAME).inst(0),
		Sha256Request(encode_protobuf(req)?),
	)
	.await?;
	let res = crypto::ShaResponse::decode(r.0.as_slice())?;
	Ok(res.hash)
}

pub async fn ether_verify(account: Account, data: String, signature_hex: String) -> Result<bool> {
	let res = call(
		RegId::Static(NAME).inst(0),
		EtherVerifyRequest {
			data,
			account,
			signature: signature_hex,
		},
	)
	.await?;
	Ok(res.0)
}
