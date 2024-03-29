use crate::actor_txns::{tsid::Hash, txn_hash, IntoSerial, ToHash, Txn, TxnSerial};
use crate::tapp::{ReplicaId, Ts};
use serde::{Deserialize, Serialize};
use std::vec::Vec;

#[doc(hidden)]
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Followup {
	pub ts: Ts,
	pub hash: crate::actor_txns::Hash,
	pub sender: ReplicaId,
}

impl Followup {
	pub fn test_now(txn: &TxnSerial, sender: ReplicaId, ts: Ts) -> Self {
		let buf: Vec<u8> = txn.hash_bytes().expect("failed to serialize");
		let hash_key: crate::actor_txns::Hash =
			txn_hash(buf.as_slice()).expect("wrong length hash");
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
			.into_serial(0, 0, 10000)
			.expect("convert txn serial failed");
		let hash_key =
			txn_hash(&serial.hash_bytes().expect("serialize failed")).expect("wrong length hash");
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
