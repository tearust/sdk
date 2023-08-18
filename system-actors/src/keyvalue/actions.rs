use serde::{Deserialize, Serialize};
use tea_codec::pricing::Priced;
use tea_codec::serde::TypeId;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(0)]
pub struct GetRequest {
	pub key: String,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetResponse {
	pub exists: bool,
	pub value: Option<Vec<u8>>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct SetRequest {
	pub key: String,
	pub value: Vec<u8>,
	pub expires_s: Option<i32>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct SetResponse {
	pub value: Vec<u8>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct DelRequest {
	pub key: String,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct DelResponse {
	pub key: String,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct AddRequest {
	pub key: String,
	pub value: i32,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct AddResponse {
	pub value: i32,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(ListResponse)]
pub struct ListPushRequest {
	pub key: String,
	pub value: Vec<u8>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(ListResponse)]
pub struct ListDelItemRequest {
	pub key: String,
	pub value: Vec<u8>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(DelResponse)]
pub struct ListClearRequest {
	pub key: String,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct ListRangeRequest {
	pub key: String,
	pub start: i32,
	pub stop: i32,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ListRangeResponse {
	pub values: Vec<Vec<u8>>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ListResponse {
	pub new_count: i32,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(SetOperationResponse)]
pub struct SetAddRequest {
	pub key: String,
	pub value: Vec<u8>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(SetOperationResponse)]
pub struct SetRemoveRequest {
	pub key: String,
	pub value: Vec<u8>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct SetQueryRequest {
	pub key: String,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct SetQueryResponse {
	pub values: Vec<Vec<u8>>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(SetQueryResponse)]
pub struct SetIntersectionRequest {
	pub keys: Vec<String>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(SetQueryResponse)]
pub struct SetUnionRequest {
	pub keys: Vec<String>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct SetOperationResponse {
	pub new_count: i32,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct KeyExistsQueryRequest {
	pub key: String,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct KeyExistsQueryResponse {
	pub exists: bool,
	pub value: Option<Vec<u8>>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct TupleKeyValue {
	pub k: i32,
	pub v: Vec<u8>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct KeyVecInsertRequest {
	pub key: String,
	pub value: Option<TupleKeyValue>,
	pub overwrite: bool,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct KeyVecInsertResponse {
	pub success: bool,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct KeyVecTailOffRequest {
	pub key: String,
	pub remain: u32,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct KeyVecTailOffResponse {
	pub len: u32,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct KeyVecGetRequest {
	pub key: String,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct KeyVecGetResponse {
	pub values: Vec<TupleKeyValue>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct KeyVecRemoveItemRequest {
	pub key: String,
	pub value_idx: i32,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct KeyVecRemoveItemResponse {
	pub success: bool,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct PersistentStorage {
	pub kvp: ::std::collections::HashMap<String, Vec<u8>>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct PersistentStoreRequest {
	pub prefix_list: Vec<String>,
	pub file_name: String,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(0)]
pub struct RestoreFromFileRequest {
	pub file_name: String,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct RestoreFromFileResponse {
	pub keys: Vec<String>,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(0)]
pub struct TaskMemorySizeRequest {
	pub uuid: String,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct TaskMemorySizeResponse {
	pub size: u64,
}
