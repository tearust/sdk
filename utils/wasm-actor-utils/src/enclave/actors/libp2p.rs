use super::{
	enclave::generate_uuid,
	replica::{random_select_validators_locally, IntelliSendMode},
};
use crate::enclave::{
	action::{add_callback, CallbackReturn},
	error::{Error, Errors, Result},
};
use prost::Message;
use std::collections::HashSet;
use tea_actorx2::ActorId;
use tea_codec::{
	deserialize,
	serde::{handle::Request, FromBytes, ToBytes},
	serialize,
};
use tea_runtime_codec::actor_txns::QuerySerial;
use tea_runtime_codec::vmh::{
	error::{VmhCodec, VmhResult},
	message::{
		encode_protobuf,
		structs_proto::{libp2p, tokenstate},
	},
};
use tea_system_actors::libp2p::{
	HasCooldownRequest, ListPeersRequest, MyConnIdRequest, NextSeqNumberRequest, PubMessageRequest,
	RandomPeersRequest,
};

const INTELLI_CANDIDATES_COUNT: usize = 2;

pub async fn my_conn_id() -> Result<String> {
	let conn_id = ActorId::Static(tea_system_actors::libp2p::NAME)
		.call(MyConnIdRequest)
		.await?;
	Ok(conn_id.0)
}

pub async fn is_connection_healthy() -> Result<bool> {
	let cooldown = ActorId::Static(tea_system_actors::libp2p::NAME)
		.call(HasCooldownRequest)
		.await?;
	if !cooldown.0 {
		return Ok(false);
	}
	match connected_peers().await {
		Ok(peers) => {
			if peers.is_empty() {
				return Ok(false);
			}
		}
		Err(_) => return Ok(false),
	}

	Ok(true)
}

pub(crate) async fn libp2p_seq_number() -> Result<u64> {
	let seq_number = ActorId::Static(tea_system_actors::libp2p::NAME)
		.call(NextSeqNumberRequest)
		.await?;
	Ok(seq_number.0)
}

pub async fn send_message(
	target_conn_id: String,
	target_address: libp2p::RuntimeAddress,
	source_action: Option<String>,
	content: Vec<u8>,
) -> Result<u64> {
	let current_peers: HashSet<String> = connected_peers().await?.into_iter().collect();
	if !current_peers.contains(&target_conn_id) {
		return Err(Errors::ConnIdNotExist(target_conn_id).into());
	}

	let seq_number = libp2p_seq_number().await?;
	ActorId::Static(tea_system_actors::libp2p::NAME)
		.call(tea_system_actors::libp2p::SendMessageRequest(
			encode_protobuf(libp2p::GeneralRequest {
				source_conn_id: Default::default(),
				target_conn_id,
				seq_number: Default::default(),
				runtime_message: Some(libp2p::RuntimeMessage {
					source_address: Some(libp2p::RuntimeAddress {
						target_key: Default::default(),
						target_action: source_action.unwrap_or_default(),
					}),
					target_address: Some(target_address),
					content,
				}),
			})?,
			seq_number,
		))
		.await?;
	Ok(seq_number)
}

pub async fn pub_message(
	target_address: libp2p::RuntimeAddress,
	source_action: Option<String>,
	content: Vec<u8>,
	topic_name: Option<String>,
) -> Result<()> {
	ActorId::Static(tea_system_actors::libp2p::NAME)
		.call(PubMessageRequest(encode_protobuf(libp2p::PubMessage {
			source_conn_id: Default::default(),
			topic: topic_name.map(|topic_name| libp2p::Topic { topic_name }),
			runtime_message: Some(libp2p::RuntimeMessage {
				source_address: Some(libp2p::RuntimeAddress {
					// `target_key` and `target_type` set to default because it will updated in libp2p provider
					target_key: Default::default(),
					target_action: source_action.unwrap_or_default(),
				}),
				target_address: Some(target_address),
				content,
			}),
		})?))
		.await?;
	Ok(())
}

pub async fn connected_peers() -> Result<Vec<String>> {
	let buf = ActorId::Static(tea_system_actors::libp2p::NAME)
		.call(ListPeersRequest)
		.await?;
	let res = libp2p::ListPeersResponse::decode(buf.0.as_slice())?;
	Ok(res.peers)
}

pub async fn get_random_peers(peer_count: u32) -> Result<(Vec<String>, bool)> {
	let buf = ActorId::Static(tea_system_actors::libp2p::NAME)
		.call(RandomPeersRequest(encode_protobuf(
			libp2p::RandomPeersRequest { count: peer_count },
		)?))
		.await?;
	let res = libp2p::RandomoPeersResponse::decode(buf.0.as_slice())?;
	Ok((res.peers, res.insufficient_peers))
}

pub async fn intelli_actor_query_ex<C, T>(
	target: &'static [u8],
	arg: C,
	mode: IntelliSendMode,
	callback: T,
) -> Result<()>
where
	C: Request + ToBytes + Clone,
	C::Response: for<'a> FromBytes<'a> + Send,
	T: FnOnce(C::Response) -> CallbackReturn + Clone + Send + Sync + 'static,
{
	match mode {
		IntelliSendMode::RemoteOnly => {
			send_remote_query_ex(target, arg, &None, Some(INTELLI_CANDIDATES_COUNT), callback).await
		}
		IntelliSendMode::LocalOnly => {
			let rtn = ActorId::Static(target).call(arg).await?;
			callback(rtn).await
		}
		IntelliSendMode::BothOk => compatible_query_ex(target, arg, callback).await,
	}
}

