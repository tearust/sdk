use crate::enclave::error::{Error, Result};
use prost::Message;
use tea_actorx::ActorId;
use tea_runtime_codec::tapp::Account;
use tea_runtime_codec::vmh::message::{encode_protobuf, structs_proto::crypto};
use tea_sdk::IntoGlobal;
use tea_system_actors::crypto::*;

/// Base sha-256.
pub async fn sha256(content: Vec<u8>) -> Result<Vec<u8>> {
	let req = crypto::ShaRequest {
		sha_type: "sha256".to_string(),
		content,
	};
	let r = ActorId::Static(NAME)
		.call(Sha256Request(encode_protobuf(req)?))
		.await?;
	let res = crypto::ShaResponse::decode(r.0.as_slice()).into_g::<Error>()?;
	Ok(res.hash)
}

/// Verify signature with ether.rs
pub async fn ether_verify(account: Account, data: String, signature_hex: String) -> Result<bool> {
	let res = ActorId::Static(NAME)
		.call(EtherVerifyRequest {
			data,
			account,
			signature: signature_hex,
		})
		.await?;
	Ok(res.0)
}

pub async fn aes_encrypt(key: Vec<u8>, data: Vec<u8>) -> Result<Vec<u8>> {
	let res = ActorId::Static(NAME)
		.call(AesEncryptRequest(encode_protobuf(
			crypto::AesEncryptRequest { key, data },
		)?))
		.await?;
	let res_data = crypto::AesEncryptResponse::decode(res.0.as_slice()).into_g::<Error>()?;
	Ok(res_data.encrypted_data)
}

pub async fn aes_decrypt(key: Vec<u8>, encrypted_data: Vec<u8>) -> Result<Vec<u8>> {
	let res = ActorId::Static(NAME)
		.call(AesDecryptRequest(encode_protobuf(
			crypto::AesDecryptRequest {
				key,
				encrypted_data,
			},
		)?))
		.await?;
	let res_data = crypto::AesDecryptResponse::decode(res.0.as_slice()).into_g::<Error>()?;
	Ok(res_data.data)
}
