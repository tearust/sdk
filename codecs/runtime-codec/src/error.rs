use tea_sdk::define_scope;
use thiserror::Error;

define_scope! {
	RuntimeCodec {
		GeneralServiceError;
		GlueSQLError;
		StateGeneralError;
		HttpExecutionError;
		BondingGeneralError;
		DbNotFoundError;
		InvalidTransactionContext;
		InvalidValidator => InvalidValidator, @Display, @Display;
		UpgradeError=> UpgradeError, @Display, @Display;
		InvalidTxnRequest;
		AsyncCanceled;
	}
}

#[derive(Debug, Error)]
pub enum InvalidValidator {
	#[error("Invalid validator: {0}")]
	Valued(String),

	#[error("My validator list is empty")]
	Empty,
}

#[derive(Debug, Error)]
pub enum UpgradeError {
	#[error("version {0} is not compatible with {1}")]
	IncompatibleVersion(String, String),
}
