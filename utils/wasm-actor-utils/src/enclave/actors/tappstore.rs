use super::{
	libp2p::{connected_peers, intelli_actor_query_ex},
	replica::IntelliSendMode,
	util::random_select,
};
use crate::enclave::{
	actors::env::tappstore_id,
	error::{Error, Result},
};
use prost::Message;
use std::collections::HashSet;
use tea_actorx::ActorId;
use tea_codec::{deserialize, serialize};
use tea_runtime_codec::actor_txns::pre_args::{Arg, ArgSlots};
use tea_runtime_codec::tapp::{
	cml::CmlId,
	ra::{NodeStatus, TeaNodeProfile},
	statement::TypedStatement,
	version::SystemVersions,
	Account, Balance, ReplicaId, Ts,
};
use tea_runtime_codec::vmh::message::{
	encode_protobuf,
	structs_proto::{persist, tappstore, tokenstate},
};
use tea_system_actors::tappstore::*;
use tea_system_actors::tokenstate::HasDbInitRequest;

/// Simple system date formatter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SimpleDate {
	pub year: i32,
	pub month: u32,
	pub day: u32,
}

impl SimpleDate {
	pub fn new(year: i32, month: u32, day: u32) -> Self {
		Self { year, month, day }
	}
}

#[doc(hidden)]
pub async fn node_profiles_by_conn_ids(
	conn_ids: Vec<String>,
	mode: IntelliSendMode,
) -> Result<Vec<TeaNodeProfile>> {
	let res = intelli_actor_query_ex(
		tea_system_actors::tappstore::NAME,
		NodeProfileByConnIdsRequest(encode_protobuf(tappstore::QueryNodeProfilesByConnIds {
			conn_ids,
		})?),
		mode,
		None,
	)
	.await?;
	Ok(res.0)
}

#[doc(hidden)]
pub async fn has_tappstore_init() -> Result<bool> {
	let buf = ActorId::Static(tea_system_actors::tokenstate::NAME)
		.call(HasDbInitRequest(encode_protobuf(
			tokenstate::HasGlueDbInitRequest {
				token_id: serialize(&tappstore_id().await?)?,
			},
		)?))
		.await?;
	let res = tokenstate::HasGlueDbInitResponse::decode(buf.0.as_slice())?;
	Ok(res.has_init)
}

#[doc(hidden)]
pub async fn random_select_active_seats_locally(
	count: usize,
	exclude_tea_id: Option<Vec<u8>>,
) -> Result<Vec<(Vec<u8>, String)>> {
	let maintainers = ActorId::Static(NAME)
		.call(QueryActiveSeatsRequest(encode_protobuf(
			tappstore::QueryActiveSeats {
				has_exclude: exclude_tea_id.is_some(),
				exclude_tea_id: exclude_tea_id.unwrap_or_default(),
			},
		)?))
		.await?;
	let connect_peers: HashSet<String> = connected_peers().await?.into_iter().collect();
	let candidatas = random_select(
		maintainers
			.0
			.into_iter()
			.filter(|m| connect_peers.contains(&m.conn_id))
			.collect(),
		count,
	)
	.await?;
	Ok(candidatas
		.into_iter()
		.map(|p| (p.tea_id, p.conn_id))
		.collect())
}

/// Return an address's TEA balance.
/// It'll need the auth_key to check permission.
/// That means users in TEA system cannot see another user's balance.
pub async fn query_tea_balance_async(
	account: &str,
	auth_key: &[u8],
	mode: IntelliSendMode,
) -> Result<(Balance, Ts)> {
	let res = intelli_actor_query_ex(
		tea_system_actors::tappstore::NAME,
		QueryTeaBalanceRequest(encode_protobuf(tappstore::TeaBalanceRequest {
			account: account.to_string(),
			token_id: serialize(&tappstore_id().await?)?,
			auth_key: auth_key.to_vec(),
		})?),
		mode,
		None,
	)
	.await?;
	let res = tappstore::TeaBalanceResponse::decode(res.0.as_slice())?;
	Ok((deserialize(&res.balance)?, deserialize(&res.ts)?))
}

