use crate::client::api;
use crate::client::error::Result;
use crate::client::txn_cache;
pub use crate::enclave::action::HttpRequest;
use futures::Future;
use std::pin::Pin;

type CBD = Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send>>;
pub type CallbackCB = dyn Fn(Vec<u8>, String) -> CBD + Sync + Send + 'static;

#[doc(hidden)]
pub async fn map_handler(action: &str, arg: Vec<u8>, from_actor: String) -> Result<Vec<u8>> {
	let res = match action {
		"login" => api::user::txn_login(arg, from_actor).await?,
		"query_session_key" => api::user::query_session_key(arg, from_actor).await?,
		"query_result" => api::user::query_result(arg, from_actor).await?,
		"queryHashResult" => api::user::query_txn_hash_result(arg, from_actor).await?,
		"queryHashResultFromAll" => {
			api::user::query_txn_hash_result_from_all(arg, from_actor).await?
		}
		"logout" => api::user::txn_logout(arg, from_actor)?,
		"query_balance" => api::user::query_balance(arg, from_actor).await?,
		"query_deposit" => api::user::query_deposit(arg, from_actor).await?,
		"query_credit" => api::user::query_credit(arg, from_actor).await?,
		"query_asset" => api::user::query_asset(arg, from_actor).await?,
		"query_allowance" => api::user::query_allowance(arg, from_actor).await?,
		"query_tapp_metadata" => api::user::query_tapp_metadata(arg, from_actor).await?,
		"query_error_log" => api::user::query_error_log(arg, from_actor).await?,
		"query_system_version" => api::user::query_system_version(arg, from_actor).await?,
		"query_multi_tapp_allowance_from_local_state" => {
			api::user::query_multi_tapp_allowance(arg, from_actor).await?
		}
		"query_txn_cache_list" => txn_cache::query_txn_cache_list(arg, from_actor).await?,
		"query_credit_system_info" => api::user::query_credit_system_info(arg, from_actor).await?,

		"open_payment_channel" => api::channel::open_payment_channel(arg, from_actor).await?,
		"payer_early_terminate" => api::channel::early_terminate(arg, from_actor).await?,
		"terminate" => api::channel::terminate(arg, from_actor).await?,
		"payer_refill_fund" => api::channel::refill_fund(arg, from_actor).await?,
		"query_channel_list_with_account" => {
			api::channel::query_channel_list_with_account(arg, from_actor).await?
		}
		"payee_update_payment" => api::channel::payee_update_payment(arg, from_actor).await?,

		_ => vec![],
	};
	Ok(res)
}

#[doc(hidden)]
pub async fn map_cb_handler(action: &str, _arg: Vec<u8>, _from_actor: String) -> Result<Vec<u8>> {
	let res = match action {
		_ => vec![],
	};
	Ok(res)
}

#[doc(hidden)]
pub async fn txn_callback(_uuid: &str, _from_actor: String) -> Result<Vec<u8>> {
	// info!("txn_callback => {:?}", uuid);
	// let target_actor = Box::leak(from_actor.clone().into_boxed_str());
	// let ori_uuid = str::replace(uuid, "hash_", "");
	// let action_key = uuid_cb_key(&ori_uuid, "action_name");
	// let req_key = uuid_cb_key(&ori_uuid, "action_req");

	// let tmp = help::get_mem_cache(&action_key).await?;
	// let action_name: String = tea_codec::deserialize(tmp)?;
	// let req_bytes = help::get_mem_cache(&req_key).await?;

	// let req = ClientTxnCbRequest {
	// 	from_actor: from_actor,
	// 	action: action_name,
	// 	payload: req_bytes,
	// 	uuid: ori_uuid,
	// };
	// let actor_id = target_actor.as_bytes().to_vec().into_actor();
	// let rs = actor_id.call(req).await?;
	Ok(vec![])
}

#[doc(hidden)]
pub fn map_fn_list() -> Vec<&'static str> {
	vec![
		"login",
		"query_session_key",
		"query_result",
		"queryHashResult",
		"queryHashResultFromAll",
		"logout",
		"query_balance",
		"query_deposit",
		"query_credit",
		"query_asset",
		"query_allowance",
		"query_tapp_metadata",
		"query_error_log",
		"query_system_version",
		"query_multi_tapp_allowance_from_local_state",
		"query_txn_cache_list",
		"query_credit_system_info",
		"open_payment_channel",
		"payer_early_terminate",
		"terminate",
		"payer_refill_fund",
		"query_channel_list_with_account",
		"payee_update_payment",
	]
}
