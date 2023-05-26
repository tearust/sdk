use super::{
	enclave::generate_uuid,
	replica::{random_select_validators_locally, IntelliSendMode},
};
use crate::enclave::error::{Error, Errors, Result};
use prost::Message;
use std::collections::HashSet;
use tea_actorx::ActorId;
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
use tea_sdk::ResultExt;
use tea_system_actors::libp2p::{
	HasCooldownRequest, ListPeersRequest, MyConnIdRequest, PubMessageRequest, RandomPeersRequest,
};

const INTELLI_CANDIDATES_COUNT: usize = 2;

/// Return current node's connection id
pub async fn my_conn_id() -> Result<String> {
	let conn_id = ActorId::Static(tea_system_actors::libp2p::NAME)
		.call(MyConnIdRequest)
		.await?;
	Ok(conn_id.0)
}

#[doc(hidden)]
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

#[doc(hidden)]
pub async fn send_message(
	target_conn_id: String,
	target_address: libp2p::RuntimeAddress,
	source_action: Option<String>,
	content: Vec<u8>,
	has_callback: bool,
) -> Result<Vec<u8>> {
	let current_peers: HashSet<String> = connected_peers().await?.into_iter().collect();
	if !current_peers.contains(&target_conn_id) {
		return Err(Errors::ConnIdNotExist(target_conn_id).into());
	}

	let res = ActorId::Static(tea_system_actors::libp2p::NAME)
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
			has_callback,
		))
		.await?;

	if has_callback {
		return match res.0 {
			Some(r) => Ok(r),
			None => Err(Errors::Libp2pCallbackIsNone.into()),
		};
	}
	Ok(vec![])
}

#[doc(hidden)]
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

#[doc(hidden)]
pub async fn connected_peers() -> Result<Vec<String>> {
	let buf = ActorId::Static(tea_system_actors::libp2p::NAME)
		.call(ListPeersRequest)
		.await?;
	let res = libp2p::ListPeersResponse::decode(buf.0.as_slice())?;
	Ok(res.peers)
}

#[doc(hidden)]
pub async fn get_random_peers(peer_count: u32) -> Result<(Vec<String>, bool)> {
	let buf = ActorId::Static(tea_system_actors::libp2p::NAME)
		.call(RandomPeersRequest(encode_protobuf(
			libp2p::RandomPeersRequest { count: peer_count },
		)?))
		.await?;
	let res = libp2p::RandomoPeersResponse::decode(buf.0.as_slice())?;
	Ok((res.peers, res.insufficient_peers))
}

#[doc(hidden)]
pub async fn intelli_actor_query_ex<C>(
	target: &'static [u8],
	arg: C,
	mode: IntelliSendMode,
) -> Result<C::Response>
where
	C: Request + ToBytes + Clone,
	C::Response: for<'a> FromBytes<'a> + Send,
{
	match mode {
		IntelliSendMode::RemoteOnly => {
			send_remote_query_ex(target, arg, &None, Some(INTELLI_CANDIDATES_COUNT)).await
		}
		IntelliSendMode::LocalOnly => {
			let rtn = ActorId::Static(target).call(arg).await?;
			Ok(rtn)
		}
		IntelliSendMode::BothOk => compatible_query_ex(target, arg).await,
	}
}

#[doc(hidden)]
async fn send_remote_query_ex<C>(
	target: &[u8],
	arg: C,
	e: &Option<Error>,
	candidate_count: Option<usize>,
) -> Result<C::Response>
where
	C: Request + ToBytes + Clone,
	C::Response: for<'a> FromBytes<'a> + Send,
{
	let content = arg.to_bytes()?;
	try_send_remotely::<C::Response>(
		e,
		generate_query_message(target, &content).await?,
		candidate_count,
	)
	.await
}

#[doc(hidden)]
async fn compatible_query_ex<C>(target: &'static [u8], arg: C) -> Result<C::Response>
where
	C: Request + ToBytes + Clone,
	C::Response: for<'a> FromBytes<'a> + Send,
{
	let actor_id = ActorId::Static(target);
	match actor_id.call(arg.clone()).await {
		Ok(rtn) => Ok(rtn),
		Err(e) => {
			info!(
				"try to query through libp2p remotely,\
					 because intercom call to {} failed: {:?}",
				actor_id, e
			);
			send_remote_query_ex(target, arg, &Some(e.into()), None).await
		}
	}
}

#[doc(hidden)]
pub(crate) async fn try_send_remotely<C>(
	e: &Option<Error>,
	state_receiver_msg: tokenstate::StateReceiverMessage,
	candidate_count: Option<usize>,
) -> Result<C>
where
	C: Send + for<'a> FromBytes<'a>,
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
	send_all_state_receiver::<C>(validators, state_receiver_msg).await
}

#[doc(hidden)]
pub async fn send_all_state_receiver<C>(
	validators: Vec<(Vec<u8>, String)>,
	state_receiver_msg: tokenstate::StateReceiverMessage,
) -> Result<C>
where
	C: Send + for<'a> FromBytes<'a>,
{
	let current_peers: HashSet<String> = connected_peers().await?.into_iter().collect();
	let validator_conn_ids = validators
		.into_iter()
		.map(|(_, conn_id)| conn_id)
		.collect::<Vec<_>>();
	if validator_conn_ids
		.iter()
		.any(|conn_id| !current_peers.contains(conn_id))
	{
		return Err(Errors::ConnIdsNotExist(validator_conn_ids).into());
	}

	let res = ActorId::Static(tea_system_actors::libp2p::NAME)
		.call(tea_system_actors::libp2p::SendMessageExRequest {
			msg: encode_protobuf(libp2p::GeneralRequest {
				source_conn_id: Default::default(),
				target_conn_id: Default::default(),
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
					content: encode_protobuf(state_receiver_msg)?,
				}),
			})?,
			with_reply: true,
			targets: validator_conn_ids,
		})
		.await?;

	to_response(res.0.ok_or(Errors::Libp2pCallbackIsNone.into())).await
}

#[doc(hidden)]
async fn to_response<C>(res: Result<Vec<u8>>) -> Result<C>
where
	C: for<'a> FromBytes<'a>,
{
	let result: VmhResult<Vec<u8>> = deserialize(res?)?;
	C::from_bytes(&result?).err_into()
}

#[doc(hidden)]
pub async fn send_message_with_callback(
	target_actor: &[u8],
	target_action: &str,
	content: &[u8],
	target_conn_id: &str,
) -> Result<Vec<u8>> {
	send_message(
		target_conn_id.to_string(),
		libp2p::RuntimeAddress {
			target_key: target_actor.to_vec(),
			target_action: target_action.to_string(),
		},
		None,
		content.to_vec(),
		true,
	)
	.await
}

#[doc(hidden)]
pub async fn send_to_state_receiver(
	target_conn_id: String,
	msg: tokenstate::StateReceiverMessage,
) -> Result<Vec<u8>> {
	send_message(
		target_conn_id,
		libp2p::RuntimeAddress {
			target_key: tea_system_actors::state_receiver::NAME.to_vec(),
			target_action: "libp2p.state-receiver".to_string(),
		},
		None,
		encode_protobuf(msg)?,
		true,
	)
	.await
}

#[doc(hidden)]
pub fn can_async_error_be_ignored(e: &Error) -> bool {
	let name = e.name();
	name == VmhCodec::IntercomActorNotSupported
		|| name == VmhCodec::IntercomRequestRejected
		|| name == tea_actorx::error::ActorX::ActorNotExist
}

#[doc(hidden)]
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
