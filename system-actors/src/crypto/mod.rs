use serde::{Deserialize, Serialize};
use tea_codec::pricing::Priced;
use tea_codec::serde::TypeId;
use tea_runtime_codec::tapp::Account;

pub mod error;

pub const KEY_TYPE_BITCOIN_MAINNET: &str = "bitcoin_mainnet";
pub const KEY_TYPE_BITCOIN_TESTNET: &str = "bitcoin_testnet";
pub const KEY_TYPE_ED25519: &str = "ed25519";

pub const NAME: &[u8] = b"tea:crypto";

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GenerateKeyPairRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GenerateKeyPairResponse(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct PublicKeyFromPrivateKeyRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct PublicKeyFromPrivateKeyResponse(pub Vec<u8>);

/// Base request for sha-256.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct Sha256Request(pub Vec<u8>);

/// Base response for sha-256.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct Sha256Response(pub Vec<u8>);

/// Base sign request in bytes.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct SignRequest(pub Vec<u8>);

/// Base sign response in bytes.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct SignResponse(pub Vec<u8>);

/// Base request to verify signature.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct VerifyRequest(pub Vec<u8>);

/// Base response for verify signature.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct VerifyResponse(pub Vec<u8>);

/// Baes request to verify signature with ether-rs.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct EtherVerifyRequest {
	pub data: String,
	pub signature: String,
	pub account: Account,
}

/// Base response for verify signature with ether-rs.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct EtherVerifyResponse(pub bool);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct ShamirShareRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ShamirShareResponse(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct ShamirRecoveryRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ShamirRecoveryResponse(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GenerateMultisigAccountRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GenerateMultisigAccountResponse(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct CombineToWitnessRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct CombineToWitnessResponse(pub Vec<u8>);

/// Base request to generate Aes key.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GenerateAesKeyRequest;

/// Base response for generating Aes key.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GenerateAesKeyResponse(pub Vec<u8>);

/// Base request for Aes encrypt.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct AesEncryptRequest(pub Vec<u8>);

/// Base response for Aes encrypt.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct AesEncryptResponse(pub Vec<u8>);

/// Base request for Aes decrypt.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct AesDecryptRequest(pub Vec<u8>);

/// Base response for Aes decrypt.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct AesDecryptResponse(pub Vec<u8>);

/// Base request to generate rsa key-pair.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GenerateRsaPemPcsk1KeypairRequest(pub Vec<u8>);

/// Base response to generate rsa key-pair.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GenerateRsaPemPcsk1KeypairResponse(pub Vec<u8>);

/// Base request to encrypt with RSA.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct RsaEncryptRequest(pub Vec<u8>);

/// Base response to encrypting with RSA.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct RsaEncryptResponse(pub Vec<u8>);

/// Base request to descrypt with RSA.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct RsaDecryptRequest(pub Vec<u8>);

/// Base response for decrypting with RSA.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct RsaDecryptResponse(pub Vec<u8>);
