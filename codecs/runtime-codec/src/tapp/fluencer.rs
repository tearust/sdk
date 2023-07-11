use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FluencerCandidateListItem {
	pub id: String,
	pub r#type: String,
	pub key: String,
	pub url: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExchangeRateItem {
	pub name: String,
	pub rate: String,
	pub updated_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SwapPriceItem {
	pub name: String,
	pub price: String,
}
