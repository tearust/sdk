use super::{
	libp2p::{connected_peers, intelli_actor_query_ex},
	replica::IntelliSendMode,
	util::random_select,
};
use crate::enclave::{
	action::CallbackReturn,
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
	Account, Balance, ReplicaId, TokenId, Ts,
};
use tea_runtime_codec::vmh::message::{
	encode_protobuf,
	structs_proto::{persist, tappstore, tokenstate},
};
use tea_system_actors::tappstore::*;
use tea_system_actors::tokenstate::HasDbInitRequest;

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

pub async fn node_profiles_by_conn_ids<T>(
	conn_ids: Vec<String>,
	mode: IntelliSendMode,
	callback: T,
) -> Result<()>
where
	T: FnOnce(Vec<TeaNodeProfile>) -> CallbackReturn + Clone + Send + Sync + 'static,
{
	intelli_actor_query_ex(
		tea_system_actors::tappstore::NAME,
		NodeProfileByConnIdsRequest(encode_protobuf(tappstore::QueryNodeProfilesByConnIds {
			conn_ids,
		})?),
		mode,
		|res| Box::pin(async move { callback(res.0).await }),
	)
	.await
}

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

pub async fn query_tea_balance_async<T>(
	account: &str,
	auth_key: &[u8],
	mode: IntelliSendMode,
	callback: T,
) -> Result<()>
where
	T: FnOnce(Balance, Ts) -> CallbackReturn + Clone + Send + Sync + 'static,
{
	intelli_actor_query_ex(
		tea_system_actors::tappstore::NAME,
		QueryTeaBalanceRequest(encode_protobuf(tappstore::TeaBalanceRequest {
			account: account.to_string(),
			token_id: serialize(&tappstore_id().await?)?,
			auth_key: auth_key.to_vec(),
		})?),
		mode,
		|res| {
			Box::pin(async move {
				let res = tappstore::TeaBalanceResponse::decode(res.0.as_slice())?;
				callback(deserialize(&res.balance)?, deserialize(&res.ts)?).await
			})
		},
	)
	.await
}

pub async fn query_cml_id_by_tea_id(tea_ids: Vec<ReplicaId>) -> Result<Vec<CmlId>> {
	let res = ActorId::Static(NAME)
		.call(QueryCmlIdsByTeaIdsRequest(serialize(&tea_ids)?))
		.await?;
	Ok(res.0)
}

pub async fn query_active_cml_ids<T>(
	exclude_tea_id: Option<Vec<u8>>,
	mode: IntelliSendMode,
	callback: T,
) -> Result<()>
where
	T: FnOnce(Vec<CmlId>) -> CallbackReturn + Clone + Send + Sync + 'static,
{
	intelli_actor_query_ex(
		tea_system_actors::tappstore::NAME,
		QueryActiveCmlsRequest(encode_protobuf(tappstore::QueryActiveNodes {
			has_exclude: exclude_tea_id.is_some(),
			exclude_tea_id: exclude_tea_id.unwrap_or_default(),
		})?),
		mode,
		|res| Box::pin(async move { callback(res.0).await }),
	)
	.await
}

pub async fn query_mining_cml_ids<T>(mode: IntelliSendMode, callback: T) -> Result<()>
where
	T: FnOnce(Vec<CmlId>) -> CallbackReturn + Clone + Send + Sync + 'static,
{
	intelli_actor_query_ex(
		tea_system_actors::tappstore::NAME,
		QueryMiningCmlIdsRequest,
		mode,
		|res| Box::pin(async move { callback(res.0).await }),
	)
	.await
}

pub async fn query_hosting_cml_ids<T>(
	token_id: TokenId,
	active_only: bool,
	mode: IntelliSendMode,
	callback: T,
) -> Result<()>
where
	T: FnOnce(Vec<CmlId>) -> CallbackReturn + Clone + Send + Sync + 'static,
{
	intelli_actor_query_ex(
		tea_system_actors::tappstore::NAME,
		QueryHostingCmlsRequest(serialize(&(token_id, active_only))?),
		mode,
		|res| Box::pin(async move { callback(res.0).await }),
	)
	.await
}

pub async fn get_statements_async<T>(
	account: Option<Account>,
	query_date: Option<SimpleDate>,
	mode: IntelliSendMode,
	callback: T,
) -> Result<()>
where
	T: FnOnce(Vec<(TypedStatement, String, String)>, bool) -> CallbackReturn
		+ Clone
		+ Send
		+ Sync
		+ 'static,
{
	// this max size is hard coded according to `MAX_PROTOCOL_SIZE` defined in libp2p general request
	const MAX_SIZE: u64 = 1024 * 1024;

	intelli_actor_query_ex(
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
		|res| Box::pin(async move { callback(res.0, res.1).await }),
	)
	.await
}

pub async fn query_cml_ra_status<T>(tea_id: &[u8], mode: IntelliSendMode, callback: T) -> Result<()>
where
	T: FnOnce(NodeStatus) -> CallbackReturn + Clone + Send + Sync + 'static,
{
	intelli_actor_query_ex(
		tea_system_actors::tappstore::NAME,
		QueryCmlRaStatusRequest(tea_id.to_vec()),
		mode,
		|res| Box::pin(async move { callback(res.0).await }),
	)
	.await
}

pub async fn query_active_nodes<T>(
	exclude_tea_id: Option<Vec<u8>>,
	mode: IntelliSendMode,
	callback: T,
) -> Result<()>
where
	T: FnOnce(Vec<TeaNodeProfile>) -> CallbackReturn + Clone + Send + Sync + 'static,
{
	intelli_actor_query_ex(
		tea_system_actors::tappstore::NAME,
		QueryActiveNodesRequest(encode_protobuf(tappstore::QueryActiveNodes {
			has_exclude: exclude_tea_id.is_some(),
			exclude_tea_id: exclude_tea_id.unwrap_or_default(),
		})?),
		mode,
		|res| Box::pin(async move { callback(res.0).await }),
	)
	.await
}

/// request all arg related values locally
pub async fn process_pre_args<T>(pre_args: Vec<Arg>, callback: T) -> Result<()>
where
	T: FnOnce(Option<ArgSlots>) -> CallbackReturn + Clone + Send + Sync + 'static,
{
	if pre_args.is_empty() {
		return callback(None).await;
	}

	intelli_actor_query_ex(
		tea_system_actors::tappstore::NAME,
		ProcessPreArgsRequest(pre_args),
		IntelliSendMode::LocalOnly,
		|res| Box::pin(async move { callback(Some(res.0)).await }),
	)
	.await
}

pub async fn query_system_versions<T>(mode: IntelliSendMode, callback: T) -> Result<()>
where
	T: FnOnce(SystemVersions) -> CallbackReturn + Clone + Send + Sync + 'static,
{
	intelli_actor_query_ex(
		tea_system_actors::tappstore::NAME,
		QuerySystemVersionsRequest,
		mode,
		|res| Box::pin(async move { callback(res.0).await }),
	)
	.await
}
