use std::sync::Arc;

use tea_codec::ResultExt;

use crate::{
	error::{OutOfActorHostContext, Result},
	host::Host,
	sdk::context::host,
};

#[inline(always)]
async fn get_host() -> Result<Arc<Host>> {
	host().ok_or(OutOfActorHostContext).err_into()
}

#[inline(always)]
pub async fn invoke(req: &[u8]) -> Result<Vec<u8>> {
	get_host().await?.invoke(req).await
}

#[inline(always)]
pub async fn activate() -> Result<()> {
	get_host().await?.activate().await
}

#[inline(always)]
pub async fn deactivate() -> Result<()> {
	get_host().await?.deactivate().await
}
