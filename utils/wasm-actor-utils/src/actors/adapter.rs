use crate::error::Result;
use tea_actorx_core::RegId;
use tea_actorx_runtime::post;
use tea_adapter_actor_codec::*;

pub async fn register_adapter_http_dispatcher(actions: Vec<String>) -> Result<()> {
	post(RegId::Static(NAME).inst(0), RegisterHttp(actions)).await?;
	Ok(())
}
