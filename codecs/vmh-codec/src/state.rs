use actor_txns::tsid::Tsid;

pub mod dump;

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
