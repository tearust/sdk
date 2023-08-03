use crate::actor_txns::{tsid::Hash, txn_hash, IntoSerial, ToHash, Txn, TxnSerial};
use crate::tapp::{ReplicaId, Ts};
use serde::{Deserialize, Serialize};
use std::vec::Vec;
use tea_sdk::serialize;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Followup {
	pub ts: Ts,
	pub hash: crate::actor_txns::Hash,
	pub sender: ReplicaId,
	pub nonce: u64,
}

impl Followup {
	pub fn test_now<'a, T>(txn: &T, sender: ReplicaId, ts: Ts) -> Self
	where
		T: Txn<'a> + Clone,
	{
		let buf: Vec<u8> = serialize(txn).expect("failed to serialize");
		let hash_key: crate::actor_txns::Hash =
			txn_hash(buf.as_slice()).expect("wrong length hash");
		Followup {
			ts,
			hash: hash_key,
			sender,
			nonce: 0_64,
		}
	}

	pub fn test_spec_time<'a, T>(ts: Ts, txn: &T, sender: ReplicaId) -> Self
	where
		T: Txn<'a> + Clone,
		<T as IntoSerial>::Error: std::fmt::Debug,
	{
		let serial: TxnSerial = txn
			.clone()
			.into_serial(0, 10000)
			.expect("convert txn serial failed");
		let hash_key =
			txn_hash(&serialize(&serial).expect("serialize failed")).expect("wrong length hash");
		Followup {
			ts,
			hash: hash_key,
			sender,
			nonce: 0_u64,
		}
	}
}

impl ToHash<Hash> for Followup {
	fn to_hash(&self) -> Hash {
		self.hash
	}
}
