use std::{
	cell::Cell,
	future::Future,
	sync::{Arc, Weak},
};

use crate::error::GasFeeExhausted;
use tokio::task_local;

use crate::{
	error::{OutOfActorHostContext, Result},
	host::Host,
};

task_local! {
	static HOST: Weak<Host>;
}

pub(crate) fn host() -> Option<Arc<Host>> {
	HOST.try_with(|x| x.upgrade()).ok().flatten()
}

pub(crate) trait WithHost: Future {
	async fn with_host(self, value: Option<Arc<Host>>) -> Self::Output;
}

impl<T> WithHost for T
where
	T: Future,
{
	async fn with_host(self, value: Option<Arc<Host>>) -> Self::Output {
		HOST.scope(
			match value {
				Some(host) => Arc::downgrade(&host),
				None => Weak::new(),
			},
			self,
		)
		.await
	}
}

task_local! {
	static GAS: Cell<u64>;
}

pub fn get_gas() -> Result<u64> {
	GAS.try_with(|x| x.get())
		.map_err(|_| OutOfActorHostContext.into())
}

pub fn set_gas(gas: u64) -> Result<()> {
	GAS.try_with(|x| x.set(gas))
		.map_err(|_| OutOfActorHostContext.into())
}

pub fn cost(cost: u64) -> Result<()> {
	if let Some(r) = get_gas()?.checked_sub(cost) {
		set_gas(r)
	} else {
		set_gas(0)?;
		Err(GasFeeExhausted.into())
	}
}

pub(crate) trait WithGas: Future {
	async fn with_gas(self) -> Self::Output;
}

impl<T> WithGas for T
where
	T: Future,
{
	async fn with_gas(self) -> Self::Output {
		GAS.scope(Cell::new(0), self).await
	}
}
