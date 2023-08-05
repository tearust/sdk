use ed25519_dalek::SignatureError;
use tea_codec::{
	define_scope,
	errorx::{smallvec, SmallVec},
};
use tea_runtime_codec::actor_txns::error::ActorTxns;
use tea_runtime_codec::runtime::error::RuntimeCodec;
use tea_runtime_codec::solc::error::SolcCodec;
use tea_runtime_codec::tapp::{error::TApp, Balance, TokenId};
use tea_runtime_codec::vmh::error::VmhCodec;
use thiserror::Error;

define_scope! {
	Actor: pub RuntimeCodec, VmhCodec, SolcCodec, TApp, ActorTxns {
		Errors as v => ServiceCall, @Display, @Debug, v.inner();
		ProviderOperationRejected => ProviderOperationRejected, @Display, @Debug;
		AsyncNotFinished => AsyncNotFinished, @Display, @Debug;
		Layer1Errors => Layer1, @Display, @Debug;
		ProcessTransactionErrorFailed as v => ProcessTransactionErrorFailed, format!("process transaction error failed: {}", v.0), @Debug, [&v.0];
		SignatureError => SignatureError, @Display, @Debug;
		GlueSqlErrors => GlueSqlErrors, @Display, @Debug;
	}
}

#[derive(Error, Debug)]
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
	FailedToParse(&'static str, String),

	#[error("Reject because target conn id {0} don't exist in current peer list")]
	ConnIdNotExist(String),

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
	StateMachineMoveFailed(String, String, Balance, Error<()>),

	#[error("Actor_statemachine cross_move failed. From token_id {0} account {1} to token_id {2} account {3} with amount {4}. Reason: {5}")]
	StateMachineCrossMoveFailed(String, String, String, String, Balance, Error<()>),

	#[error("Unknown txn request")]
	UnknownTxnRequest,

	#[error("Handle reply actor key mismatched, expected is {0}, actual is {1}")]
	HandleReplyActorKeyMismatch(&'static str, String),

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

	#[error("failed to find libp2p callback with seq number {0}")]
	Libp2pCallbackIsNone(u64),
}

impl Errors {
	fn inner(&self) -> SmallVec<[&Error<()>; 1]> {
		match self {
			Self::StateMachineMoveFailed(_, _, _, err) => smallvec![err.as_scope()],
			Self::StateMachineCrossMoveFailed(_, _, _, _, _, err) => smallvec![err.as_scope()],
			_ => Default::default(),
		}
	}
}

#[derive(Debug, Error)]
#[error("async not finished")]
pub struct AsyncNotFinished;

#[derive(Debug)]
pub struct ProcessTransactionErrorFailed(pub Error<()>);

#[derive(Error, Debug)]
pub enum Layer1Errors {
	#[error("Failed to find block, the raw request is {0}")]
	FailedToFindBlock(String),

	#[error("Failed to convert to fixed tea id, expect buffer length: {0}, actual length: {1}")]
	BufferLengthMismatch(usize, usize),

	#[error("U128 length should be {0}")]
	U128LengthMismatch(usize),
}

#[derive(Error, Debug)]
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

#[derive(Error, Debug)]
pub enum ProviderOperationRejected {
	#[error("Failed to intelli send txn because I'm not A type cml")]
	NotATypeCml,
}
