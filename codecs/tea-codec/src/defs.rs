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

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetVersionRequest;

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetVersionResponse(pub String);

#[macro_export]
macro_rules! impl_version {
	( $x:ident ) => {
		impl tea_sdk::serde::handle::Handle<tea_sdk::defs::GetVersionRequest> for $x {
			async fn handle(
				&self,
				_: tea_sdk::defs::GetVersionRequest,
			) -> tea_sdk::Result<tea_sdk::defs::GetVersionResponse> {
				let version = env!("CARGO_PKG_VERSION");
				Ok(tea_sdk::defs::GetVersionResponse(version.to_string()))
			}
		}
	};
}
