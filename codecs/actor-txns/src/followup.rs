use crate::{tsid::Hash, txn_hash, IntoSerial, ToHash, Txn, TxnSerial};
use serde::{Deserialize, Serialize};
use std::vec::Vec;
use tapp_common::{ReplicaId, Ts};
use tea_sdk::serialize;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Followup {
	pub ts: Ts,
	pub hash: crate::Hash,
	pub sender: ReplicaId,
}

impl Followup {
	pub fn test_now<'a, T>(txn: &T, sender: ReplicaId, ts: Ts) -> Self
	where
		T: Txn<'a> + Clone,
	{
		let buf: Vec<u8> = serialize(txn).expect("failed to serialize");
		let hash_key: crate::Hash = txn_hash(buf.as_slice()).expect("wrong length hash");
		Followup {
			ts,
			hash: hash_key,
			sender,
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
		}
	}
}

impl ToHash<Hash> for Followup {
	fn to_hash(&self) -> Hash {
		self.hash
	}
}
