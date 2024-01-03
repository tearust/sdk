use std::{
	cell::Cell,
	future::Future,
	sync::{Arc, Weak},
};

use crate::error::{ActorHostDropped, GasFeeExhausted};
use tokio::task_local;

use crate::{error::Result, host::Host};

#[cfg(feature = "timeout")]
pub(crate) mod tracker;

task_local! {
	static HOST: Weak<Host>;
}

pub(crate) fn host() -> Result<Arc<Host>> {
	HOST.try_with(|x| x.upgrade().ok_or(ActorHostDropped.into()))
		.expect("Invoking an actor requires an actor host context set for the current task")
}

#[cfg(feature = "track")]
pub fn tracker() -> Result<super::tracker::WorkerTracker> {
	Ok(host()?.tracker.clone())
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

pub fn get_gas() -> u64 {
	GAS.try_with(|x| x.get())
		.expect("Invoking an actor requires an actor host context set for the current task")
}

pub fn set_gas(gas: u64) {
	GAS.try_with(|x| x.set(gas))
		.expect("Invoking an actor requires an actor host context set for the current task")
}

pub fn cost(cost: u64) -> Result<()> {
	if let Some(r) = get_gas().checked_sub(cost) {
		set_gas(r);
		Ok(())
	} else {
		set_gas(0);
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
