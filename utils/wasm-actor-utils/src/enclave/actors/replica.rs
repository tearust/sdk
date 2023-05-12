use super::{
	enclave::generate_uuid, env::apply_validator, libp2p::connected_peers,
	statemachine::new_txn_serial, util::random_select,
};
use crate::enclave::{
	actors::{
		crypto::sha256,
		enclave::get_my_tea_id,
		env::{self, system_time_as_nanos},
		libp2p::try_send_remotely,
		tappstore::process_pre_args,
	},
	error::{Error, Errors, ProviderOperationRejected, Result},
};
use prost::Message;
use std::collections::HashSet;
use tea_actorx::ActorId;
use tea_codec::serialize;
use tea_runtime_codec::tapp::Hash;
use tea_runtime_codec::vmh::message::{
	encode_protobuf,
	structs_proto::{replica, tokenstate},
};
use tea_runtime_codec::{
	actor_txns::{
		pre_args::{Arg, ArgSlots},
		tsid::Tsid,
		Followup, TxnSerial,
	},
	tapp::Ts,
};
use tea_sdk::ResultExt;
use tea_system_actors::replica::{
	GetExecCursorRequest, ReceiveFollowupRequest, ReceiveTxnRequest, ReportTxnExecErrorRequest,
	NAME,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntelliSendMode {
	LocalOnly,
	RemoteOnly,
	BothOk,
}

impl Default for IntelliSendMode {
	fn default() -> Self {
		IntelliSendMode::BothOk
	}
}

pub async fn intelli_send_txn(
	target_actor: &[u8],
	txn_bytes: &[u8],
	pre_args: Vec<Arg>,
	mode: IntelliSendMode,
	gas_limit: u64,
) -> Result<Option<Tsid>> {
	let txn_serial = new_txn_serial(target_actor, txn_bytes.to_vec(), gas_limit).await?;

	if mode == IntelliSendMode::RemoteOnly {
		return try_send_transaction_remotely(&txn_serial, pre_args, &None).await;
	}

	if !apply_validator().await? {
		if mode == IntelliSendMode::LocalOnly {
			return Err(ProviderOperationRejected::NotATypeCml.into());
		}
		return try_send_transaction_remotely(&txn_serial, pre_args, &None).await;
	}

	match send_transaction_locally(&txn_serial, pre_args.clone(), true).await {
		Ok(rtn) => Ok(rtn),
		Err(e) => {
			if mode == IntelliSendMode::LocalOnly {
				return Err(e);
			}
			try_send_transaction_remotely(&txn_serial, pre_args, &Some(e)).await
		}
	}
}

async fn try_send_transaction_remotely(
	txn_serial: &TxnSerial,
	pre_args: Vec<Arg>,
	e: &Option<Error>,
) -> Result<Option<Tsid>> {
	let (cmd, hash, uuid) = gen_command_messages(txn_serial, pre_args).await?;
	info!(
		"try send transaction 0x{} remotely{}",
		hex::encode(hash),
		e.as_ref().map_or_else(
			|| "".to_string(),
			|e| format!(" because send transaction locally failed: {e:?}")
		)
	);

	let from_token = env::get_current_wasm_actor_token_id().await?;
	try_send_remotely::<Option<Tsid>>(
		e,
		tokenstate::StateReceiverMessage {
			uuid,
			msg: Some(tokenstate::state_receiver_message::Msg::TxnFollowupPair(
				tokenstate::TxnFollowupPair {
					txn: Some(cmd),
					followup: Some(tokenstate::StateFollowup {
						data: serialize(&Followup {
							ts: system_time_as_nanos().await?,
							hash,
							sender: get_my_tea_id().await?.as_slice().try_into()?,
						})?,
					}),
				},
			)),
			from_token,
		},
		None,
	)
	.await
}

async fn gen_command_messages(
	txn_serial: &TxnSerial,
	pre_args: Vec<Arg>,
) -> Result<(tokenstate::StateCommand, Hash, String)> {
	let txn_bytes = serialize(txn_serial)?;
	let txn_hash: Hash = sha256(txn_bytes).await?.as_slice().try_into()?;
	let uuid = generate_uuid().await?;

	Ok((
		tokenstate::StateCommand {
			data: txn_serial.bytes().to_vec(),
			target: txn_serial.actor_name().to_vec(),
			nonce: txn_serial.nonce(),
			gas_limit: txn_serial.gas_limit(),
			pre_args: serialize(&pre_args)?,
		},
		txn_hash,
		uuid,
	))
}

/// avoid call this method in tappstore wasm actor
pub async fn send_transaction_locally(
	txn_serial: &TxnSerial,
	pre_args: Vec<Arg>,
	gen_followup: bool,
) -> Result<Option<Tsid>> {
	let txn_serial = txn_serial.clone();
	let args = process_pre_args(pre_args).await?;
	let rtn = send_transaction_locally_ex(&txn_serial, args, gen_followup, None).await?;
	Ok(rtn)
}

pub async fn send_transaction_locally_ex(
	txn_serial: &TxnSerial,
	args: Option<ArgSlots>,
	gen_followup: bool,
	ts: Option<Ts>,
) -> Result<Option<Tsid>> {
	let txn_bytes = serialize(txn_serial)?;
	let txn_hash: Hash = sha256(txn_bytes.clone()).await?.as_slice().try_into()?;
	info!(
		"try send transaction 0x{} locally {} followup",
		hex::encode(txn_hash),
		match gen_followup {
			true => "with",
			false => "without",
		}
	);

	let tsid = ActorId::Static(NAME)
		.call(ReceiveTxnRequest(encode_protobuf(replica::ReceiveTxn {
			txn_bytes,
			args: args.as_ref().map(serialize).transpose()?,
		})?))
		.await?;

	if gen_followup {
		let followup: Vec<u8> = serialize(&Followup {
			ts: match ts {
				Some(ts) => ts,
				None => system_time_as_nanos().await?,
			},
			hash: txn_hash,
			sender: get_my_tea_id().await?.as_slice().try_into()?,
		})?;
		let fol_rtn = ActorId::Static(NAME)
			.call(ReceiveFollowupRequest(encode_protobuf(
				replica::ReceiveFollowup { followup },
			)?))
			.await?;

		if fol_rtn.0.is_some() {
			return Ok(fol_rtn.0);
		}
	};

	Ok(tsid.0)
}

pub async fn report_txn_error(txn_hash: Vec<u8>, error_msg: String) -> Result<()> {
	ActorId::Static(NAME)
		.call(ReportTxnExecErrorRequest(
			txn_hash.as_slice().try_into()?,
			error_msg,
		))
		.await?;
	Ok(())
}

pub async fn import_round_table(round_table_serial: Vec<u8>) -> Result<()> {
	ActorId::Static(tea_system_actors::replica_service::NAME)
		.call(tea_system_actors::replica_service::ImportRoundTableRequest(
			round_table_serial,
		))
		.await
		.err_into()
}

pub async fn export_round_table(tsid: &Option<Tsid>) -> Result<Vec<u8>> {
	let res = ActorId::Static(tea_system_actors::replica_service::NAME)
		.call(tea_system_actors::replica_service::ExportRoundTableRequest(
			*tsid,
		))
		.await?;
	Ok(res.0)
}

pub async fn is_in_round_table_async(tea_id: &[u8]) -> Result<bool> {
	let v = ActorId::Static(tea_system_actors::replica_service::NAME)
		.call(tea_system_actors::replica_service::IsInRoundTableRequest(
			tea_id.try_into()?,
		))
		.await?;
	Ok(v.0)
}

pub async fn get_exec_cursor() -> Result<Option<Tsid>> {
	let tsid = ActorId::Static(NAME).call(GetExecCursorRequest).await?;
	Ok(tsid.0)
}

pub async fn get_validator_members_locally() -> Result<Option<Vec<(Vec<u8>, String)>>> {
	let msg = ActorId::Static(tea_system_actors::replica_service::NAME)
		.call(tea_system_actors::replica_service::ValidatorsMembersRequest)
		.await?;

	let res = replica::ValidatorMembersResponse::decode(msg.0.as_slice())?;
	match res.validator_members {
		Some(validators_members) => {
			let mut replicas = vec![];
			if validators_members.members.len() != validators_members.conn_ids.len() {
				return Err(Errors::MembersConnIdsMismatch.into());
			}
			for i in 0..validators_members.members.len() {
				replicas.push((
					validators_members.members[i].clone(),
					validators_members.conn_ids[i].clone(),
				));
			}
			Ok(Some(replicas))
		}
		None => Ok(None),
	}
}

pub async fn fetch_validator_state_async() -> Result<Option<replica::ValidatorsState>> {
	let buf = ActorId::Static(tea_system_actors::replica_service::NAME)
		.call(tea_system_actors::replica_service::ValidatorsStateRequest)
		.await?;
	let res = replica::ValidatorsStateResponse::decode(buf.0.as_slice())?;
	Ok(res.validators_state)
}

pub async fn random_select_validators_locally(count: usize) -> Result<Vec<(Vec<u8>, String)>> {
	let all_validators = get_validator_members_locally()
		.await?
		.ok_or(Errors::ValidatorIsEmpty)?;
	random_select_connect_peers(all_validators, count).await
}

pub async fn random_select_connect_peers(
	mut peers: Vec<(Vec<u8>, String)>,
	count: usize,
) -> Result<Vec<(Vec<u8>, String)>> {
	let connected_peers: HashSet<String> = connected_peers().await?.into_iter().collect();
	peers.retain(|(_, peer)| connected_peers.contains(peer));

	if peers.is_empty() {
		return Err(Errors::ConnectedPeersIsEmpty.into());
	}
	let validators = random_select(peers, count).await?;
	Ok(validators)
}

pub async fn has_replica_service_init() -> Result<bool> {
	let b = ActorId::Static(tea_system_actors::replica_service::NAME)
		.call(tea_system_actors::replica_service::HasInitRequest)
		.await?;
	Ok(b.0)
}

/// get candidate validators by given tsid, this is a simple VRF that is different from random select
///    functions (such as `random_select_validators_locally`) because all seat nodes can get the same value
pub async fn get_candidate_validators_locally(
	tsid: Tsid,
	count: usize,
) -> Result<Option<Vec<(Vec<u8>, String)>>> {
	let all_validators = get_validator_members_locally().await?;
	Ok(all_validators.map(|mut all_validators| {
		if all_validators.len() < count {
			all_validators.sort_by(|(a, _), (b, _)| a.cmp(b));
			all_validators
		} else {
			sort_by_tsid(tsid, count, all_validators)
		}
	}))
}

fn sort_by_tsid(
	tsid: Tsid,
	count: usize,
	validators: Vec<(Vec<u8>, String)>,
) -> Vec<(Vec<u8>, String)> {
	let mut indicators: Vec<(Vec<u8>, Vec<u8>, String)> = validators
		.into_iter()
		.map(|(k, v)| {
			let indicator = tsid
				.get_seed()
				.into_iter()
				.enumerate()
				.map(|(i, v)| v | k.get(i).unwrap_or(&0))
				.collect::<Vec<u8>>();
			(indicator, k, v)
		})
		.collect();

	indicators.sort_by(|(a, _, _), (b, _, _)| a.cmp(b));
	indicators
		.into_iter()
		.map(|(_, k, v)| (k, v))
		.take(count)
		.collect()
}
