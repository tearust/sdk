use serde::{Deserialize, Serialize};

#[doc(hidden)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StartMiningEvent {
	pub tea_id: Vec<u8>,
	pub ip_address: String,
}

#[doc(hidden)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StopMiningEvent {
	pub tea_id: Vec<u8>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MigrateEvent {
	pub tea_id: Vec<u8>,
	pub ip_address: Option<String>,
}
