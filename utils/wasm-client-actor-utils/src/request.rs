use actor_txns::pre_args::Arg;
use tappstore_actor_codec::txns::TappstoreTxn;
use tea_codec::{
	serde::{handle::Request, FromBytes, ToBytes},
	serialize, ResultExt,
};
pub use wasm_actor_utils::actors::http;
use wasm_actor_utils::{
	action::CallbackReturn,
	actors::{
		libp2p::intelli_actor_query_ex,
		replica::{intelli_send_txn, IntelliSendMode},
	},
};

use crate::help;
use crate::Result;

pub async fn send_tappstore_query<C, T>(_from_actor: &str, arg: C, callback: T) -> Result<()>
where
	C: Request + ToBytes + Clone,
	C::Response: for<'a> FromBytes<'a> + Send,
	T: FnOnce(C::Response) -> CallbackReturn + Clone + Send + Sync + 'static,
{
	Ok(intelli_actor_query_ex(
		tappstore_actor_codec::NAME,
		arg,
		IntelliSendMode::RemoteOnly,
		callback,
	)
	.await?)
}

pub async fn send_tappstore_txn(
	_from_actor: &str,
	action_name: &str,
	uuid: &str,
	req_bytes: Vec<u8>,
	txn: TappstoreTxn,
	pre_args: Vec<Arg>,
	gas_limit: Option<u64>,
) -> Result<()> {
	let ori_uuid = str::replace(uuid, "txn_", "");
	let action_key = uuid_cb_key(&ori_uuid, "action_name");
	let req_key = uuid_cb_key(&ori_uuid, "action_req");
	help::set_mem_cache(&action_key, tea_codec::serialize(&action_name)?).await?;
	help::set_mem_cache(&req_key, req_bytes).await?;

	let uuid = uuid.to_string();

	let gas_limit = gas_limit.unwrap_or(crate::CLIENT_DEFAULT_GAS_LIMIT);

	intelli_send_txn(
		tappstore_actor_codec::NAME,
		&serialize(&txn)?,
		pre_args,
		IntelliSendMode::RemoteOnly,
		gas_limit,
		|rtn| {
			Box::pin(async move {
				if let Some(tsid) = rtn {
					info!("txn command successfully, tsid is: {:?}", tsid);

					let x = serde_json::json!({
						"ts": &tsid.ts.to_string(),
						"hash": hex::encode(tsid.hash),
						"sender": hex::encode(tsid.sender),
						"uuid": uuid,
					});
					help::cache_json_with_uuid(&uuid, x).await?;
				}

				Ok(())
			})
		},
	)
	.await
	.err_into()
}

pub fn uuid_cb_key(uuid: &str, stype: &str) -> String {
	let rs = format!("{stype}_msg_{uuid}");
	rs
}
pub fn cb_key_to_uuid(key: &str, stype: &str) -> String {
	let ss = format!("{stype}_msg_");

	str::replace(key, &ss, "")
}
