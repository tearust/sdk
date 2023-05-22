use crate::enclave::error::Result;
use serde::{de::DeserializeOwned, Serialize};
use tea_actorx::ActorId;
use tea_codec::{deserialize, serialize, ResultExt};
use tea_system_actors::keyvalue::actions::*;

const KVP_ACTOR: ActorId = ActorId::Static(tea_system_actors::keyvalue::NAME);

pub struct ShabbyLock {
	key: String,
}

impl ShabbyLock {
	pub async fn lock(uid: &str) -> Self {
		// TODO: no need to lock for now
		Self {
			key: uid.to_string(),
		}
	}
}

impl Drop for ShabbyLock {
	fn drop(&mut self) {
		trace!("drop ShabbyLock");
		drop(del(&self.key));
	}
}

/// Set cache value to key-value actor and not expired.
pub async fn set_forever<T: Serialize + DeserializeOwned>(key: &str, value: &T) -> Result<T> {
	let req = SetRequest {
		key: key.to_owned(),
		value: serialize(value)?,
		expires_s: None,
	};
	let r = KVP_ACTOR.call(req).await?;
	deserialize(r.value.as_slice()).err_into()
}

/// Return cache value from key-value actor.
pub async fn get<T: DeserializeOwned>(key: &str) -> Result<Option<T>> {
	let req = GetRequest {
		key: key.to_owned(),
	};
	let r = KVP_ACTOR.call(req).await?;
	if r.exists {
		match r.value {
			Some(value) => {
				let result: T = deserialize(value)?;
				Ok(Some(result))
			}
			_ => Ok(None),
		}
	} else {
		Ok(None)
	}
}

/// Set cache value to key-value actor. can set expired second.
pub async fn set<T: Serialize + DeserializeOwned>(
	key: &str,
	value: &T,
	expires_s: i32,
) -> Result<T> {
	let req = SetRequest {
		key: key.to_owned(),
		value: serialize(value)?,
		expires_s: Some(expires_s),
	};
	let r = KVP_ACTOR.call(req).await?;
	deserialize(r.value.as_slice()).err_into()
}

/// Remove cache value from key-value actor.
pub async fn del(key: &str) -> Result<String> {
	let req = DelRequest {
		key: key.to_owned(),
	};
	let r = KVP_ACTOR.call(req).await?;
	Ok(r.key)
}

#[doc(hidden)]
pub async fn add(key: &str, value: i32) -> Result<i32> {
	let req = AddRequest {
		key: key.to_owned(),
		value,
	};
	let res = KVP_ACTOR.call(req).await?;
	Ok(res.value)
}

#[doc(hidden)]
pub async fn list_clear(key: &str) -> Result<String> {
	let req = ListClearRequest {
		key: key.to_owned(),
	};
	let res = KVP_ACTOR.call(req).await?;
	Ok(res.key)
}

#[doc(hidden)]
pub async fn list_range<T: Serialize + DeserializeOwned>(
	key: &str,
	start: i32,
	stop: i32,
) -> Result<Vec<T>> {
	let req = ListRangeRequest {
		key: key.to_owned(),
		start,
		stop,
	};
	let res = KVP_ACTOR.call(req).await?;
	let result: Vec<T> = res
		.values
		.into_iter()
		.map(|t| deserialize(t.as_slice()))
		.collect::<Result<_, _>>()?;
	Ok(result)
}

#[doc(hidden)]
pub async fn list_push<T: Serialize + DeserializeOwned>(key: &str, value: &T) -> Result<i32> {
	let req = ListPushRequest {
		key: key.to_owned(),
		value: serialize(value)?,
	};
	let res = KVP_ACTOR.call(req).await?;
	Ok(res.new_count)
}

#[doc(hidden)]
pub async fn list_del_item<T: Serialize>(key: &str, value: &T) -> Result<i32> {
	let req = ListDelItemRequest {
		key: key.to_owned(),
		value: serialize(value)?,
	};
	let res = KVP_ACTOR.call(req).await?;
	Ok(res.new_count)
}

