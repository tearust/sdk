use std::{fs, path::Path};

use primitive_types::H160;
use serde::{Deserialize, Serialize};
use tea_actorx2_core::{
	metadata::{Claim, Metadata},
	sign::{sign, verify},
};

use crate::error::Result;

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
