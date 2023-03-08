use serde::{Deserialize, Serialize};
use tea_codec::pricing::Priced;
use tea_codec::serde::TypeId;
use tea_runtime_codec::actor_txns::{pre_args::ArgSlots, tsid::Tsid, txn::FullTxn};
use tea_runtime_codec::tapp::{Hash, ReplicaId};

pub mod error;

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct TxnItem {
	pub txn: FullTxn,
	pub tsid: Tsid,
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct HistoryItem {
	pub txn_item: TxnItem,
	pub err_msg: Option<String>,
}

pub const NAME: &[u8] = b"tea:replica";

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GenerateSyncMessageRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GenerateSyncMessageResponse(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct ReceiveSyncMessageRequest(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct CleanUpSyncReplicaRequest;

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetReplicaCountRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetReplicaCountResponse {
	pub count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct ReceiveFollowupRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ReceiveFollowupResponse(pub Option<Tsid>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct ReceiveTxnRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ReceiveTxnResponse(pub Option<Tsid>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct FetchHistoryRequest {
	pub end_tsid: Vec<u8>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct FetchHistoryResponse {
	pub history_items: Vec<TxnItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct RecoverHistoryRequest(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetHistorySinceRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetHistorySinceResponse(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct ResetReplicasCountRequest(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct FindExecutedTxnRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct FindExecutedTxnResponse(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetConsensusReplicasRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetConsensusReplicasResponse(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(0)]
#[response(())]
pub struct SetSingleModeRequest {
	pub single_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetSingleModeRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetSingleModeResponse(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct ReportTxnExecErrorRequest(pub Hash, pub String);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetExecCursorRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetExecCursorResponse(pub Option<Tsid>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct ResetExecCursorRequest(pub Tsid);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(0)]
pub struct DumpTxnSeqRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct DumpTxnSeqResponse(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct ResetMagicNumberRequest(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(0)]
pub struct IdleCleanupRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct IdleCleanupResponse(pub Vec<ReplicaId>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct TryPopupReadyTxnRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct TryPopupReadyTxnResponse(pub Option<(Tsid, FullTxn)>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct AppendToHistoryRequest(pub HistoryItem);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct IsTxnAlreadyExecutedRequest(pub Tsid);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct IsTxnAlreadyExecutedResponse(pub bool);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10)]
#[response(())]
pub struct ExecTxnCast(pub Tsid, pub Vec<u8>, pub Option<ArgSlots>);
