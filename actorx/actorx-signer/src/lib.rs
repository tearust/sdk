#![feature(min_specialization)]

use error::{InvalidSignatureFormat, Result, SignatureMismatch};
use openssl::{
	hash::MessageDigest,
	pkey::PKey,
	sign::{Signer, Verifier},
};
use primitive_types::H160;
use serde::{Deserialize, Serialize};
pub mod error;

const WASM_HEAD_LENGTH: usize = 8;

const CURRENT_VERSION: u32 = 1;

const SECTION_NAME: &[u8] = b"\tSignature";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Metadata {
	pub id: Vec<u8>,
	pub signer: Vec<u8>,
	pub claims: Vec<Claim>,
}

impl Metadata {
	pub fn get_token_id(&self) -> Option<H160> {
		self.claims.iter().find_map(|x| {
			if let Claim::TokenId(id) = x {
				Some(*id)
			} else {
				None
			}
		})
	}
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Claim {
	ActorAccess(Vec<u8>),
	TokenId(H160),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Metatoken {
	version: u32,
	payload: Vec<u8>,
	signature: Vec<u8>,
}

pub fn sign(wasm: &mut Vec<u8>, mut data: Metadata) -> Result<()> {
	let key = PKey::private_key_from_pem(&data.signer)
		.or_else(|_| PKey::private_key_from_der(&data.signer))?;
	data.signer = key.public_key_to_der()?;

	let payload = tea_codec::serialize(&data)?;

	let mut signer = Signer::new(MessageDigest::sha256(), &key)?;
	signer.update(&wasm[0..WASM_HEAD_LENGTH])?;
	signer.update(&wasm[WASM_HEAD_LENGTH..])?;
	signer.update(&payload)?;

	let signature = signer.sign_to_vec()?;

	let token = Metatoken {
		version: CURRENT_VERSION,
		payload,
		signature,
	};

	let token = tea_codec::serialize(&token)?;

	let mut token = &token[..];

	let token = zstd::encode_all(&mut token, zstd::zstd_safe::max_c_level())?;

	let mut data_len = [0; 5];
	let count = leb128::write::unsigned(
		&mut &mut data_len[..],
		(token.len() + SECTION_NAME.len()) as _,
	)?;
	let data_len = data_len.into_iter().take(count);

	let token = Some(0)
		.into_iter()
		.chain(data_len)
		.chain(SECTION_NAME.iter().cloned())
		.chain(token);

	wasm.splice(WASM_HEAD_LENGTH..WASM_HEAD_LENGTH, token);

	Ok(())
}

pub fn verify(wasm: &[u8]) -> Result<Metadata> {
	let (head, mut token) = wasm.split_at(WASM_HEAD_LENGTH);
	token = &token[1..];
	let data_len = leb128::read::unsigned(&mut token)?;

	let (token, rest) = token.split_at(data_len as _);

	if !token.starts_with(SECTION_NAME) {
		return Err(InvalidSignatureFormat.into());
	}

	let token = &token[SECTION_NAME.len()..];

	let token = zstd::decode_all(token)?;

	let token: Metatoken = tea_codec::deserialize(token)?;

	let payload: Metadata = tea_codec::deserialize(&token.payload)?;

	let key = PKey::public_key_from_der(&payload.signer)?;

	let mut signer = Verifier::new(MessageDigest::sha256(), &key)?;
	signer.update(head)?;
	signer.update(rest)?;
	signer.update(&token.payload)?;

	if signer.verify(&token.signature)? {
		Ok(payload)
	} else {
		Err(SignatureMismatch.into())
	}
}
