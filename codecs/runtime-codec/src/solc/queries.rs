use serde::{Deserialize, Serialize};

use crate::solc::{BlockNumber, CmlId};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AsyncQuery {
	pub at_height: Option<BlockNumber>,
	pub query_type: QueryType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum QueryType {
	Default,
	CmlInfo(Vec<CmlId>),
	MultisigThreshold,
}

impl Default for QueryType {
	fn default() -> Self {
		QueryType::Default
	}
}
