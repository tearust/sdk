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
	use tea_actorx_core::RegId;
	use tea_actorx_runtime::call;
	use tea_runtime_codec::runtime::ops::persist::OP_ASYNC_REQUEST;
	use tea_runtime_codec::vmh::message::encode_protobuf;
	use tea_system_actors::persist::*;

	let msg = call(
		RegId::Static(NAME).inst(0),
		ProtoRequest(OP_ASYNC_REQUEST.into(), encode_protobuf(req)?),
	)
	.await?;
	let res = persist::PersistResponse::decode(msg.0.as_slice())?;
	match res.msg.as_ref() {
		Some(persist::persist_response::Msg::ErrorMessage(res)) => {
			Err(Errors::AsyncPersistFailed(res.message.clone()).into())
		}
		_ => Ok(res),
	}
}

pub async fn persist_file(file_name: String, data: Vec<u8>) -> Result<()> {
	async_persist_request(persist::PersistRequest {
		msg: Some(persist::persist_request::Msg::WriteFile(
			persist::WriteFile { file_name, data },
		)),
		..Default::default()
	})
	.await?;
	Ok(())
}
