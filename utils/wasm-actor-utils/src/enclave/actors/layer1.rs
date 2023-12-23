use crate::enclave::error::Result;
#[cfg(feature = "__test")]
use mocktopus::macros::*;
use tea_codec::OptionExt;
use tea_runtime_codec::tapp::{
	cml::{CmlId, CmlIntrinsic},
	seat::SeatId,
	Account, BlockNumber,
};
#[cfg(not(feature = "__test"))]
use {tea_actorx::ActorId, tea_system_actors::layer1::*};

#[cfg(feature = "__test")]
#[mockable]
pub async fn get_mining_startup_nodes() -> Result<Vec<(Vec<u8>, SeatId, String)>> {
	Ok(Default::default())
}

#[cfg(not(feature = "__test"))]
pub async fn get_mining_startup_nodes() -> Result<Vec<(Vec<u8>, SeatId, String)>> {
	let rtn = ActorId::Static(tea_system_actors::env::NAME)
		.call(tea_system_actors::env::GetMiningStartupRequest)
		.await?;
	Ok(rtn
		.0
		.into_iter()
		.map(|(tea_id, seat_id, ip)| (tea_id.to_vec(), seat_id, ip))
		.collect())
}

pub async fn get_first_mining_startup_node() -> Result<(Vec<u8>, SeatId, String)> {
	let nodes = get_mining_startup_nodes().await?;
	Ok(nodes
		.first()
		.ok_or_err("first mining startup node")?
		.clone())
}

#[cfg(not(feature = "__test"))]
pub async fn is_first_startup_node(tea_id: &[u8]) -> Result<bool> {
	let (startup_tea_id, _, _) = get_first_mining_startup_node().await?;
	Ok(startup_tea_id.eq(tea_id))
}

#[cfg(feature = "__test")]
#[allow(clippy::forget_copy, clippy::forget_ref, clippy::swap_ptr_to_ref)]
#[mockable]
pub async fn is_first_startup_node(_tea_id: &[u8]) -> Result<bool> {
	Ok(true)
}

#[cfg(feature = "__test")]
#[allow(clippy::forget_copy, clippy::forget_ref, clippy::swap_ptr_to_ref)]
#[mockable]
pub async fn get_tapp_startup_nodes(
	_at_height: Option<BlockNumber>,
) -> Result<Vec<(Vec<u8>, CmlId, String)>> {
	Ok(vec![])
}

#[cfg(not(feature = "__test"))]
pub async fn get_tapp_startup_nodes(
	at_height: Option<BlockNumber>,
) -> Result<Vec<(Vec<u8>, CmlId, String)>> {
	use tea_runtime_codec::solc::queries::AsyncQuery;

	if at_height.is_none() {
		// only try get cache if at_height is none
		let cached_startup = ActorId::Static(NAME)
			.call(TappStartupNodesFromCacheRequest)
			.await?;
		if let Some(cached_startup) = cached_startup.0 {
			return Ok(cached_startup);
		}
	}

	let nodes = ActorId::Static(NAME)
		.call(TappStartupNodesRequest(AsyncQuery {
			at_height,
			..Default::default()
		}))
		.await?;
	let startup_nodes = complete_stateup_nodes_info(nodes.0);
	if let Ok(n) = startup_nodes.as_ref() {
		ActorId::Static(NAME)
			.call(UpdateTappStartupNodesRequest(n.clone()))
			.await?;
	}
	startup_nodes
}

#[cfg(not(feature = "__test"))]
fn complete_stateup_nodes_info(
	startup_nodes: Vec<(CmlId, String)>,
) -> Result<Vec<(Vec<u8>, CmlId, String)>> {
	Ok(startup_nodes
		.into_iter()
		.enumerate()
		.map(|(i, (cml_id, ip))| {
			(
				// startup tea_id generated automatically by the node index
				primitive_types::H256::from_low_u64_be(i as u64)
					.to_fixed_bytes()
					.to_vec(),
				cml_id,
				ip,
			)
		})
		.collect())
}

#[cfg(feature = "__test")]
#[allow(clippy::forget_copy)]
#[mockable]
pub async fn cmls_info_from_layer1(
	_cml_ids: Vec<CmlId>,
	_at_height: Option<BlockNumber>,
) -> Result<Vec<CmlIntrinsic>> {
	Ok(vec![])
}

/// Returned result items count and order is not parented to matched with `cml_ids`
#[cfg(not(feature = "__test"))]
pub async fn cmls_info_from_layer1(
	cml_ids: Vec<CmlId>,
	at_height: Option<BlockNumber>,
) -> Result<Vec<CmlIntrinsic>> {
	use tea_runtime_codec::solc::queries::{AsyncQuery, QueryType};

	let (cached_cmls, missing_cml_ids): (Vec<CmlIntrinsic>, Vec<CmlId>) = if at_height.is_none() {
		let (cached_cmls, missing_cml_ids) = get_cached_cmls(&cml_ids).await?;
		if missing_cml_ids.is_empty() {
			return Ok(cached_cmls);
		}
		(cached_cmls, missing_cml_ids)
	} else {
		(vec![], cml_ids)
	};

	let mut cmls = ActorId::Static(NAME)
		.call(GetCmlInfoRequest(AsyncQuery {
			at_height,
			query_type: QueryType::CmlInfo(missing_cml_ids),
		}))
		.await?;
	update_cml_cache(&cmls.0).await?;
	cmls.0.extend(cached_cmls);
	Ok(cmls.0)
}

#[cfg(not(feature = "__test"))]
async fn get_cached_cmls(cml_ids: &[CmlId]) -> Result<(Vec<CmlIntrinsic>, Vec<CmlId>)> {
	// only try get cache if at_height is none

	let mut cached_cmls = vec![];
	let mut missing_cml_ids = vec![];
	for id in cml_ids {
		let cached_cml_info = ActorId::Static(NAME)
			.call(CmlInfoFromCacheRequest(*id))
			.await?;
		if let Some(info) = cached_cml_info.0 {
			cached_cmls.push(info);
		} else {
			missing_cml_ids.push(*id);
		}
	}

	Ok((cached_cmls, missing_cml_ids))
}

#[cfg(not(feature = "__test"))]
async fn update_cml_cache(cmls: &[CmlIntrinsic]) -> Result<()> {
	ActorId::Static(NAME)
		.call(UpdateCmlInfoRequest(cmls.to_vec()))
		.await?;
	Ok(())
}

#[cfg(not(feature = "__test"))]
pub async fn appstore_owner_account(at_height: Option<BlockNumber>) -> Result<Account> {
	use tea_runtime_codec::solc::queries::AsyncQuery;

	let res = ActorId::Static(NAME)
		.call(TappstoreOwnerRequest(AsyncQuery {
			at_height,
			query_type: Default::default(),
		}))
		.await?;
	Ok(res.0)
}

#[cfg(feature = "__test")]
#[allow(clippy::forget_copy)]
#[mockable]
pub async fn appstore_owner_account(_at_height: Option<BlockNumber>) -> Result<Account> {
	Ok(Default::default())
}
