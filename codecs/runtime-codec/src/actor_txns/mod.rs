use crate::actor_txns::error::{Result, TxnError};
use crate::tapp::Hash;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::array::TryFromSliceError;
use std::convert::{TryFrom, TryInto};

pub mod auth;
pub mod context;
pub mod error;
mod followup;
pub mod pre_args;
pub mod tsid;
pub mod txn;

pub use followup::Followup;
pub use tsid::Tsid;

pub trait ToHash<H> {
	fn to_hash(&self) -> H;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TxnSerial {
	actor_name: Vec<u8>,
	bytes: Vec<u8>,
	nonce: u64,
	gas_limit: u64,
}

impl TxnSerial {
	pub fn new(actor_name: Vec<u8>, bytes: Vec<u8>, nonce: u64, _gas_limit: u64) -> Self {
		TxnSerial {
			actor_name,
			bytes,
			nonce,
			gas_limit: 1_000_000_u64,
		}
	}

	pub fn actor_name(&self) -> &[u8] {
		self.actor_name.as_slice()
	}

	pub fn bytes(&self) -> &[u8] {
		self.bytes.as_slice()
	}

	pub fn nonce(&self) -> u64 {
		self.nonce
	}

	pub fn gas_limit(&self) -> u64 {
		self.gas_limit
	}
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct QuerySerial {
	pub actor_name: Vec<u8>,
	pub bytes: Vec<u8>,
}

pub trait Transferable {
	/// for each txn derived some_txn, this function will return
	/// an unique String. Thie String is the actor_name.
	/// the actor will execute this txn and generate the context
	/// the context will then commit to the state so make a change
	fn get_handler_actor() -> Vec<u8>;
}

pub trait IntoSerial {
	type Error;
	fn into_serial(self, nonce: u64, gas_limit: u64) -> Result<TxnSerial, Self::Error>;
}

pub trait Txn<'a>:
	Transferable + TryFrom<TxnSerial> + IntoSerial + Serialize + Deserialize<'a> + std::fmt::Debug
{
}

pub trait Query<'a>:
	Transferable
	+ TryFrom<QuerySerial>
	+ TryInto<QuerySerial>
	+ Serialize
	+ Deserialize<'a>
	+ std::fmt::Debug
{
}

pub fn txn_hash(txn_bytes: &[u8]) -> Result<Hash> {
	let hash_g_array = Sha256::digest(txn_bytes);
	let hash_key: Hash = hash_g_array
		.as_slice()
		.try_into()
		.map_err(|e: TryFromSliceError| {
			TxnError::TxnHashError(e.to_string(), txn_bytes.to_vec())
		})?;
	Ok(hash_key)
}
