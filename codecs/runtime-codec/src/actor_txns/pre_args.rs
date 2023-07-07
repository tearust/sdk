use crate::actor_txns::error::{Result, TxnError};
use crate::tapp::{Hash, ReplicaId, TokenId};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{array::TryFromSliceError, convert::TryInto};
use tea_sdk::serialize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Arg {
	pub ty: Type,
	pub filter: Filter,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Type {
	Cml(String),
	TappstoreOwner(String),
	CurrentHeight(String),
	TopupLogs(String),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Filter {
	Single(Indentity),
	Multiple(Vec<Indentity>),
	ByStatus(Status),
	Uncountable,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Indentity {
	U64(u64),
	Hash(Hash),
	TeaId(ReplicaId),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Status {
	Active,
	Mining,
	Hosting(TokenId, bool), // (TokenId, Is active only)
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArgSlots {
	pub args: Vec<ArgResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ArgResult {
	pub arg: Arg,
	pub result: Vec<u8>,
}

impl ArgSlots {
	pub fn hash(&self) -> Result<Hash> {
		// Note here use serialize to calculate, that means all args related fields must be
		//  order-deterministic (HashSet or HashMap related type should not be used)
		let txn_bytes = serialize(self)?;

		let hash_g_array = Sha256::digest(txn_bytes);
		let hash_key: Hash = hash_g_array
			.as_slice()
			.try_into()
			.map_err(|e: TryFromSliceError| TxnError::PreArgsHashError(e.to_string()))?;
		Ok(hash_key)
	}

	pub fn size(&self) -> usize {
		self.args.iter().fold(0, |acc, x| acc + x.size())
	}
}

impl ArgResult {
	pub fn size(&self) -> usize {
		self.result.len() + self.arg.size()
	}
}

impl Arg {
	pub fn single_cml(key: String, cml_id: u64) -> Self {
		Arg {
			ty: Type::Cml(key),
			filter: Filter::Single(Indentity::U64(cml_id)),
		}
	}

	pub fn single_status(key: String, status: Status) -> Self {
		Arg {
			ty: Type::Cml(key),
			filter: Filter::ByStatus(status),
		}
	}

	pub fn single_tea_id(key: String, tea_id: ReplicaId) -> Self {
		Arg {
			ty: Type::Cml(key),
			filter: Filter::Single(Indentity::TeaId(tea_id)),
		}
	}

	pub fn multi_cmls(key: String, cml_ids: &[u64]) -> Self {
		let cml_ids: Vec<Indentity> = cml_ids.iter().map(|id| Indentity::U64(*id)).collect();
		Arg {
			ty: Type::Cml(key),
			filter: Filter::Multiple(cml_ids),
		}
	}

	pub fn tappstore_owner(key: String) -> Self {
		Arg {
			ty: Type::TappstoreOwner(key),
			filter: Filter::Uncountable,
		}
	}

	pub fn current_height(key: String) -> Self {
		Arg {
			ty: Type::CurrentHeight(key),
			filter: Filter::Uncountable,
		}
	}

	pub fn topup_logs(key: String) -> Self {
		Arg {
			ty: Type::TopupLogs(key),
			filter: Filter::Uncountable,
		}
	}

	pub fn size(&self) -> usize {
		self.filter.size() + 1
	}
}

impl Filter {
	pub fn size(&self) -> usize {
		1 + match self {
			Filter::Single(s) => s.size(),
			Filter::Multiple(s) => s.iter().fold(0, |acc, x| acc + x.size()),
			Filter::ByStatus(_) => 1,
			Filter::Uncountable => 0,
		}
	}
}

impl Indentity {
	pub fn size(&self) -> usize {
		1 + match self {
			Indentity::Hash(_) => 32,
			Indentity::U64(_) => 8,
			Indentity::TeaId(_) => 32,
		}
	}
}
