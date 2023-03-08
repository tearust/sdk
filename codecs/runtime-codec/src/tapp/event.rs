use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StartMiningEvent {
	pub tea_id: Vec<u8>,
	pub ip_address: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StopMiningEvent {
	pub tea_id: Vec<u8>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MigrateEvent {
	pub tea_id: Vec<u8>,
	pub ip_address: Option<String>,
}
