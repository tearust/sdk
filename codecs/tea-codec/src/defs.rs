use serde::{Deserialize, Serialize};
use tea_codec_macros::TypeId;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FreezeTimeSettings {
	pub schedule_at: i64,   // timestamp in seconds
	pub freeze_before: u64, // freeze seconds
	pub freeze_after: u64,  // freeze seconds
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(Vec<u8>)]
pub struct ExportRequest;

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct ImportRequest(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct SetFreezeRequest(pub FreezeTimeSettings);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct CancelFreezeRequest;

pub const RUNTIME_NAME: &[u8] = b"com.tea.system";