#[doc(hidden)]
pub async fn set_add<T: Serialize>(key: &str, value: &T) -> Result<i32> {
	let req = SetAddRequest {
		key: key.to_owned(),
		value: serialize(value)?,
	};
	let res = KVP_ACTOR.call(req).await?;
	Ok(res.new_count)
}

#[doc(hidden)]
pub async fn set_remove<T: Serialize>(key: &str, value: &T) -> Result<i32> {
	let req = SetRemoveRequest {
		key: key.to_owned(),
		value: serialize(value)?,
	};
	let res = KVP_ACTOR.call(req).await?;
	Ok(res.new_count)
}

#[doc(hidden)]
pub async fn set_union<T: DeserializeOwned>(keys: Vec<&str>) -> Result<Vec<T>> {
	let keys: Vec<String> = keys.into_iter().map(|k| k.to_owned()).collect();
	let req = SetUnionRequest { keys };
	let res = KVP_ACTOR.call(req).await?;
	let result: Vec<T> = res
		.values
		.into_iter()
		.map(|t| deserialize(t.as_slice()))
		.collect::<Result<_, _>>()?;
	Ok(result)
}

#[doc(hidden)]
pub async fn set_intersect<T: DeserializeOwned>(keys: Vec<&str>) -> Result<Vec<T>> {
	let keys: Vec<String> = keys.into_iter().map(|k| k.to_owned()).collect();
	let req = SetIntersectionRequest { keys };
	let res = KVP_ACTOR.call(req).await?;
	let result: Vec<T> = res
		.values
		.into_iter()
		.map(|t| deserialize(t.as_slice()))
		.collect::<Result<_, _>>()?;
	Ok(result)
}

pub async fn set_query<T: DeserializeOwned>(key: &str) -> Result<Vec<T>> {
	let req = SetQueryRequest {
		key: key.to_owned(),
	};
	let res = KVP_ACTOR.call(req).await?;
	let result: Vec<T> = res
		.values
		.into_iter()
		.map(|t| deserialize(t.as_slice()))
		.collect::<Result<_, _>>()?;
	Ok(result)
}

pub async fn exists(key: &str) -> Result<bool> {
	let req = KeyExistsQuery {
		key: key.to_owned(),
	};
	let res = KVP_ACTOR.call(req).await?;
	Ok(res.exists)
}

#[doc(hidden)]
pub async fn keyvec_insert<T: Serialize>(
	key: &str,
	tuple: (i32, &T),
	overwrite: bool,
) -> Result<bool> {
	let t = TupleKeyValue {
		k: tuple.0,
		v: serialize(tuple.1)?,
	};
	let req = KeyVecInsertRequest {
		key: key.to_string(),
		value: Some(t),
		overwrite,
	};
	let res = KVP_ACTOR.call(req).await?;
	Ok(res.success)
}

#[doc(hidden)]
pub async fn keyvec_get<T: DeserializeOwned>(key: &str) -> Result<Vec<(i32, T)>> {
	let req = KeyVecGetRequest {
		key: key.to_string(),
	};

	let res = KVP_ACTOR.call(req).await?;
	let result: Vec<(i32, T)> = res
		.values
		.into_iter()
		.map(|t| (t.k, deserialize(t.v.as_slice()).unwrap()))
		.collect();
	Ok(result)
}

#[doc(hidden)]
pub async fn keyvec_remove_item(key: &str, value_idx: i32) -> Result<()> {
	let req = KeyVecRemoveItemRequest {
		key: key.to_string(),
		value_idx,
	};
	let _res = KVP_ACTOR.call(req).await?;
	Ok(())
}

#[doc(hidden)]
pub async fn keyvec_tail_off(key: &str, remain: usize) -> Result<usize> {
	let req = KeyVecTailOffRequest {
		key: key.to_string(),
		remain: remain as u32,
	};
	let res = KVP_ACTOR.call(req).await?;
	Ok(res.len as usize)
}
