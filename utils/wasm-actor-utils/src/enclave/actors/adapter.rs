use crate::enclave::error::Result;
use tea_actorx::ActorId;
use tea_system_actors::adapter::*;

pub async fn register_adapter_http_dispatcher(actions: Vec<String>) -> Result<()> {
	ActorId::Static(NAME).call(RegisterHttp(actions)).await?;
	Ok(())
}
