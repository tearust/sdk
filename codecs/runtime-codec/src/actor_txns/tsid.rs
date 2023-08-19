use crate::actor_txns::{Followup, ToHash};
use crate::tapp::{ReplicaId, Ts};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash as StdHash, Hasher};

pub type Hash = [u8; 32];

#[doc(hidden)]
#[derive(Default, Eq, Copy, Clone, Serialize, Deserialize)]
pub struct Tsid {
	pub ts: Ts,
	pub hash: Hash, //hash of the txn
	pub sender: ReplicaId,
	seed: Hash,
	/// hash of all txn pre-args and related values
	args_hash: Option<Hash>,

	pub nonce: Option<u64>,
}

impl Tsid {
	pub fn byte_size() -> usize {
		16 + 32 + 32 + 32
	}

	pub fn from_followup(seed: Hash, args_hash: Option<Hash>, followup: &Followup) -> Self {
		Tsid {
			ts: followup.ts,
			sender: followup.sender,
			hash: followup.hash,
			seed,
			args_hash,
			nonce: None,
		}
	}
	pub fn genesis() -> Self {
		Self {
			ts: 0,
			sender: [0u8; 32],
			hash: [0u8; 32],
			seed: [0u8; 32],
			args_hash: Default::default(),
			nonce: None,
		}
	}
	pub fn get_seed(&self) -> Hash {
		self.seed
	}

	pub fn clone_with_nonce(&self, nonce: u64) -> Tsid {
		let mut new_ob = self.clone();
		new_ob.nonce = Some(nonce);
		new_ob
	}

	pub fn args_hash(&self) -> Option<&Hash> {
		self.args_hash.as_ref()
	}

	pub fn same_hash(&self, other_hash: &Hash) -> bool {
		for (i, v) in self.hash.iter().enumerate() {
			println!("i {} bytes {}, {}", &i, v, &other_hash[i]);
			if *v != other_hash[i] {
				return false;
			}
		}
		true
	}

	pub fn raw(&self) -> Vec<u8> {
		let mut buf = self.ts.to_le_bytes().to_vec();
		buf.extend(self.hash);
		buf.extend(self.sender);
		buf.extend(self.seed);
		if let Some(hash) = self.args_hash {
			buf.extend(hash);
		}
		buf
	}

	pub fn reover_from_readable(&mut self, tsid: TsidReadable) -> bool {
		self.ts = tsid.ts;
		if let Ok(sender) = hex::decode(tsid.sender) {
			if sender.len() == 32 {
				let new_sender = TryInto::<[u8; 32]>::try_into(sender.as_slice());
				if new_sender.is_ok() {
					self.sender = new_sender.unwrap();
				}
			}
		}
		if let Ok(hash) = hex::decode(tsid.hash) {
			if hash.len() == 32 {
				let new_hash = TryInto::<[u8; 32]>::try_into(hash.as_slice());
				if new_hash.is_ok() {
					self.hash = new_hash.unwrap();
				}
			}
		}
		if let Ok(seed) = hex::decode(tsid.seed) {
			if seed.len() == 32 {
				let new_seed = TryInto::<[u8; 32]>::try_into(seed.as_slice());
				if new_seed.is_ok() {
					self.seed = new_seed.unwrap();
				}
			}
		}

		true
	}
}

impl PartialOrd for Tsid {
	#[inline]
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		match self.ts.partial_cmp(&other.ts) {
			Some(Ordering::Equal) => {}
			ord => return ord,
		}
		match self.hash.partial_cmp(&other.hash) {
			Some(Ordering::Equal) => {}
			ord => return ord,
		}
		match self.seed.partial_cmp(&other.seed) {
			Some(Ordering::Equal) => {}
			ord => return ord,
		}
		self.sender.partial_cmp(&other.sender)
	}
}

impl Ord for Tsid {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		let mut ord = self.ts.cmp(&other.ts);
		if ord != Ordering::Equal {
			return ord;
		}

		ord = self.hash.cmp(&other.hash);
		if ord != Ordering::Equal {
			return ord;
		}

		ord = self.sender.cmp(&other.sender);
		if ord != Ordering::Equal {
			return ord;
		}

		self.seed.cmp(&other.seed)
	}
}

impl PartialEq for Tsid {
	fn eq(&self, other: &Self) -> bool {
		self.ts == other.ts
			&& self.hash == other.hash
			&& self.sender == other.sender
			&& self.seed == other.seed
	}
}

impl StdHash for Tsid {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.ts.hash(state);
		self.hash.hash(state);
		self.sender.hash(state);
		self.seed.hash(state);
	}
}

impl ToHash<Hash> for Tsid {
	fn to_hash(&self) -> Hash {
		self.hash
	}
}

impl fmt::Debug for Tsid {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Tsid")
			.field("ts", &self.ts)
			.field("sender(hex)", &hex::encode(self.sender))
			.field("hash(hex)", &hex::encode(self.hash))
			.field("seed(hex)", &hex::encode(self.seed))
			.field("arg_hash(hex)", &self.args_hash.map(hex::encode))
			.field(
				"nonce",
				&match self.nonce {
					Some(v) => v.to_string(),
					None => "".to_string(),
				},
			)
			.finish()
	}
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TsidReadable {
	pub ts: u128,
	pub sender: String,
	pub hash: String,
	pub seed: String,
}

