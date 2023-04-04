use tea_actorx2_core::actor::ActorId;
use tea_sdk::serde::ToBytes;

use crate::{
	error::Result,
	hooks::{Activate, Deactivate},
	wasm::runtime::Interrupt,
};

#[inline(always)]
pub async fn invoke(id: ActorId, req: &[u8]) -> Result<Vec<u8>> {
	Interrupt(Some((id.to_vec(), req.to_vec()))).await
}

#[inline(always)]
pub async fn activate(id: ActorId) -> Result<()> {
	Interrupt(Some((id.to_vec(), Activate.to_bytes()?))).await?;
	Ok(())
}

#[inline(always)]
pub async fn deactivate(id: ActorId) -> Result<()> {
	Interrupt(Some((id.to_vec(), Deactivate.to_bytes()?))).await?;
	Ok(())
}