/// Return CML id from a miner tea_id.
pub async fn query_cml_id_by_tea_id(tea_ids: Vec<ReplicaId>) -> Result<Vec<CmlId>> {
	let res = ActorId::Static(NAME)
		.call(QueryCmlIdsByTeaIdsRequest(serialize(&tea_ids)?))
		.await?;
	Ok(res.0)
}

/// Return all active CML ids.
/// Active means a miner is binding the CML and active now.
pub async fn query_active_cml_ids(
	exclude_tea_id: Option<Vec<u8>>,
	mode: IntelliSendMode,
) -> Result<Vec<CmlId>> {
	let res = intelli_actor_query_ex(
		tea_system_actors::tappstore::NAME,
		QueryActiveCmlsRequest(encode_protobuf(tappstore::QueryActiveNodes {
			has_exclude: exclude_tea_id.is_some(),
			exclude_tea_id: exclude_tea_id.unwrap_or_default(),
		})?),
		mode,
		None,
	)
	.await?;
	Ok(res.0)
}

/// Return all mining CML ids.
/// Mining means a miner is binding the CML to a machine.
pub async fn query_mining_cml_ids(mode: IntelliSendMode) -> Result<Vec<CmlId>> {
	let res = intelli_actor_query_ex(
		tea_system_actors::tappstore::NAME,
		QueryMiningCmlIdsRequest,
		mode,
		None,
	)
	.await?;
	Ok(res.0)
}

/// Return the basic state transaction logs.
pub async fn get_statements_async(
	account: Option<Account>,
	query_date: Option<SimpleDate>,
	mode: IntelliSendMode,
) -> Result<(Vec<(TypedStatement, String, String)>, bool)> {
	// this max size is hard coded according to `MAX_PROTOCOL_SIZE` defined in libp2p general request
	const MAX_SIZE: u64 = 1024 * 1024;

	let res = intelli_actor_query_ex(
		tea_system_actors::tappstore::NAME,
		GetStatementsRequest(encode_protobuf(persist::GetStatements {
			max_size: MAX_SIZE,
			account_filter: account
				.map(|v| {
					Ok::<_, Error>(persist::GetStatementsAccount {
						account: serialize(&v)?,
					})
				})
				.transpose()?,
			date: query_date.map(|v| persist::GetStatementsDatetime {
				year: v.year,
				month: v.month,
				day: v.day,
			}),
		})?),
		mode,
		None,
	)
	.await?;
	Ok((res.0, res.1))
}

#[doc(hidden)]
pub async fn query_cml_ra_status(tea_id: &[u8], mode: IntelliSendMode) -> Result<NodeStatus> {
	let res = intelli_actor_query_ex(
		tea_system_actors::tappstore::NAME,
		QueryCmlRaStatusRequest(tea_id.to_vec()),
		mode,
		None,
	)
	.await?;
	Ok(res.0)
}

/// Return all active node profiles.
pub async fn query_active_nodes(
	exclude_tea_id: Option<Vec<u8>>,
	mode: IntelliSendMode,
) -> Result<Vec<TeaNodeProfile>> {
	let res = intelli_actor_query_ex(
		tea_system_actors::tappstore::NAME,
		QueryActiveNodesRequest(encode_protobuf(tappstore::QueryActiveNodes {
			has_exclude: exclude_tea_id.is_some(),
			exclude_tea_id: exclude_tea_id.unwrap_or_default(),
		})?),
		mode,
		None,
	)
	.await?;
	Ok(res.0)
}

/// Request all arg related values locally
pub async fn process_pre_args(pre_args: Vec<Arg>) -> Result<Option<ArgSlots>> {
	if pre_args.is_empty() {
		return Ok(None);
	}

	let res = intelli_actor_query_ex(
		tea_system_actors::tappstore::NAME,
		ProcessPreArgsRequest(pre_args),
		IntelliSendMode::LocalOnly,
		None,
	)
	.await?;
	Ok(Some(res.0))
}

/// Return current runtime version
pub async fn query_system_versions(mode: IntelliSendMode) -> Result<SystemVersions> {
	let res = intelli_actor_query_ex(
		tea_system_actors::tappstore::NAME,
		QuerySystemVersionsRequest,
		mode,
		None,
	)
	.await?;
	Ok(res.0)
}
