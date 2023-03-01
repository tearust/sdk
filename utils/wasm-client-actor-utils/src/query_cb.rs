use crate::help;
use crate::Result;

pub async fn query_callback(_from_actor: String, key: &str) -> Result<serde_json::Value> {
	let value = help::get_mem_cache(key).await?;
	let rs: serde_json::Value = serde_json::from_slice(&value)?;
	Ok(rs)
}
