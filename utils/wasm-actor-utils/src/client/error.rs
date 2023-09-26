use crate::enclave::error::Actor;
use tea_codec::define_scope;
use thiserror::Error;

define_scope! {
	ClientUtilityActor: Actor {
		Errors => ClientUtilityActor, @Display, @Debug;
	}
}

#[derive(Debug, Error)]
pub enum Errors {
	#[error("Unknown request")]
	UnknownRequest,

	#[error("Unknown action {0}")]
	UnknownAction(String),

	#[error("failed to parse address from string")]
	ParseAddressError,

	#[error("u128 length should be {0}")]
	U128Length(usize),

	#[error("You need at least 0.001 TEA or Credit to submit txn.")]
	NotEnoughBalanceForTxn,
}
