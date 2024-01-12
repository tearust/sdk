use serde::{Deserialize, Serialize};
use tea_runtime_codec::actor_txns::error::ActorTxnsError;
use tea_runtime_codec::runtime::error::RuntimeCodec;
use tea_runtime_codec::tapp::error::RuntimeTappError;
use tea_runtime_codec::tapp::{Balance, TokenId};
use tea_runtime_codec::vmh::error::VmhError;
use tea_sdk::errorx::Global;
use thiserror::Error;

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Error, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Error {
	#[error("Wasm actor utils error: {0}")]
	Unnamed(String),

	#[error("Global error: {0}")]
	Global(#[from] Global),

	#[error("Vmh code error: {0}")]
	VmhCodec(String),

	#[error("Wasm runtime tapp error: {0}")]
	RuntimeTapp(String),

	#[error("Wasm runtime codec error: {0}")]
	RuntimeCodec(String),

	#[error("Actor txns error: {0}")]
	ActorTxnsError(String),

	#[error("Utils general error: {0}")]
	General(#[from] Errors),

	#[error("Layer1 error: {0}")]
	Layer1(#[from] Layer1Errors),

	#[error("Glue sql error: {0}")]
	GlueSqlErrors(#[from] GlueSqlErrors),

	#[error(transparent)]
	ProcessTransactionErrorFailed(#[from] ProcessTransactionErrorFailed),

	#[error(transparent)]
	ProviderOperationRejected(#[from] ProviderOperationRejected),

	#[error(transparent)]
	AsyncNotFinished(#[from] AsyncNotFinished),

	#[error("Http request error: {0}")]
	HttpRequest(String),

	#[error("Parse address error: {0}")]
	ParseAddress(String),
}

impl From<VmhError> for Error {
	fn from(e: VmhError) -> Self {
		Self::VmhCodec(format!("{e:?}"))
	}
}

impl From<RuntimeTappError> for Error {
	fn from(e: RuntimeTappError) -> Self {
		Self::RuntimeTapp(format!("{e:?}"))
	}
}

impl From<RuntimeCodec> for Error {
	fn from(e: RuntimeCodec) -> Self {
		Self::RuntimeCodec(format!("{e:?}"))
	}
}

impl From<ActorTxnsError> for Error {
	fn from(e: ActorTxnsError) -> Self {
		Self::ActorTxnsError(format!("{e:?}"))
	}
}

#[derive(Error, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Errors {
	#[error("When calling request_intercom, always leave reply_to empty, because it is used for response socket")]
	ReplyToNotEmpty,

	#[error("Actor request intercom failed")]
	ActorRequestIntercomFailed,

	#[error("Failed to convert public key to account, public key is: {0:?}")]
	FailedToConvertPublicKeyToAccount(Vec<u8>),

	#[error("Tea id is uninitialized")]
	UninitializedTeaId,

	#[error("Ephemeral public key is uninitialized")]
	UninitializedEphemeralPublicKey,

	#[error("Ephemeral key is uninitialized")]
	UninitializedEphemeralKey,

	#[error("Failed to get environment variable: {0}")]
	FailedToGetEnvironmentVariable(String),

	#[error("Failed to parse {0} from {1}")]
	FailedToParse(String, String),

	#[error("Reject because target conn id {0} don't exist in current peer list")]
	ConnIdNotExist(String),

	#[error("Reject because target conn id list {0:?} don't exist in current peer list")]
	ConnIdsNotExist(Vec<String>),

	#[error("Raft set value failed: {0}")]
	RaftSetValueFailed(String),

	#[error("Raft get value failed: {0}")]
	RaftGetValueFailed(String),

	#[error("Raft delete value failed: {0}")]
	RaftDeleteValueFailed(String),

	#[error("Validators members and conn ids length mismatched when get validator members")]
	MembersConnIdsMismatch,

	#[error("Neutralize expectation: (neutralize credit: {0}, neutralize debit: {1}), actual neutral balance: {2:?}")]
	NeutralizeExpectation(Balance, Balance, (Balance, Balance)),

	#[error("Actor_statemachine move {0} to {1} with amount {2} failed: {3}")]
	StateMachineMoveFailed(String, String, Balance, String),

	#[error("Actor_statemachine cross_move failed. From token_id {0} account {1} to token_id {2} account {3} with amount {4}. Reason: {5}")]
	StateMachineCrossMoveFailed(String, String, String, String, Balance, String),

	#[error("Unknown txn request")]
	UnknownTxnRequest,

	#[error("Handle reply actor key mismatched, expected is {0}, actual is {1}")]
	HandleReplyActorKeyMismatch(String, String),

	#[error("Actor not support system arg: {0}")]
	InvalidSystemArgs(String),

	#[error("none of async returns reached")]
	NoneAsyncReturn,

	#[error("local validiators is empty")]
	ValidatorIsEmpty,

	#[error("connected peers is empty")]
	ConnectedPeersIsEmpty,

	#[error("async persisted request failed: {0}")]
	AsyncPersistFailed(String),

	#[error("failed to find libp2p callback")]
	Libp2pCallbackIsNone,

	#[error("all libp2p response error, last is: {0}")]
	Libp2pAllResponseError(String),
}

#[derive(Error, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[error("async not finished")]
pub struct AsyncNotFinished;

#[derive(Error, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[error("process transaction error failed: {0}")]
pub struct ProcessTransactionErrorFailed(pub String);

#[derive(Error, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Layer1Errors {
	#[error("Failed to find block, the raw request is {0}")]
	FailedToFindBlock(String),

	#[error("Failed to convert to fixed tea id, expect buffer length: {0}, actual length: {1}")]
	BufferLengthMismatch(usize, usize),

	#[error("U128 length should be {0}")]
	U128LengthMismatch(usize),
}

#[derive(Error, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum GlueSqlErrors {
	#[error("failed to get first row")]
	InvalidFirstRow,

	#[error("failed to get first value")]
	InvalidFirstValue,

	#[error("failed to get payload select result")]
	InvalidSelectResult,

	#[error("failed to get row count of table {0} about token {1:?}")]
	InvalidTableRowCount(String, TokenId),

	#[error("value {0} to I64 failed")]
	InvalidI64(String),

	#[error("value {0} to String failed")]
	InvalidString(String),

	#[error("failed to get first payload with sql '{0}' about token {1:?}")]
	InvalidFirstPayload(String, TokenId),
}

#[derive(Error, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProviderOperationRejected {
	#[error("Failed to intelli send txn because I'm not A type cml")]
	NotATypeCml,
}
