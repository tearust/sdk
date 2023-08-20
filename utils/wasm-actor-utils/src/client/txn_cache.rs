use crate::client::error::Result;
use crate::client::help;
use crate::enclave::actors::kvp;
// use prost::Message;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;
use tea_runtime_codec::actor_txns::tsid::Tsid;
use tea_runtime_codec::tapp::Account;

pub const CACHE_TXN_KEY: &str = "cache_txn_key";

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TxnCacheItem {
	pub time: u128,
	pub from_actor: String,
	pub sender: Account,
	pub hash_hex: Option<String>,
	pub ts: Option<u128>,
	pub nonce: Option<u64>,
	pub txn_name: String,
	pub txn_bytes: Vec<u8>,
	pub txn_status: String,
	pub error: Option<String>,
	pub status: Option<String>,
}

impl TxnCacheItem {
	pub fn to_json(&self) -> serde_json::Value {
		let r = serde_json::json!({
			"id": self.time.to_string(),
			"sender": format!("{:?}", self.sender),
			"hash_hex": match &self.hash_hex {
				Some(t) => t.clone(),
				None => "".to_string(),
			},
			"txn_name": self.txn_name,
			"txn_args": match serde_json::from_slice(self.txn_bytes) {
				Ok(v) => v,
				Err(_) => serde_json::json!({}),
			}
			"txn_status": self.txn_status,
			"nonce": match &self.nonce {
				Some(t) => t.to_string(),
				None => "".to_string(),
			},
			"ts": match &self.ts {
				Some(t) => t.to_string(),
				None => "".to_string(),
			},
			"status": match &self.status {
				Some(s) => s.clone(),
				None => "".to_string(),
			},
			"error": match &self.error {
				Some(s) => s.clone(),
				None => "".to_string(),
			},
		});
		r
	}
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryTxnCacheListRequest {
	pub uuid: String,
	pub address: String,
	pub sender: Option<String>,
}

pub async fn query_txn_cache_list(payload: Vec<u8>, _from_actor: String) -> Result<Vec<u8>> {
	let req: QueryTxnCacheListRequest = serde_json::from_slice(&payload)?;
	info!("start query txn cache list...");

	let uuid = req.uuid;
	let cache_list = get_cache_instance().await?;
	let list: Vec<serde_json::Value> = if let Some(acct) = req.sender {
		cache_list
			.iter()
			.filter(|x| x.sender == acct.parse().unwrap())
			.map(|x| x.to_json())
			.collect()
	} else {
		cache_list.iter().map(|x| x.to_json()).collect()
	};
	let list_json = json!({
	  "list": &list,
	});
	help::cache_json_with_uuid(&uuid, list_json).await?;
	help::result_ok()
}

#[allow(clippy::too_many_arguments)]
pub async fn add_to_txn_cache(
	txn_name: String,
	payload: Vec<u8>,
	sender: String,
	from_actor: String,
) -> Result<&TxnCacheItem> {
	let cache_item = TxnCacheItem {
		time: crate::enclave::actors::env::system_time_as_nanos().await?,
		from_actor,
		hash_hex: None,
		ts: None,
		nonce: None,
		txn_name,
		txn_bytes: payload,
		txn_status: "Normal".into(),
		sender: sender.parse()?,
		status: None,
		error: None,
	};

	let mut cache_list = get_cache_instance().await?;
	cache_list.push(cache_item);

	if cache_list.len() > 500 {
		//TODO
		cache_list.pop();
	}
	set_cache_instance(cache_list).await?;

	Ok(&cache_item)
}

fn cache_key() -> String {
	format!("_{CACHE_TXN_KEY}_")
}

pub async fn get_cache_instance() -> Result<Vec<TxnCacheItem>> {
	let key = cache_key();

	let wrap_bytes = kvp::get(&key).await;
	if wrap_bytes.is_err() {
		return Ok(Vec::new());
	}
	let wrap_bytes = wrap_bytes.unwrap();
	if wrap_bytes.is_none() {
		return Ok(Vec::new());
	}
	let bytes: Vec<u8> = wrap_bytes.unwrap();
	let list: Vec<TxnCacheItem> = tea_codec::deserialize(bytes)?;
	Ok(list)
}

pub async fn set_cache_instance(list: Vec<TxnCacheItem>) -> Result<()> {
	let key = cache_key();
	let bytes = tea_codec::serialize(&list)?;
	kvp::set(&key, &bytes, 60 * 60 * 24 * 365).await?;
	Ok(())
}

pub async fn get_cache_item_by_ts(ts_str: &str) -> Result<Option<(Vec<TxnCacheItem>, usize)>> {
	let list = get_cache_instance().await?;
	let ts = u128::from_str(ts_str)?;
	let wrap_search = list.binary_search_by(|x| x.time.cmp(&ts));

	if wrap_search.is_err() {
		return Ok(None);
	}

	let index = wrap_search.unwrap();

	Ok(Some((list, index)))
}

pub async fn get_cache_item_by_hash(hash: &str) -> Result<Option<(Vec<TxnCacheItem>, usize)>> {
	let list = get_cache_instance().await?;

	let mut index: Option<usize> = None;
	for (i, x) in list.iter().enumerate() {
		if let Some(v_hash) = &x.hash_hex {
			if v_hash.eq_ignore_ascii_case(hash) {
				index = Some(i);
			}
		}
	}
	if index.is_none() {
		return Ok(None);
	}
	let index = index.unwrap();

	Ok(Some((list, index)))
}

pub async fn set_item_tsid(item: &TxnCacheItem, tsid: Tsid) -> Result<()> {
	let ts_str = item.time.to_string();
	let wrap = get_cache_item_by_ts(&ts_str).await?;
	if wrap.is_none() {
		return Ok(());
	}
	let (mut list, index) = wrap.unwrap();
	let mut item = list.get_mut(index).unwrap();
	item.hash_hex = Some(hex::encode(tsid.hash));
	item.ts = Some(tsid.ts);
	item.nonce = Some(tsid.nonce);

	info!("After set_item_tsid => {:?}", item);
	set_cache_instance(list).await?;

	Ok(())
}

pub async fn set_item_status(hash: &str, error: Option<&str>) -> Result<()> {
	let wrap = get_cache_item_by_hash(hash).await?;
	if wrap.is_none() {
		return Ok(());
	}
	let (mut list, index) = wrap.unwrap();
	let mut item = list.get_mut(index).unwrap();

	if let Some(err) = error {
		item.status = Some("Fail".into());
		item.error = Some(err.into());
	} else {
		item.status = Some("Success".into());
	}
	info!("After set_item_status => {:?}", item);
	set_cache_instance(list).await?;

	Ok(())
}