impl From<Tsid> for TsidReadable {
	fn from(tsid: Tsid) -> Self {
		TsidReadable {
			ts: tsid.ts,
			sender: hex::encode(tsid.sender),
			hash: hex::encode(tsid.hash),
			seed: hex::encode(tsid.get_seed()),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::TsidReadable;
	use crate::actor_txns::tsid::{Hash, Tsid};
	use std::collections::hash_map::DefaultHasher;
	use std::hash::{Hash as StdHash, Hasher};

	#[test]
	fn tsid_equal_works() {
		assert_eq!(Tsid::genesis(), Tsid::genesis());

		let id1 = Tsid {
			ts: 1111,
			sender: [2; 32],
			hash: [3; 32],
			seed: [4; 32],
			args_hash: Some([5; 32]),
			nonce: None,
		};

		let mut id2 = id1;
		assert_eq!(id1, id2);

		id2.ts = 5555;
		assert_ne!(id1, id2);

		id2 = id1;
		id2.sender = [5; 32];
		assert_ne!(id1, id2);

		id2 = id1;
		id2.hash = [5; 32];
		assert_ne!(id1, id2);

		id2 = id1;
		id2.seed = [5; 32];
		assert_ne!(id1, id2);

		id2 = id1;
		id2.args_hash = Some([6; 32]);
		assert_eq!(id1, id2);
	}

	#[test]
	fn tsid_compare_works() {
		let id1 = Tsid {
			ts: 1111,
			sender: [2; 32],
			hash: [3; 32],
			seed: [4; 32],
			args_hash: Some([5; 32]),
			nonce: None,
		};

		let mut id2 = id1;
		assert_eq!(id1, id2);

		id2.ts = id1.ts + 1;
		assert!(id1 < id2);
		id2.hash = min_hash();
		assert!(id1 < id2);
		id2.sender = min_hash();
		assert!(id1 < id2);
		id2.seed = min_hash();
		assert!(id1 < id2);

		id2 = id1;
		id2.ts = id1.ts - 1;
		assert!(id1 > id2);
		id2.hash = max_hash();
		assert!(id1 > id2);
		id2.sender = max_hash();
		assert!(id1 > id2);
		id2.seed = max_hash();
		assert!(id1 > id2);

		id2 = id1;
		id2.hash[31] += 1;
		assert!(id1 < id2);
		id2.sender = min_hash();
		assert!(id1 < id2);
		id2.seed = min_hash();
		assert!(id1 < id2);

		id2 = id1;
		id2.hash[31] -= 1;
		assert!(id1 > id2);
		id2.sender = max_hash();
		assert!(id1 > id2);
		id2.seed = max_hash();
		assert!(id1 > id2);

		id2 = id1;
		id2.seed[31] += 1;
		assert!(id1 < id2);
		id2.sender = min_hash();
		assert!(id1 < id2);

		id2 = id1;
		id2.seed[31] -= 1;
		assert!(id1 > id2);
		id2.sender = max_hash();
		assert!(id1 > id2);

		id2 = id1;
		id2.sender[31] += 1;
		assert!(id1 < id2);

		id2 = id1;
		id2.sender[31] -= 1;
		assert!(id1 > id2);

		id2 = id1;
		id2.args_hash.unwrap()[31] += 1;
		assert_eq!(id1, id2);

		id2 = id1;
		id2.args_hash.unwrap()[31] -= 1;
		assert_eq!(id1, id2);
	}

	#[test]
	fn tsid_hash_works() {
		assert_eq!(Tsid::genesis(), Tsid::genesis());

		let id1 = Tsid {
			ts: 1111,
			sender: [2; 32],
			hash: [3; 32],
			seed: [4; 32],
			args_hash: Some([5; 32]),
			nonce: None,
		};

		let mut id2 = id1;
		assert_eq!(calculate_hash(&id1), calculate_hash(&id2));

		id2.ts = 5555;
		assert_ne!(calculate_hash(&id1), calculate_hash(&id2));

		id2 = id1;
		id2.sender = [5; 32];
		assert_ne!(calculate_hash(&id1), calculate_hash(&id2));

		id2 = id1;
		id2.hash = [5; 32];
		assert_ne!(calculate_hash(&id1), calculate_hash(&id2));

		id2 = id1;
		id2.seed = [5; 32];
		assert_ne!(calculate_hash(&id1), calculate_hash(&id2));

		id2 = id1;
		id2.args_hash = Some([6; 32]);
		assert_eq!(calculate_hash(&id1), calculate_hash(&id2));
	}

	fn calculate_hash(tsid: &Tsid) -> u64 {
		let mut hasher = DefaultHasher::new();
		tsid.hash(&mut hasher);
		hasher.finish()
	}

	fn min_hash() -> Hash {
		[0u8; 32]
	}

	fn max_hash() -> Hash {
		[u8::MAX; 32]
	}

	#[test]
	fn test_from_readable() {
		let tsid_readable = TsidReadable {
			ts: 1687402281641461423,
			sender: "0000000000000000000000000000000000000000000000000000000000000000".into(),
			hash: "0ce12f9cd99e4c3b2804ebf697762f19d28cb294561e17344f973daf264caf9d".into(),
			seed: "715c6b9229a3e0c5974db007d2ef7a03eb939d6ca8f0a617995181a96abba019".into(),
		};
		let mut tsid = Tsid::genesis();
		tsid.reover_from_readable(tsid_readable);

		assert_eq!(tsid.ts, 1687402281641461423);
		println!("{:?}", tsid);
	}
}
