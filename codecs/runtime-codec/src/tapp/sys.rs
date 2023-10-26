use serde::{Deserialize, Serialize};
use tea_sdk::defs::FreezeTimeSettings;

#[doc(hidden)]
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FreezeRequest {
	pub time: FreezeTimeSettings,
}
