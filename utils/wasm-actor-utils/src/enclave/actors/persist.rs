use crate::enclave::error::Result;
#[cfg(feature = "__test")]
use mocktopus::macros::*;
use tea_runtime_codec::vmh::message::structs_proto::persist;

#[mockable]
#[cfg(feature = "__test")]
pub async fn async_persist_request(
	_req: persist::PersistRequest,
) -> Result<persist::PersistResponse> {
	Ok(Default::default())
}

#[cfg(not(feature = "__test"))]
pub async fn async_persist_request(
	req: persist::PersistRequest,
) -> Result<persist::PersistResponse> {
	use crate::enclave::error::Errors;
	use prost::Message;
	use tea_actorx::ActorId;
	use tea_runtime_codec::runtime::ops::persist::OP_ASYNC_REQUEST;
	use tea_runtime_codec::vmh::message::encode_protobuf;
	use tea_system_actors::persist::*;

	let msg = ActorId::Static(NAME)
		.call(ProtoRequest(
			OP_ASYNC_REQUEST.into(),
			encode_protobuf(req)?,
			true,
		))
		.await?;
	let res = persist::PersistResponse::decode(msg.0.as_slice())?;
	match res.msg.as_ref() {
		Some(persist::persist_response::Msg::ErrorMessage(res)) => {
			Err(Errors::AsyncPersistFailed(res.message.clone()).into())
		}
		_ => Ok(res),
	}
}

#[mockable]
#[cfg(feature = "__test")]
pub async fn async_persist_request_silently(
	_req: persist::PersistRequest,
) -> Result<persist::PersistResponse> {
	Ok(Default::default())
}

#[cfg(not(feature = "__test"))]
pub async fn async_persist_request_silently(req: persist::PersistRequest) -> Result<()> {
	use tea_actorx::ActorId;
	use tea_runtime_codec::runtime::ops::persist::OP_ASYNC_REQUEST;
	use tea_runtime_codec::vmh::message::encode_protobuf;
	use tea_system_actors::persist::*;

	ActorId::Static(NAME)
		.call(ProtoRequest(
			OP_ASYNC_REQUEST.into(),
			encode_protobuf(req)?,
			false,
		))
		.await?;
	Ok(())
}
