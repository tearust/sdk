use serde::{Deserialize, Serialize};
use tea_actorx::ActorId;
use tea_codec::pricing::Priced;
use tea_codec::serde::TypeId;
use tea_runtime_codec::vmh::io::HostType;

pub mod error;

pub const NAME: &[u8] = b"tea:libp2p";

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct ReplyMessageRequest {
	pub reply_msg: Vec<u8>,
	pub caller: ActorId,
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct MyConnIdRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct MyConnIdResponse(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct PubMessageRequest(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct StartUseIpRequest(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct StopUseIpRequest(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct CloseLibp2pRequest(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct HasCooldownRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct HasCooldownResponse(pub bool);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct ListPeersRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ListPeersResponse(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct RandomPeersRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct RandomPeersResponse(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct SubscribeGossipTopicRequest(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct UnsubscribeGossipTopicRequest(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct SendMessageRequest(pub Vec<u8>, pub bool);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct SendMessageResponse(pub Option<Vec<u8>>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct SendMessageExRequest {
	pub msg: Vec<u8>,
	pub with_reply: bool,
	pub targets: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct SendMessageExResponse(pub Option<Vec<u8>>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct BoundState {
	pub conn_id: String,
	pub port: u32,
	pub host: HostType,
	pub caller: ActorId,
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(100)]
#[response(())]
pub struct Libp2pRequest(pub String, pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(Vec<u8>)]
pub struct DecryptDataRequest {
	pub conn_id: String,
	pub encryted_data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct HandshakeRequest {
	pub conn_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct RecordEncryptKeyRequest {
	pub conn_id: String,
	pub ciphertext: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct ActivateEncryptKeyRequest {
	pub conn_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct DumpPeersSecurityRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct DumpPeersSecurityResponse(pub Vec<u8>);
