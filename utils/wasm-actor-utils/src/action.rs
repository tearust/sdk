use crate::{
	actors::{
		env::tappstore_id,
		replica::report_txn_error,
		tokenstate::{cancel_sql_transaction, is_in_sql_transaction},
	},
	error::{Error, Errors, ProcessTransactionErrorFailed, Result},
};
use futures::Future;
use std::{cell::UnsafeCell, collections::HashMap, pin::Pin};
use tea_actor_txns::tsid::Tsid;
use tea_tapp_common::TokenId;

pub use tea_system_actors::adapter::HttpRequest;

struct Handlers(UnsafeCell<Option<HashMap<u64, CallbackItem>>>);
unsafe impl Send for Handlers {}
unsafe impl Sync for Handlers {}

fn libp2p_handlers() -> &'static mut HashMap<u64, CallbackItem> {
	static HANDLERS: Handlers = Handlers(UnsafeCell::new(None));
	unsafe {
		let handers = &mut *HANDLERS.0.get();
		if handers.is_none() {
			*handers = Some(HashMap::new());
		}
		handers.as_mut().unwrap_unchecked()
	}
}

pub type CallbackReturn = Pin<Box<dyn Future<Output = Result<()>> + Send>>;
pub type Callback = dyn FnOnce(Vec<u8>) -> CallbackReturn + Sync + Send + 'static;
pub(crate) struct CallbackItem {
	pub callback: Box<Callback>,
}

pub(crate) async fn add_callback<T>(seq_number: u64, callback: T) -> Result<()>
where
	T: FnOnce(Vec<u8>) -> CallbackReturn + Send + Sync + 'static,
{
	libp2p_handlers().insert(
		seq_number,
		CallbackItem {
			callback: Box::new(callback),
		},
	);
	Ok(())
}

pub async fn callback_reply(seq_number: u64, payload: Vec<u8>) -> Result<()> {
	let item = libp2p_handlers()
		.remove(&seq_number)
		.ok_or(Errors::Libp2pCallbackIsNone(seq_number))?;
	(item.callback)(payload).await
}

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
