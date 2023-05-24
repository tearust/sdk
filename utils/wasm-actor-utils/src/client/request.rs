pub use crate::enclave::actors::http;
use crate::enclave::actors::{
	libp2p::intelli_actor_query_ex,
	replica::{intelli_send_txn, IntelliSendMode},
};
use std::str::FromStr;
use tea_codec::{
	serde::{handle::Request, FromBytes, ToBytes},
	serialize,
};
use tea_runtime_codec::actor_txns::pre_args::Arg;
use tea_system_actors::tappstore::txns::TappstoreTxn;

use self::http::RequestExt;
use crate::client::help;
use crate::client::Result;

/// Send query to state node via libp2p
/// Can set a custom target actor.
pub async fn send_custom_query<C>(
	_from_actor: &str,
	arg: C,
	target: &'static [u8],
) -> Result<C::Response>
where
	C: Request + ToBytes + Clone,
	C::Response: for<'a> FromBytes<'a> + Send,
{
	Ok(intelli_actor_query_ex(target, arg, IntelliSendMode::RemoteOnly).await?)
}

/// Send query to tappstore actor.
pub async fn send_tappstore_query<C>(from_actor: &str, arg: C) -> Result<C::Response>
where
	C: Request + ToBytes + Clone,
	C::Response: for<'a> FromBytes<'a> + Send,
{
	send_custom_query(from_actor, arg, tea_system_actors::tappstore::NAME).await
}

/// Send txn to state node via libp2p
/// Can set a custom target actor.
pub async fn send_custom_txn(
	from_actor: &str,
	action_name: &str,
	uuid: &str,
	req_bytes: Vec<u8>,
	txn_bytes: Vec<u8>,
	pre_args: Vec<Arg>,
	target: &[u8],
) -> Result<()> {
	info!(
		"Send custom txn from {:?} to {:?} => {:?}",
		from_actor,
		String::from_utf8(target.to_vec())?,
		action_name
	);
	let ori_uuid = str::replace(uuid, "txn_", "");
	let action_key = uuid_cb_key(&ori_uuid, "action_name");
	let req_key = uuid_cb_key(&ori_uuid, "action_req");
	help::set_mem_cache(&action_key, tea_codec::serialize(&action_name)?).await?;
	help::set_mem_cache(&req_key, req_bytes).await?;

	let uuid = uuid.to_string();

	let gas_limit = crate::client::CLIENT_DEFAULT_GAS_LIMIT;
	let rtn = intelli_send_txn(
		target,
		&txn_bytes,
		pre_args,
		IntelliSendMode::RemoteOnly,
		gas_limit,
	)
	.await?;

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
}

/// Send a txn to tappstore-actor.
pub async fn send_tappstore_txn(
	from_actor: &str,
	action_name: &str,
	uuid: &str,
	req_bytes: Vec<u8>,
	txn: TappstoreTxn,
	pre_args: Vec<Arg>,
) -> Result<()> {
	send_custom_txn(
		from_actor,
		action_name,
		uuid,
		req_bytes,
		serialize(&txn)?,
		pre_args,
		tea_system_actors::tappstore::NAME,
	)
	.await
}

#[doc(hidden)]
pub fn uuid_cb_key(uuid: &str, stype: &str) -> String {
	let rs = format!("{stype}_msg_{uuid}");
	rs
}

#[doc(hidden)]
pub fn cb_key_to_uuid(key: &str, stype: &str) -> String {
	let ss = format!("{stype}_msg_");

	str::replace(key, &ss, "")
}

#[doc(hidden)]
pub async fn http_get(
	url: &str,
	headers_vec: Option<Vec<(String, String)>>,
) -> Result<serde_json::Value> {
	let mut builder = http::Request::builder().method("GET").uri(url);
	let headers = builder.headers_mut().unwrap();
	if headers_vec.is_some() {
		for (key, val) in headers_vec.unwrap() {
			headers.insert(
				http::HeaderName::from_str(&key)?,
				http::HeaderValue::from_str(&val)?,
			);
		}
	}
	let res = builder.request::<serde_json::Value>().await?;
	Ok(res.into_body())
}
