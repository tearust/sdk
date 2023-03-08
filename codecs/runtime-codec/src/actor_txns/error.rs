use crate::actor_txns::{auth::AllowedOp, context::TappStorageType};
use crate::runtime::error::RuntimeCodec;
use crate::tapp::{error::TApp, Account, AuthKey, TokenId};
use serde::{Deserialize, Serialize};
use tea_sdk::define_scope;
use thiserror::Error;

define_scope! {
	ActorTxns: pub RuntimeCodec, TApp {
		TxnError => Txn, @Display, @Debug;
		ContextError => Context, @Display, @Debug;
	}
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum TxnError {
	#[error("Txn error:'{0}")]
	ErrorMessage(String),
	#[error("Unknown error")]
	Unknown,
	#[error("New txn type that not handled")]
	NewTxnTypeNotHandled,
	#[error("Parase bytes into TxnSerial failed. Error: {0:?}")]
	ParseFailed(String),
	#[error("Cannot read authkey or expired. authkey:{0}, err:{1}")]
	NoAuthKey(AuthKey, String),
	#[error("Hash {1:?} failed: {0}")]
	TxnHashError(String, Vec<u8>),
	#[error("Pre-args hash failed: {0}")]
	PreArgsHashError(String),
	#[error(
		"CX_214__login_did_not_authorize_this_operation__'token_id:{0:?},acct:{1:?},op:{2:?}'"
	)]
	AuthCheckFailed(TokenId, Account, AllowedOp),
	#[error("CX_219__storage_'{0:?}'_not_been_changed")]
	StorageIsEmpty(TappStorageType),
	#[error("CX_203__shouldnot_compare_two_context_with_the_same_tsid")]
	ShouldNotCheckSameTsid,
	#[error(
		"CX_208__shouldnot_check_conflict_with_any_tsid_later_than_myself_and_only_check_with_earlier_tsid"
	)]
	ShouldNotCheckConflictWithLaterTsid,
	#[error("CX_202__base_ts_doesnot_match_the_current_ts_when_commit")]
	BaseNotMatchError,
	#[error("CX_218__storage_'{0:?}'_has_been_touched_already")]
	StorageHasBeTouched(TappStorageType),
}

#[derive(Debug, Error, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContextError {
	#[error("CX_204__read_an_account_while_other_debit_the_same_account")]
	ReadWhileDebit,
	#[error("CX_205__read_an_account_while_other_credit_the_same_account")]
	ReadWhileCredit,
	#[error("CX_206__debit_while_other_debit_the_same_account")]
	DoubleDebit,
	#[error("CX_220_subtraction_overflow")]
	SubtractionOverflow,
	#[error("CX_221_subtraction_overflow")]
	AddOverflow,
}
