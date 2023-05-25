use serde::{Deserialize, Serialize};

#[doc(hidden)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FluencerCandidateListItem {
	pub id: String,
	pub r#type: String,
	pub key: String,
	pub url: String,
}
