use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tea_codec::pricing::Priced;
use tea_codec::serde::TypeId;
use tea_runtime_codec::tapp::ra::PcrType;

pub const NAME: &[u8] = b"tea:nitro";

#[doc(hidden)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AttestationDocRequest {
	pub user_data: Option<Vec<u8>>,
	pub nonce: Option<Vec<u8>>,
	pub pubkey: Option<Vec<u8>>,
}

#[doc(hidden)]
pub type AttestationDocResponse = Vec<u8>;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PcrVerifyRequest {
	pub doc_request: AttestationDocRequest,
	pub doc_buf: AttestationDocResponse,
	pub pcr_slots: PcrVerifySlots,
	pub allow_dummy: bool,
}

#[doc(hidden)]
pub type PcrVerifySlots = Vec<HashMap<PcrType, Vec<u8>>>;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RaPeerRequest {
	pub seq_number: u64,
	pub conn_id: String,
	pub doc_request: AttestationDocRequest,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetTeaIdRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetTeaIdResponse(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct EphemeralPubkeyRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct EphemeralPubkeyResponse(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct EphemeralKeyRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct EphemeralKeyResponse(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(Vec<u8>)]
pub struct GenerateRandomRequest(pub u32);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GenerateUuidRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GenerateUuidResponse(pub String);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetAttestationDocRequest(pub AttestationDocRequest);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetAttestationDocResponse(pub AttestationDocResponse);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct VerifyAttestationDocRequest(pub PcrVerifyRequest);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetVerificationPcrsRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetVerificationPcrsResponse(pub PcrVerifySlots);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(Vec<u8>)]
pub struct NitroEncryptRequest {
	pub tag: String,
	pub data: Vec<u8>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(Vec<u8>)]
pub struct NitroDecryptRequest {
	pub tag: String,
	pub cipher_data: Vec<u8>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GenerateDataKeyRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GenerateDataKeyResponse {
	pub secret: Vec<u8>,
	pub ciphertext: String,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(Vec<u8>)]
pub struct DecryptDataKeyRequest {
	pub ciphertext: String,
}
