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
use tea_system_actors::layer1::*;

#[cfg(feature = "__test")]
#[mockable]
pub async fn get_mining_startup_nodes() -> Result<Vec<(Vec<u8>, SeatId, String)>> {
	Ok(Default::default())
}

#[cfg(not(feature = "__test"))]
pub async fn get_mining_startup_nodes() -> Result<Vec<(Vec<u8>, SeatId, String, String)>> {
	use tea_actorx_core::RegId;
	use tea_actorx_runtime::call;

	let rtn = call(
		RegId::Static(tea_system_actors::env::NAME).inst(0),
		tea_system_actors::env::GetMiningStartupRequest,
	)
	.await?;
	Ok(rtn
		.0
		.into_iter()
		.map(|(tea_id, seat_id, ip, key)| (tea_id.to_vec(), seat_id, ip, key))
		.collect())
}

pub async fn get_first_mining_startup_node() -> Result<(Vec<u8>, SeatId, String, String)> {
	let nodes = get_mining_startup_nodes().await?;
	Ok(nodes
		.first()
		.ok_or_err("first mining startup node")?
		.clone())
}

#[cfg(not(feature = "__test"))]
pub async fn is_first_startup_node(tea_id: &[u8]) -> Result<bool> {
	let (startup_tea_id, _, _, _) = get_first_mining_startup_node().await?;
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
pub async fn get_tapp_startup_nodes() -> Result<Vec<(Vec<u8>, CmlId, String)>> {
	Ok(vec![])
}

#[cfg(not(feature = "__test"))]
pub async fn get_tapp_startup_nodes() -> Result<Vec<(Vec<u8>, CmlId, String)>> {
	use super::env::get_delegate_startup_nodes;

	let nodes = get_delegate_startup_nodes().await?;
	let mut result = vec![];
	for item in nodes {
		result.push((hex::decode(item.machine_id)?, item.delegate_id, item.ip))
	}
	Ok(result)
}

#[cfg(not(feature = "__test"))]
pub async fn appstore_owner_account(
	key: String,
	at_height: Option<BlockNumber>,
) -> Result<Account> {
	use tea_actorx_core::RegId;
	use tea_actorx_runtime::call;
	use tea_runtime_codec::solc::queries::AsyncQuery;

	let res = call(
		RegId::Static(NAME).inst(0),
		TappstoreOwnerRequest(
			key,
			AsyncQuery {
				at_height,
				query_type: Default::default(),
			},
		),
	)
	.await?;
	Ok(res.0)
}

#[cfg(feature = "__test")]
#[allow(clippy::forget_copy)]
#[mockable]
pub async fn appstore_owner_account(
	_key: String,
	_at_height: Option<BlockNumber>,
) -> Result<Account> {
	Ok(Default::default())
}
