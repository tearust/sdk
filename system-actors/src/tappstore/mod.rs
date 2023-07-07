use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tea_actorx_core::ActorId;
use tea_codec::serde::TypeId;
use tea_runtime_codec::actor_txns::{
	pre_args::{Arg, ArgSlots},
	tsid::Tsid,
};
use tea_runtime_codec::solc::txns::{MintCmlRecordTrans, UnlockRecordTrans};
use tea_runtime_codec::tapp::{
	cml::{CmlId, CmlIntrinsic, CmlVariable},
	machine::{MiningIntrinsic, MiningVariable},
	ra::{EnclaveType, NodeStatus, PcrType, TeaNodeProfile},
	seat::SeatMaintainer,
	statement::TypedStatement,
	sys::FreezeRequest,
	version::SystemVersions,
	Account, AuthKey, Hash, TimestampShort, TokenId,
};

pub mod error;

pub mod txns;

pub type QueryCmlResultItem = (CmlIntrinsic, CmlVariable, Vec<u8>, Option<TokenId>);

pub const NAME: &[u8] = b"com.tea.tappstore-actor";

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryCmlIntrinsicRequest(pub String, pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryCmlIntrinsicResponse(pub CmlIntrinsic);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryMiningVariableRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryMiningVariableResponse(pub MiningVariable);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryMiningCmlsRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryMiningCmlsResponse(pub Vec<QueryCmlResultItem>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryMiningCmlIdsRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryMiningCmlIdsResponse(pub Vec<CmlId>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryMachineInfoRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryMachineInfoResponse(pub MiningIntrinsic, pub MiningVariable);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct NodeProfileByConnIdRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct NodeProfileByConnIdResponse(pub TeaNodeProfile);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct NodeProfileByTeaIdRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct NodeProfileByTeaIdResponse(pub TeaNodeProfile);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct NodeProfileByTeaIdsRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct NodeProfileByTeaIdsResponse(pub Vec<TeaNodeProfile>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct NodeProfileByConnIdsRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct NodeProfileByConnIdsResponse(pub Vec<TeaNodeProfile>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryActiveNodesRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryActiveNodesResponse(pub Vec<TeaNodeProfile>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryActiveCmlsRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryActiveCmlsResponse(pub Vec<CmlId>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryActiveSeatsRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryActiveSeatsResponse(pub Vec<SeatMaintainer>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct QueryActiveSeatsAsyncRequest {
	pub sender: ActorId,
	pub tsid: Tsid,
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct QueryActiveSeatsAsyncReply {
	pub seat_nodes: Vec<SeatMaintainer>,
	pub tsid: Tsid,
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryTeaBalanceRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryTeaBalanceResponse(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryTeaDepositRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryTeaDepositResponse(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct FetchAccountAssetRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct FetchAccountAssetResponse(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct FindExecutedTxnRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct FindExecutedTxnResponse(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct CheckUserSessionRequest {
	pub token_id: TokenId,
	pub account: Account,
}
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct CheckUserSessionResponse {
	pub aes_key: Vec<u8>,
	pub auth_key: Option<AuthKey>,
	pub token_id: TokenId,
	pub account: Account,
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct CommonSqlQueryRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct CommonSqlQueryResponse(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetStatementsRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetStatementsResponse(pub Vec<(TypedStatement, String, String)>, pub bool);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct FetchAllowanceRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct FetchAllowanceResponse(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryCmlIdsByTeaIdsRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryCmlIdsByTeaIdsResponse(pub Vec<CmlId>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryHostingCmlsRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryHostingCmlsResponse(pub Vec<CmlId>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryCmlRaStatusRequest(pub Vec<u8>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryCmlRaStatusResponse(pub NodeStatus);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QuerySystemVersionsRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QuerySystemVersionsResponse(pub SystemVersions);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryExpiredWithdrawsRequest(pub TimestampShort);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryExpiredWithdrawsResponse(pub Vec<(Hash, Vec<UnlockRecordTrans>, String)>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryExpiredMintCmlsRequest(pub TimestampShort);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryExpiredMintCmlsResponse(pub Vec<(Hash, Vec<MintCmlRecordTrans>, String)>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ProcessPreArgsRequest(pub Vec<Arg>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ProcessPreArgsResponse(pub ArgSlots);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryLastFreezeRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryLastFreezeResponse(pub Option<FreezeRequest>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ListAvailablePcrsRequest(pub EnclaveType);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ListAvailablePcrsResponse(pub Vec<HashMap<PcrType, Vec<u8>>>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct HandleTappAddErrorLogRequest {
	pub token_id: TokenId,
	pub actor_type: String,
	pub details: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct HandleTappRemoveErrorLogRequest {
	pub token_id: TokenId,
	pub actor_type: String,
}
