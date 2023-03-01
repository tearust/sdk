use runtime_codec::error::RuntimeCodec;
use tea_sdk::define_scope;
use thiserror::Error;

define_scope! {
	TApp: pub RuntimeCodec {
		StatementTypeParse as v => StatementTypeParse, format!("Failed to parse '{}' to statement type", &v.0);
		Errors => TApp, @Display, @Debug;
	}
}

#[derive(Debug)]
pub struct StatementTypeParse(pub String);

#[derive(Debug, Error)]
pub enum Errors {
	#[error("Failed to to parse mining status from \"{0}\"")]
	ParseMiningStatus(String),

	#[error("Failed to to parse market status from \"{0}\"")]
	ParseMarketStatus(String),

	#[error("Failed to to parse maintain status from \"{0}\"")]
	ParseMaintainStatus(String),

	#[error("failed to parse {0} to node status")]
	ParseNodeStatusFailed(String),

	#[error("failed to parse {0} to pcr value")]
	UnknowPcrValue(usize),

	#[error("failed to parse address from string")]
	ParseAddressError,
}
