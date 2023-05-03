use crate::enclave::{
	actors::{
		env::tappstore_id,
		replica::report_txn_error,
		tokenstate::{cancel_sql_transaction, is_in_sql_transaction},
	},
	error::{Error, ProcessTransactionErrorFailed, Result},
};
use tea_runtime_codec::actor_txns::tsid::Tsid;
use tea_runtime_codec::tapp::TokenId;
pub use tea_system_actors::adapter::HttpRequest;

pub async fn process_txn_error(tsid: Tsid, inner: Error) -> Result<()> {
	let token_id = tappstore_id().await?;
	return if let Err(e) = process_txn_error_inner(token_id, tsid.hash.to_vec(), &inner).await {
		Err(ProcessTransactionErrorFailed(e.into()).into())
	} else {
		Err(inner)
	};
}

async fn process_txn_error_inner(token_id: TokenId, txn_hash: Vec<u8>, e: &Error) -> Result<()> {
	report_txn_error(txn_hash, serde_json::to_string(&e)?).await?;

	if is_in_sql_transaction(token_id).await? {
		cancel_sql_transaction(token_id).await?;
	}
	Ok(())
}
