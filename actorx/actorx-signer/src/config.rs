use crate::{Claim, Metadata};
use primitive_types::H160;
use serde::{Deserialize, Serialize};

use crate::error::Result;

#[derive(Clone, Serialize, Deserialize)]
pub struct Manifest {
	actor_id: String,
	owner_id: String,
	token_id: H160,
	access: Vec<String>,
}

impl Manifest {
	pub fn into_metadata(self, priv_key: Vec<u8>) -> Result<Metadata> {
		Ok(Metadata {
			id: {
				let mut id = handle_base64(self.owner_id)?;
				id.push(b'.');
				id.extend(handle_base64(self.actor_id)?.into_iter());
				id
			},
			signer: priv_key,
			claims: self
				.access
				.into_iter()
				.map(handle_base64)
				.map(|x| x.map(Claim::ActorAccess))
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
