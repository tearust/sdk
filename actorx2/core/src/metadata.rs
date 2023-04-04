use primitive_types::H160;
use serde::{Deserialize, Serialize};

use crate::actor::ActorId;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Metadata {
	pub id: ActorId,
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
	ActorAccess(ActorId),
	TokenId(H160),
}
