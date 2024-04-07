use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tea_actorx::ActorId;
use tea_codec::serde::TypeId;
use tea_runtime_codec::tapp::{
	cml::{CmlId, CmlIntrinsic, CmlVariable},
	fluencer::CreditSystemInfo,
	machine::{MiningIntrinsic, MiningVariable},
	ra::{EnclaveType, NodeStatus, PcrType, TeaNodeProfile},
	seat::SeatMaintainer,
	statement::TypedStatement,
	sys::FreezeRequest,
	version::{GlobalVersionsReadable, SystemVersions},
	Account, AuthKey, Hash, TokenId,
};
use tea_runtime_codec::{
	actor_txns::{
		pre_args::{Arg, ArgSlots},
		tsid::Tsid,
	},
	tapp::ra::VersionPcrs,
};

pub mod txns;

pub type QueryCmlResultItem = (CmlIntrinsic, CmlVariable, Vec<u8>, Option<TokenId>);

pub const NAME: &[u8] = b"com.tea.tappstore-actor";

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryCmlIntrinsicRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryCmlIntrinsicResponse(pub CmlIntrinsic);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryMiningVariableRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryMiningVariableResponse(pub MiningVariable);

/// Base request to return mining CMLs.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryMiningCmlsRequest(pub Vec<u8>);

/// Base response to return mining CMLs.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryMiningCmlsResponse(pub Vec<QueryCmlResultItem>);

/// Base request to return all mining CML ids.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryMiningCmlIdsRequest;

/// Base response from returning all mining CML ids.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryMiningCmlIdsResponse(pub Vec<CmlId>);

/// Base request to return machine info.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryMachineInfoRequest(pub Vec<u8>);

/// Base response from returning machine info.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryMachineInfoResponse(pub MiningIntrinsic, pub MiningVariable);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct NodeProfileByConnIdRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct NodeProfileByConnIdResponse(pub TeaNodeProfile);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct NodeProfileByTeaIdRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct NodeProfileByTeaIdResponse(pub TeaNodeProfile);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct NodeProfileByTeaIdsRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct NodeProfileByTeaIdsResponse(pub Vec<TeaNodeProfile>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct NodeProfileByConnIdsRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct NodeProfileByConnIdsResponse(pub Vec<TeaNodeProfile>);

/// Base request to return active node.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryActiveNodesRequest(pub Vec<u8>);

/// Base response from returning active node.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryActiveNodesResponse(pub Vec<TeaNodeProfile>);

/// Base request to return active CML info.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryActiveCmlsRequest(pub Vec<u8>);

/// Base response to return active CML info.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryActiveCmlsResponse(pub Vec<CmlId>);

/// Base request to return active seat info.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryActiveSeatsRequest(pub Vec<u8>);

/// Base response to return active seat info.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryActiveSeatsResponse(pub Vec<SeatMaintainer>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryActiveSeatsAsyncRequest {
	pub sender: ActorId,
	pub tsid: Tsid,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryActiveSeatsAsyncResponse {
	pub seat_nodes: Vec<SeatMaintainer>,
	pub tsid: Tsid,
}

/// Base request to return account tea balance.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryTeaBalanceRequest(pub Vec<u8>);

/// Base response to return account tea balance.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryTeaBalanceResponse(pub Vec<u8>);

/// Base request to return account tea deposit.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryTeaDepositRequest(pub Vec<u8>);

/// Base response to return account tea deposit.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryTeaDepositResponse(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct FetchAccountAssetRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct FetchAccountAssetResponse(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct FindExecutedTxnRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct FindExecutedTxnResponse(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct FindExecutedTxnFromAllRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct FindExecutedTxnFromAllResponse(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct CheckUserSessionRequest {
	pub token_id: TokenId,
	pub account: Account,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct CheckUserSessionResponse {
	pub aes_key: Vec<u8>,
	pub auth_key: Option<AuthKey>,
	pub token_id: TokenId,
	pub account: Account,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct CommonSqlQueryRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct CommonSqlQueryResponse(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetStatementsRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetStatementsResponse(pub Vec<(TypedStatement, String, String)>, pub bool);

/// Base request to return account allowance.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct FetchAllowanceRequest(pub Vec<u8>);

/// Base response to return account allowance.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct FetchAllowanceResponse(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryCmlIdsByTeaIdsRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryCmlIdsByTeaIdsResponse(pub Vec<CmlId>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryHostingCmlsRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryHostingCmlsResponse(pub Vec<CmlId>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryCmlRaStatusRequest(pub Vec<u8>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryCmlRaStatusResponse(pub NodeStatus);

/// Base request to return system version.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QuerySystemVersionsRequest;

/// Base response to return system version.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QuerySystemVersionsResponse(pub SystemVersions);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryCreditSystemInfoRequest;

/// Base response to return system version.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryCreditSystemInfoResponse(pub Option<CreditSystemInfo>, pub String);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ProcessPreArgsRequest(pub Vec<Arg>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ProcessPreArgsResponse(pub ArgSlots);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryLastFreezeRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct QueryLastFreezeResponse(pub Option<FreezeRequest>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ListAvailablePcrsRequest(pub EnclaveType);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ListAvailablePcrsResponse(pub Vec<HashMap<PcrType, Vec<u8>>>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ListAvailablePcrsExRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ListAvailablePcrsExResponse(pub Vec<VersionPcrs>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct HandleTappAddErrorLogRequest {
	pub token_id: TokenId,
	pub actor_type: String,
	pub details: String,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct HandleTappRemoveErrorLogRequest {
	pub token_id: TokenId,
	pub actor_type: String,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct CheckUserTappstoreAuthRequest {
	pub auth_b64: String,
	pub user: Account,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct UnregisterMultisigInfoRequst {
	pub txn_hash: Hash,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct IsNodeRaValidRequest {
	pub conn_id: String,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct IsNodeRaValidResponse {
	pub valid: bool,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ListActorVersionsRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ListActorVersionsResponse(pub HashMap<String, String>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GlobalVersionsReadableRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GlobalVersionsReadableResponse(pub GlobalVersionsReadable);
