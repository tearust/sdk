use std::fs;
use std::path::Path;

use error::{InvalidSignatureFormat, Result, SignatureMismatch};
use openssl::sign::Signer;
use openssl::{hash::MessageDigest, pkey::PKey, sign::Verifier};
use primitive_types::H160;
use serde::{Deserialize, Serialize};

use crate::metadata::Claim;

use super::metadata::Metadata;
pub mod error;

extern crate tea_codec as tea_sdk;

const WASM_HEAD_LENGTH: usize = 8;

const CURRENT_VERSION: u32 = 1;

const SECTION_NAME: &[u8] = b"\tSignature";

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

#[derive(Clone, Serialize, Deserialize)]
pub struct Manifest {
	pub actor_id: String,
	pub owner_id: String,
	pub token_id: H160,
	pub access: Vec<String>,
}

impl Manifest {
	pub fn into_metadata(self, priv_key: Vec<u8>) -> Result<Metadata> {
		Ok(Metadata {
			id: {
				let mut id = handle_base64(self.owner_id)?;
				id.push(b'.');
				id.extend(handle_base64(self.actor_id)?.into_iter());
				id.into()
			},
			signer: priv_key,
			claims: self
				.access
				.into_iter()
				.map(handle_base64)
				.map(|x| x.map(Into::into).map(Claim::ActorAccess))
				.chain(Some(Ok(Claim::TokenId(self.token_id))))
				.try_collect()?,
		})
	}
}

fn handle_base64(input: String) -> Result<Vec<u8>> {
	Ok(if let [b'#', input @ ..] = input.as_bytes() {
		base64::decode(input)?
	} else {
		input.into_bytes()
	})
}

pub fn sign_file(
	wasm: impl AsRef<Path>,
	manifest: impl AsRef<Path>,
	priv_key: impl AsRef<Path>,
) -> Result<()> {
	let mut wasm_file = fs::read(wasm.as_ref())?;
	if verify(&wasm_file).is_ok() {
		return Ok(());
	}
	let manifest = fs::File::open(manifest)?;
	let priv_key = fs::read(priv_key)?;
	let manifest: Manifest = serde_yaml::from_reader(manifest)?;
	let metadata = manifest.into_metadata(priv_key)?;
	sign(&mut wasm_file, metadata)?;
	fs::write(wasm, wasm_file)?;
	Ok(())
}
