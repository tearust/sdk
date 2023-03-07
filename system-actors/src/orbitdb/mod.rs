use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use tea_codec::pricing::Priced;
use tea_codec::serde::TypeId;

pub mod error;

pub const NAME: &[u8] = b"tea:orbitdb";
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct ProtoRequest(pub Cow<'static, str>, pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ProtoResponse(pub Vec<u8>);

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
