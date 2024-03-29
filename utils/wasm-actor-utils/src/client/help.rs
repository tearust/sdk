use crate::client::{Errors, Result};
use crate::enclave::actors::kvp;
use serde::Serialize;
use tea_codec::OptionExt;
use tea_runtime_codec::tapp::Balance;

/// Set a cache value for 1800 senond.
pub async fn set_mem_cache(key: &str, val: Vec<u8>) -> Result<()> {
	kvp::set(key, &val, 1800).await?;
	Ok(())
}

pub async fn set_mem_cache_with_time(key: &str, val: Vec<u8>, time: i32) -> Result<()> {
	kvp::set(key, &val, time).await?;
	Ok(())
}

/// Return a cache value.
pub async fn get_mem_cache(key: &str) -> Result<Vec<u8>> {
	let rs: Vec<u8> = kvp::get(key).await?.ok_or_err_else(|| "")?;
	Ok(rs)
}

#[doc(hidden)]
pub async fn save_session_key(session_key: String, tapp_id_hex: &str, address: &str) -> Result<()> {
	let key = format!("session_key_{tapp_id_hex}_{address}");

	kvp::set(&key, &session_key, 3600 * 24).await?;

	Ok(())
}

#[doc(hidden)]
pub async fn get_session_key(tapp_id_hex: &str, address: &str) -> Result<String> {
	let key = format!("session_key_{tapp_id_hex}_{address}");

	let session_key: String = kvp::get(&key).await?.ok_or_err_else(|| "")?;

	Ok(session_key)
}

#[doc(hidden)]
pub async fn save_aes_key(aes_key: Vec<u8>, tapp_id_b64: &str) -> Result<()> {
	let key = format!("aes_key_{tapp_id_b64}");

	kvp::set(&key, &aes_key, 3600 * 24).await?;

	Ok(())
}

#[doc(hidden)]
pub async fn get_aes_key(tapp_id_b64: &str) -> Result<Vec<u8>> {
	let key = format!("aes_key_{tapp_id_b64}");

	let aes_key: Vec<u8> = kvp::get(&key).await?.ok_or_err("")?;

	Ok(aes_key)
}

/// Return a success json value.
pub fn result_ok() -> Result<Vec<u8>> {
	let json = serde_json::json!({
		"data": "ok",
		"status": true,
	});
	Ok(serde_json::to_vec(&json)?)
}

/// Return a custom error json value.
pub fn result_error(e: String) -> Result<Vec<u8>> {
	let json = serde_json::json!({
		"error": e,
	});
	Ok(serde_json::to_vec(&json)?)
}

/// Return a custom json value.
pub fn result_json(v: serde_json::Value) -> Result<Vec<u8>> {
	Ok(serde_json::to_vec(&v)?)
}

pub fn http_json<T>(v: T) -> Result<Vec<u8>>
where
	T: Serialize + Clone,
{
	let json = serde_json::json!({
		"status": true,
		"data": v
	});
	Ok(serde_json::to_vec(&json)?)
}

/// Transform a u128 value from u8 buffer.
pub fn u128_from_le_buffer(data: &[u8]) -> Result<u128> {
	const U128_LENGTH: usize = 16;

	if data.len() < U128_LENGTH {
		return Err(Errors::U128Length(U128_LENGTH).into());
	}

	let mut u128_buf = [0u8; U128_LENGTH];
	u128_buf.copy_from_slice(&data[0..U128_LENGTH]);
	Ok(u128::from_le_bytes(u128_buf))
}

/// Set a cache json with an uuid key.
pub async fn cache_json_with_uuid(uuid: &str, val: serde_json::Value) -> Result<()> {
	set_mem_cache(uuid, serde_json::to_vec(&val)?).await
}

/// Using to cache value for query request.
/// The end-user will return data from miner node directly if using this for a query.
/// The expired time is 10 minutes.
pub async fn set_query_cache(key: &str, val: serde_json::Value) -> Result<()> {
	let key = format!("cache_{key}");
	let val = serde_json::json!({ "cache": val });
	let new_val = serde_json::to_vec(&val)?;
	kvp::set(&key, &new_val, 600).await?;
	Ok(())
}

/// Return cache value if set query cache before.
pub async fn get_query_cache(key: &str) -> Result<Option<Vec<u8>>> {
	let key = format!("cache_{key}");
	let val: Vec<u8> = kvp::get(&key).await?.ok_or_err_else(|| "")?;
	if val.is_empty() {
		return Ok(None);
	}
	info!("query cache => {key}");
	Ok(Some(val))
}

/// Remove cache value
pub async fn remove_query_cache(key: &str) -> Result<()> {
	let key = format!("cache_{key}");
	kvp::del(&key).await?;
	Ok(())
}

pub fn string_to_balance(balance_str: &str) -> Result<Balance> {
	let balance = Balance::from_str_radix(&balance_str, 10)
		.map_err(|_| Errors::Unnamed(format!("Balance_string parse error.")))?;
	Ok(balance)
}
