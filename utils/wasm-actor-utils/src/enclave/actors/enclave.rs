use super::crypto::{aes_decrypt, aes_encrypt};
use crate::enclave::error::{Errors, Result};
#[cfg(feature = "__test")]
use mocktopus::macros::mockable;
use tea_actorx::ActorId;
use tea_sdk::ResultExt;
use tea_system_actors::nitro::*;

/// Return current node's tea_id
pub async fn get_my_tea_id() -> Result<Vec<u8>> {
	let res_vec = ActorId::Static(NAME).call(GetTeaIdRequest).await?;
	if res_vec.0.is_empty() {
		Err(Errors::UninitializedTeaId.into())
	} else {
		Ok(res_vec.0)
	}
}

#[doc(hidden)]
pub async fn get_my_ephemeral_id() -> Result<Vec<u8>> {
	let res_vec = ActorId::Static(NAME).call(EphemeralPubkeyRequest).await?;
	if res_vec.0.is_empty() {
		Err(Errors::UninitializedEphemeralPublicKey.into())
	} else {
		Ok(res_vec.0)
	}
}

#[doc(hidden)]
pub async fn get_my_ephemeral_key() -> Result<Vec<u8>> {
	let res_vec = ActorId::Static(NAME).call(EphemeralKeyRequest).await?;
	if res_vec.0.is_empty() {
		Err(Errors::UninitializedEphemeralKey.into())
	} else {
		Ok(res_vec.0)
	}
}

/// Return a random uuid
pub async fn generate_uuid() -> Result<String> {
	let uuid = ActorId::Static(NAME).call(GenerateUuidRequest).await?;
	Ok(uuid.0)
}

/// Return a random u64
pub async fn random_u64() -> Result<u64> {
	const U64_SIZE: usize = 8;
	let mut u64_buf = [0u8; U64_SIZE];
	let rand_buf = generate_random(U64_SIZE as u32).await?;
	u64_buf.copy_from_slice(&rand_buf[0..U64_SIZE]);
	Ok(u64::from_le_bytes(u64_buf))
}

/// Return a fix-length random u8 array.
pub async fn generate_random(len: u32) -> Result<Vec<u8>> {
	let res = ActorId::Static(NAME)
		.call(GenerateRandomRequest(len))
		.await?;
	Ok(res)
}

#[doc(hidden)]
#[cfg(not(feature = "__test"))]
pub async fn verify_peer(
	doc_request: AttestationDocRequest,
	conn_id: &str,
	is_seat: bool,
) -> Result<bool> {
	use tea_system_actors::ra::*;

	let res = ActorId::Static(NAME)
		.call(VerifyPeerRequest {
			data: RaPeerRequest {
				seq_number: 0,
				conn_id: conn_id.to_string(),
				doc_request,
			},
			is_seat,
		})
		.await?;
	Ok(res.0)
}

#[cfg(feature = "__test")]
#[mockable]
pub async fn verify_peer(
	_doc_request: AttestationDocRequest,
	_conn_id: &str,
	_is_seat: bool,
) -> Result<bool> {
	Ok(true)
}

#[doc(hidden)]
pub async fn nitro_encrypt(tag: &str, data: Vec<u8>) -> Result<Vec<u8>> {
	ActorId::Static(NAME)
		.call(NitroEncryptRequest {
			tag: tag.to_string(),
			data,
		})
		.await
		.err_into()
}

#[doc(hidden)]
pub async fn nitro_decrypt(tag: &str, encrypted_data: Vec<u8>) -> Result<Vec<u8>> {
	ActorId::Static(NAME)
		.call(NitroDecryptRequest {
			tag: tag.to_string(),
			cipher_data: encrypted_data,
		})
		.await
		.err_into()
}

pub async fn nitro_temp_encrypt(data: Vec<u8>) -> Result<(Vec<u8>, String)> {
	let data_key_res = ActorId::Static(NAME).call(GenerateDataKeyRequest).await?;
	let ciphertext = data_key_res.ciphertext;
	Ok((aes_encrypt(data_key_res.secret, data).await?, ciphertext))
}

pub async fn nitro_temp_decrypt(ciphertext: String, encrypted_data: Vec<u8>) -> Result<Vec<u8>> {
	let key = ActorId::Static(NAME)
		.call(DecryptDataKeyRequest { ciphertext })
		.await?;
	aes_decrypt(key, encrypted_data).await
}
