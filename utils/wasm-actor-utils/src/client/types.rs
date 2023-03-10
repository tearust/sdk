use crate::client::api;
use crate::client::error::Result;
pub use crate::enclave::action::HttpRequest;

pub type Callback = dyn Fn(Vec<u8>, String) -> Result<Vec<u8>> + Sync + Send + 'static;
pub async fn map_handler(action: &str, arg: Vec<u8>, from_actor: String) -> Result<Vec<u8>> {
	let res = match action {
		"login" => api::user::txn_login(arg, from_actor).await?,
		"query_session_key" => api::user::query_session_key(arg, from_actor).await?,
		"query_result" => api::user::query_result(arg, from_actor).await?,
		"queryHashResult" => api::user::query_txn_hash_result(arg, from_actor).await?,
		"logout" => api::user::txn_logout(arg, from_actor)?,
		"query_balance" => api::user::query_balance(arg, from_actor).await?,
		"query_deposit" => api::user::query_deposit(arg, from_actor).await?,
		"query_asset" => api::user::query_asset(arg, from_actor).await?,
		"query_allowance" => api::user::query_allowance(arg, from_actor).await?,
		"query_tapp_metadata" => api::user::query_tapp_metadata(arg, from_actor).await?,
		"query_error_log" => api::user::query_error_log(arg, from_actor).await?,
		"query_system_version" => api::user::query_system_version(arg, from_actor).await?,

		_ => vec![],
	};
	Ok(res)
}

pub fn map_fn_list() -> Vec<&'static str> {
	vec![
		"login",
		"query_session_key",
		"query_result",
		"queryHashResult",
		"logout",
		"query_balance",
		"query_deposit",
		"query_asset",
		"query_allowance",
		"query_tapp_metadata",
		"query_error_log",
		"query_system_version",
	]
}
