use serde::{Deserialize, Serialize};
use tea_sdk::defs::FreezeTimeSettings;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FreezeRequest {
	pub time: FreezeTimeSettings,
	pub options: FreezeOptions,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreezeOptions {
	/// do not allow custom txns send from B nodes
	pub custom_txn: bool,
	/// persis cronjobs
	pub persist_cronjob: bool,
	/// all random tick drops
	pub ticks: bool,
	/// all tappstore cronjobs
	pub tappstore_cronjobs: bool,
	/// freeze layer1 event
	pub layer1: bool,
	/// freeze all outside http request
	pub adapter: bool,
	/// freeze all libp2p request
	pub libp2p: bool,
}

impl Default for FreezeOptions {
	fn default() -> Self {
		Self {
			custom_txn: true,         // freeze custom txn by default
			layer1: true,             // freeze layer1 event to avoid state inconsistency
			tappstore_cronjobs: true, // freeze appstore cronjobs to avoid execute scheduled txns
			persist_cronjob: false,   // allow scheduled persistent by default
			ticks: false,             // all random ticks by default
			adapter: false,           // all outside http request by default
			libp2p: false,            // all libp2p request (node communications) by default
		}
	}
}
