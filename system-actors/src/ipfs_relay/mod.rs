use serde::{Deserialize, Serialize};
use tea_codec::{pricing::Priced, serde::TypeId};
use tea_runtime_codec::tapp::TokenId;
use tea_runtime_codec::vmh::ipfs::IpfsRequestType;

pub const NAME: &[u8] = b"tea:ipfs";
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct RelayRequestRequest(pub IpfsRequestType);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct RelayRequestResponse(pub Vec<u8>);

pub const OP_IDENTITY: &str = "Identity";
pub const OP_PUT_VIEWS: &str = "PutViews";
pub const OP_GET_VIEWS: &str = "GetViews";
pub const OP_WRITE_ORBIT_ID: &str = "WriteOrBitId";

pub const OP_BBS_ADD_MESSAGE: &str = "bbs_AddMessage";
pub const OP_BBS_GET_MESSAGE: &str = "bbs_GetMessage";
pub const OP_BBS_DELETE_MESSAGE: &str = "bbs_DeleteMessage";
pub const OP_BBS_EXTEND_MESSAGE: &str = "bbs_ExtendMessage";

pub const OP_NOTIFICATION_ADD_MESSAGE: &str = "notification_AddMessage";
pub const OP_NOTIFICATION_GET_MESSAGE: &str = "notification_GetMessage";

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct GetFileRequest {
	pub cid: String,
	pub filename: String,
	pub token_id: TokenId,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(Vec<u8>)]
pub struct GetFileLocallyRequest {
	pub filename: String,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct PersistFileRequest {
	pub filename: String,
	pub content: Vec<u8>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct GetFileReply {
	pub cid: String,
	pub filename: String,
	pub token_id: TokenId,
	pub download_path: String,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct IsFileExistRequest {
	pub filename: String,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct IsFileExistResponse(pub bool);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[response(())]
pub struct SetDownloaded {
	pub cid: String,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetDownloadPathRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetDownloadPathResponse(pub String);
