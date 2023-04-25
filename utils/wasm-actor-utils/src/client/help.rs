use crate::client::{Errors, Result};
use crate::enclave::actors::kvp;
use tea_codec::OptionExt;

pub async fn set_mem_cache(key: &str, val: Vec<u8>) -> Result<()> {
	kvp::set(key, &val, 1800).await?;
	Ok(())
}

pub async fn get_mem_cache(key: &str) -> Result<Vec<u8>> {
	let rs: Vec<u8> = kvp::get(key).await?.ok_or_err_else(|| "")?;
	Ok(rs)
}

pub async fn save_session_key(session_key: String, tapp_id_hex: &str, address: &str) -> Result<()> {
	let key = format!("session_key_{tapp_id_hex}_{address}");

	kvp::set(&key, &session_key, 1800).await?;

	Ok(())
}
pub async fn get_session_key(tapp_id_hex: &str, address: &str) -> Result<String> {
	let key = format!("session_key_{tapp_id_hex}_{address}");

	let session_key: String = kvp::get(&key).await?.ok_or_err_else(|| "")?;

	Ok(session_key)
}

pub async fn save_aes_key(aes_key: Vec<u8>, tapp_id_b64: &str) -> Result<()> {
	let key = format!("aes_key_{tapp_id_b64}");

	kvp::set_forever(&key, &aes_key).await?;

	Ok(())
}
pub async fn get_aes_key(tapp_id_b64: &str) -> Result<Vec<u8>> {
	let key = format!("aes_key_{tapp_id_b64}");

	let aes_key: Vec<u8> = kvp::get(&key).await?.ok_or_err("")?;

	Ok(aes_key)
}

pub fn result_ok() -> Result<Vec<u8>> {
	let json = serde_json::json!({
		"data": "ok",
		"status": true,
	});
	Ok(serde_json::to_vec(&json)?)
}
pub fn result_error(e: String) -> Result<Vec<u8>> {
	let json = serde_json::json!({
		"error": e,
	});
	Ok(serde_json::to_vec(&json)?)
}
pub fn result_json(v: serde_json::Value) -> Result<Vec<u8>> {
	Ok(serde_json::to_vec(&v)?)
}

pub fn u128_from_le_buffer(data: &[u8]) -> Result<u128> {
	const U128_LENGTH: usize = 16;

	if data.len() < U128_LENGTH {
		return Err(Errors::U128Length(U128_LENGTH).into());
	}

	let mut u128_buf = [0u8; U128_LENGTH];
	u128_buf.copy_from_slice(&data[0..U128_LENGTH]);
	Ok(u128::from_le_bytes(u128_buf))
}

pub async fn cache_json_with_uuid(uuid: &str, val: serde_json::Value) -> Result<()> {
	set_mem_cache(uuid, serde_json::to_vec(&val)?).await
}

pub async fn set_query_cache(key: &str, val: serde_json::Value) -> Result<()> {
	let key = format!("cache_{key}");
	let val = serde_json::json!({ "cache": val });
	let new_val = serde_json::to_vec(&val)?;
	kvp::set(&key, &new_val, 900).await?;
	Ok(())
}
pub async fn get_query_cache(key: &str) -> Result<Option<Vec<u8>>> {
	let key = format!("cache_{key}");
	let val: Vec<u8> = kvp::get(&key).await?.ok_or_err_else(|| "")?;
	if val.is_empty() {
		return Ok(None);
	}
	info!("query cache => {key}");
	Ok(Some(val))
}