async fn send_remote_query_ex<C, T>(
	target: &[u8],
	arg: C,
	e: &Option<Error>,
	candidate_count: Option<usize>,
	callback: T,
) -> Result<()>
where
	C: Request + ToBytes + Clone,
	C::Response: for<'a> FromBytes<'a> + Send,
	T: FnOnce(C::Response) -> CallbackReturn + Clone + Send + Sync + 'static,
{
	let content = arg.to_bytes()?;
	try_send_remotely::<C::Response, T>(
		e,
		generate_query_message(target, &content).await?,
		candidate_count,
		callback,
	)
	.await
}

async fn compatible_query_ex<C, T>(target: &'static [u8], arg: C, callback: T) -> Result<()>
where
	C: Request + ToBytes + Clone,
	C::Response: for<'a> FromBytes<'a> + Send,
	T: FnOnce(C::Response) -> CallbackReturn + Clone + Send + Sync + 'static,
{
	let actor_id = ActorId::Static(target);
	match actor_id.call(arg.clone()).await {
		Ok(rtn) => callback(rtn).await,
		Err(e) => {
			info!(
				"try to query through libp2p remotely,\
					 because intercom call to {} failed: {:?}",
				actor_id, e
			);
			send_remote_query_ex(target, arg, &Some(e.into()), None, callback).await
		}
	}
}

pub(crate) async fn try_send_remotely<C, T>(
	e: &Option<Error>,
	state_receiver_msg: tokenstate::StateReceiverMessage,
	candidate_count: Option<usize>,
	callback: T,
) -> Result<()>
where
	C: Send + for<'a> FromBytes<'a>,
	T: FnOnce(C) -> CallbackReturn + Clone + Send + Sync + 'static,
{
	if let Some(e) = e {
		if !can_async_error_be_ignored(e) {
			return Err(e.clone());
		}
		info!("error can be ignored continue to send remotely");
	}

	let count = if let Some(c) = candidate_count {
		c
	} else {
		INTELLI_CANDIDATES_COUNT
	};
	let validators = random_select_validators_locally(count).await?;
	send_all_state_receiver::<C, T>(validators, state_receiver_msg, callback).await
}

pub async fn send_all_state_receiver<C, T>(
	validators: Vec<(Vec<u8>, String)>,
	state_receiver_msg: tokenstate::StateReceiverMessage,
	callback: T,
) -> Result<()>
where
	C: Send + for<'a> FromBytes<'a>,
	T: FnOnce(C) -> CallbackReturn + Clone + Send + Sync + 'static,
{
	for (_, target_conn_id) in validators {
		let callback_cp = callback.clone();
		send_to_state_receiver(target_conn_id, state_receiver_msg.clone(), move |buf| {
			Box::pin(async move {
				let result: VmhResult<Vec<u8>> = deserialize(buf)?;
				let ret = C::from_bytes(&result?)?;
				callback_cp(ret).await
			})
		})
		.await?;
	}
	Ok(())
}

pub async fn send_message_with_callback<T>(
	target_actor: &[u8],
	target_action: &str,
	content: &[u8],
	target_conn_id: &str,
	callback: T,
) -> Result<()>
where
	T: FnOnce(Vec<u8>) -> CallbackReturn + Send + Sync + 'static,
{
	let seq_number = send_message(
		target_conn_id.to_string(),
		libp2p::RuntimeAddress {
			target_key: target_actor.to_vec(),
			target_action: target_action.to_string(),
		},
		None,
		content.to_vec(),
	)
	.await?;

	add_callback(seq_number, callback).await
}

pub async fn send_to_state_receiver<T>(
	target_conn_id: String,
	msg: tokenstate::StateReceiverMessage,
	callback: T,
) -> Result<()>
where
	T: FnOnce(Vec<u8>) -> CallbackReturn + Send + Sync + 'static,
{
	let seq_number = libp2p_seq_number().await?;
	ActorId::Static(tea_system_actors::libp2p::NAME)
		.call(tea_system_actors::libp2p::SendMessageRequest(
			encode_protobuf(libp2p::GeneralRequest {
				source_conn_id: Default::default(),
				target_conn_id,
				seq_number: Default::default(),
				runtime_message: Some(libp2p::RuntimeMessage {
					source_address: Some(libp2p::RuntimeAddress {
						target_key: Default::default(),
						target_action: Default::default(),
					}),
					target_address: Some(libp2p::RuntimeAddress {
						target_key: tea_system_actors::state_receiver::NAME.to_vec(),
						target_action: "libp2p.state-receiver".to_string(),
					}),
					content: encode_protobuf(msg)?,
				}),
			})?,
			seq_number,
		))
		.await?;
	add_callback(seq_number, callback).await
}

pub fn can_async_error_be_ignored(e: &Error) -> bool {
	let name = e.name();
	name == VmhCodec::IntercomActorNotSupported
		|| name == VmhCodec::IntercomRequestRejected
		|| name == tea_actorx2::error::ActorX2::ActorNotExist
}

pub async fn generate_query_message(
	target_actor: &[u8],
	content: &[u8],
) -> Result<tokenstate::StateReceiverMessage> {
	let uuid = generate_uuid().await?;
	let query = QuerySerial {
		actor_name: target_actor.to_vec(),
		bytes: content.to_vec(),
	};

	let from_token = crate::enclave::actors::env::get_current_wasm_actor_token_id().await?;
	Ok(tokenstate::StateReceiverMessage {
		uuid,
		msg: Some(tokenstate::state_receiver_message::Msg::StateQuery(
			tokenstate::StateQuery {
				data: serialize(&query)?,
			},
		)),
		from_token,
	})
}
