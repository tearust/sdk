use crate::client::api;
use crate::client::error::Result;
use crate::client::help;
use crate::client::request::uuid_cb_key;
pub use crate::enclave::action::HttpRequest;
use futures::Future;
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::{collections::HashMap, pin::Pin};
use tea_actorx2::ActorId;

use serde::{Deserialize, Serialize};
use tea_codec::serde::TypeId;

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(Vec<u8>)]
pub struct ClientTxnCbRequest {
	pub action: String,
	pub payload: Vec<u8>,
	pub from_actor: String,
	pub uuid: String,
}

type CBD = Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send>>;
pub type CallbackCB = dyn Fn(Vec<u8>, String) -> CBD + Sync + Send + 'static;

lazy_static! {
	pub static ref HANDLER_CB_MAP: Mutex<HashMap<String, Box<CallbackCB>>> =
		Mutex::new(HashMap::new());
}

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

pub async fn map_cb_handler(action: &str, _arg: Vec<u8>, _from_actor: String) -> Result<Vec<u8>> {
	let res = match action {
		_ => vec![],
	};
	Ok(res)
}

pub fn add_cb_handler<C>(action_name: &str, handler_cb: C) -> Result<()>
where
	C: Fn(Vec<u8>, String) -> CBD + Send + Sync + 'static,
{
	HANDLER_CB_MAP
		.lock()
		.unwrap()
		.insert(action_name.to_string(), Box::new(handler_cb));

	Ok(())
}

pub async fn txn_callback(uuid: &str, from_actor: String) -> Result<Vec<u8>> {
	info!("txn_callback => {:?}", uuid);
	let target_actor = Box::leak(from_actor.clone().into_boxed_str());
	let ori_uuid = str::replace(uuid, "hash_", "");
	let action_key = uuid_cb_key(&ori_uuid, "action_name");
	let req_key = uuid_cb_key(&ori_uuid, "action_req");

	let tmp = help::get_mem_cache(&action_key).await?;
	let action_name: String = tea_codec::deserialize(tmp)?;
	let req_bytes = help::get_mem_cache(&req_key).await?;

	let req = ClientTxnCbRequest {
		from_actor: from_actor,
		action: action_name,
		payload: req_bytes,
		uuid: ori_uuid,
	};
	let actor_id: ActorId = target_actor.as_bytes().into();
	let rs = actor_id.call(req).await?;
	Ok(rs)
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
