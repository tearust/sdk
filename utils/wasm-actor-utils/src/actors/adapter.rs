use crate::error::Result;
use adapter_actor_codec::*;
use tea_actorx_core::RegId;
use tea_actorx_runtime::post;

pub async fn register_adapter_http_dispatcher(actions: Vec<String>) -> Result<()> {
    post(RegId::Static(NAME).inst(0), RegisterHttp(actions)).await?;
    Ok(())
}
