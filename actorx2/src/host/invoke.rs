use crate::{error::Result, sdk::context::host};

#[inline(always)]
pub async fn invoke(req: &[u8]) -> Result<Vec<u8>> {
	host()?.invoke(req).await
}

#[inline(always)]
pub async fn activate() -> Result<()> {
	host()?.activate().await
}

#[inline(always)]
pub async fn deactivate() -> Result<()> {
	host()?.deactivate().await
}
