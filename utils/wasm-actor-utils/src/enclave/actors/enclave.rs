use crate::enclave::error::{Errors, Result};
use tea_actorx::ActorId;
use tea_sdk::ResultExt;
use tea_system_actors::nitro::*;

pub async fn get_my_tea_id() -> Result<Vec<u8>> {
	let res_vec = ActorId::Static(NAME).call(GetTeaIdRequest).await?;
	if res_vec.0.is_empty() {
		Err(Errors::UninitializedTeaId.into())
	} else {
		Ok(res_vec.0)
	}
}

pub async fn get_my_ephemeral_id() -> Result<Vec<u8>> {
	let res_vec = ActorId::Static(NAME).call(EphemeralPubkeyRequest).await?;
	if res_vec.0.is_empty() {
		Err(Errors::UninitializedEphemeralPublicKey.into())
	} else {
		Ok(res_vec.0)
	}
}

pub async fn get_my_ephemeral_key() -> Result<Vec<u8>> {
	let res_vec = ActorId::Static(NAME).call(EphemeralKeyRequest).await?;
	if res_vec.0.is_empty() {
		Err(Errors::UninitializedEphemeralKey.into())
	} else {
		Ok(res_vec.0)
	}
}

pub async fn generate_uuid() -> Result<String> {
	let uuid = ActorId::Static(NAME).call(GenerateUuidRequest).await?;
	Ok(uuid.0)
}

pub async fn random_u64() -> Result<u64> {
	const U64_SIZE: usize = 8;
	let mut u64_buf = [0u8; U64_SIZE];
	let rand_buf = generate_random(U64_SIZE as u32).await?;
	u64_buf.copy_from_slice(&rand_buf[0..U64_SIZE]);
	Ok(u64::from_le_bytes(u64_buf))
}

pub async fn generate_random(len: u32) -> Result<Vec<u8>> {
	let res = ActorId::Static(NAME)
		.call(GenerateRandomRequest(len))
		.await?;
	Ok(res)
}

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

pub async fn nitro_encrypt(tag: &str, data: Vec<u8>) -> Result<Vec<u8>> {
	ActorId::Static(NAME)
		.call(NitroEncryptRequest {
			tag: tag.to_string(),
			data,
		})
		.await
		.err_into()
}

pub async fn nitro_decrypt(tag: &str, encrypted_data: Vec<u8>) -> Result<Vec<u8>> {
	ActorId::Static(NAME)
		.call(NitroDecryptRequest {
			tag: tag.to_string(),
			cipher_data: encrypted_data,
		})
		.await
		.err_into()
}
