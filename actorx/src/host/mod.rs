use std::{
	collections::{hash_map, HashMap},
	future::Future,
	iter::FusedIterator,
	mem::ManuallyDrop,
	sync::Arc,
};

use crate::{core::actor::ActorId, metadata::Metadata};
use tea_codec::{
	errorx::Global,
	serde::{get_type_id, ToBytes, TypeId},
	ResultExt,
};
use tokio::{
	sync::{RwLock, RwLockReadGuard},
	task::JoinHandle,
};

use crate::{
	error::{ActorDeactivating, ActorNotExist, Result},
	sdk::{
		actor::{ActorSend, DynActorSend},
		context::{current, current_ref, host, WithGas, WithHost},
		hooks::{Activate, Deactivate},
	},
};

pub mod sys;
mod wasm;
pub use wasm::*;

pub mod context;
pub mod invoke;

type OutputHandler = Arc<RwLock<Box<dyn Fn(String, ActorId) + Send + Sync>>>;

pub(crate) struct Host {
	actors: RwLock<HashMap<ActorId, Arc<ActorAgent>>>,
	wasm_output_handler: OutputHandler,
	#[cfg(feature = "track")]
	tracker: tracker::WorkerTracker,
}

impl Host {
	#[inline(always)]
	async fn get_actor(&self) -> Result<Arc<ActorAgent>> {
		let actors = self.actors.read().await;
		let actor = current_ref(|current| {
			actors
				.get(current)
				.cloned()
				.ok_or_else(|| ActorNotExist(current.clone()))
		})??;
		Ok(actor)
	}

	#[inline(always)]
	pub async fn invoke(&self, req: &[u8]) -> Result<Vec<u8>> {
		self.get_actor().await?.invoke(req).await
	}

	#[inline(always)]
	pub async fn activate(&self) -> Result<()> {
		self.get_actor().await?.activate().await
	}

	#[inline(always)]
	pub async fn deactivate(&self) -> Result<()> {
		self.get_actor().await?.deactivate().await
	}

	#[inline(always)]
	pub async fn remove(&self, id: &ActorId) -> Result<()> {
		let mut actors = self.actors.write().await;
		actors
			.remove(id)
			.expect("Actor runtime internal error: failed to remove actor");
		Ok(())
	}

	#[inline(always)]
	pub async fn register(&self, actor: impl ActorSend) {
		let mut actors = self.actors.write().await;
		actors.insert(
			actor.id().expect("No id is not defined"),
			Arc::new(ActorAgent {
				actor: Box::new(actor),
				is_active: RwLock::new(Status::Uninit),
			}),
		);
	}
}

struct ActorAgent {
	actor: DynActorSend,
	is_active: RwLock<Status>,
}

enum Status {
	Uninit,
	Active,
	Deactivating,
}

impl ActorAgent {
	#[inline(always)]
	async fn metadata(&self) -> Result<Arc<Metadata>> {
		self.actor.metadata().await.err_into()
	}

	#[inline(always)]
	async fn size(&self) -> Result<u64> {
		self.actor.size().await.err_into()
	}

	async fn activate(&self) -> Result<()> {
		let is_active = self.is_active.read().await;
		match *is_active {
			Status::Uninit => (),
			Status::Active => return Ok(()),
			Status::Deactivating => return Err(ActorDeactivating(current()?).into()),
		}

		drop(is_active);

		let mut is_active = self.is_active.write().await;
		match *is_active {
			Status::Uninit => (),
			Status::Active => return Ok(()),
			Status::Deactivating => return Err(ActorDeactivating(current()?).into()),
		}
		*is_active = Status::Active;

		if let Err(e) = self.actor.invoke(&Activate.to_bytes()?).await {
			if e.name() != Global::UnexpectedType {
				return Err(e.into());
			}
		}

		drop(is_active);
		Ok(())
	}

	async fn invoke(&self, req: &[u8]) -> Result<Vec<u8>> {
		let type_id = get_type_id(req);

		if let Ok(Deactivate::TYPE_ID) = type_id {
			return self
				.deactivate()
				.await
				.and_then(|_| ().to_bytes().err_into());
		}

		self.activate().await?;

		if let Ok(Activate::TYPE_ID) = type_id {
			return Ok(().to_bytes()?);
		}

		self.actor.invoke(req).await.err_into()
	}

	async fn deactivate(&self) -> Result<()> {
		let mut is_active = self.is_active.write().await;
		match *is_active {
			Status::Uninit => {
				*is_active = Status::Deactivating;
				self.remove_self().await?;
				return Ok(());
			}
			Status::Active => (),
			Status::Deactivating => return Ok(()),
		}
		*is_active = Status::Deactivating;
		drop(is_active);

		if let Err(e) = self.actor.invoke(&Deactivate.to_bytes()?).await {
			if e.name() != Global::UnexpectedType {
				self.remove_self().await?;
				return Err(e.into());
			}
		}

		self.remove_self().await?;
		Ok(())
	}

	#[inline(always)]
	async fn remove_self(&self) -> Result<()> {
		host()?.remove(&current()?).await
	}
}

#[inline(always)]
pub fn spawn<T>(future: T) -> JoinHandle<T::Output>
where
	T: Future + Send + 'static,
	T::Output: Send + 'static,
{
	tokio::spawn(future.with_host(host().ok()).with_gas())
}

pub trait WithActorHost: Future {
	async fn with_actor_host(self) -> Self::Output;
}

impl<T> WithActorHost for T
where
	T: Future,
{
	#[inline(always)]
	async fn with_actor_host(self) -> Self::Output {
		let host = Arc::new(Host {
			actors: RwLock::new(HashMap::new()),
			wasm_output_handler: Arc::new(RwLock::new(Box::new(|content, actor| {
				println!("{actor}: {content}");
			}))),
			#[cfg(feature = "track")]
			tracker: tracker::WorkerTracker::new(),
		});
		let r = self.with_host(Some(host.clone())).with_gas().await;
		drop(host);
		r
	}
}

pub fn set_wasm_output_handler<F>(handler: F) -> Result<()>
where
	F: Fn(String, ActorId) + Send + Sync + 'static,
{
	let host = host()?;
	let handler = Box::new(handler);
	if let Ok(mut target) = host.wasm_output_handler.try_write() {
		*target = handler;
		return Ok(());
	}
	tokio::spawn(async move {
		let mut target = host.wasm_output_handler.write().await;
		*target = handler;
	});
	Ok(())
}

pub trait ActorExt: ActorSend {
	async fn register(self) -> Result<()>;
}

impl<T> ActorExt for T
where
	T: ActorSend,
{
	#[inline(always)]
	async fn register(self) -> Result<()> {
		host()?.register(self).await;
		Ok(())
	}
}

impl ActorId {
	#[inline(always)]
	pub async fn metadata(&self) -> Result<Arc<Metadata>> {
		let host = host()?;
		let actors = host.actors.read().await;
		actors.get(self).ok_or(ActorNotExist)?.metadata().await
	}

	#[inline(always)]
	pub async fn size(&self) -> Result<u64> {
		let host = host()?;
		let actors = host.actors.read().await;
		actors.get(self).ok_or(ActorNotExist)?.size().await
	}

	#[inline(always)]
	pub async fn iter() -> Result<Iter> {
		use std::mem::transmute;
		let host = host()?;
		let actors = unsafe {
			transmute::<_, RwLockReadGuard<'static, HashMap<ActorId, Arc<ActorAgent>>>>(
				host.actors.read().await,
			)
		};
		let keys = unsafe {
			transmute::<_, hash_map::Keys<'static, ActorId, Arc<ActorAgent>>>(actors.keys())
		};
		Ok(Iter(
			ManuallyDrop::new(host),
			ManuallyDrop::new(actors),
			keys,
		))
	}
}

pub struct Iter(
	ManuallyDrop<Arc<Host>>,
	ManuallyDrop<RwLockReadGuard<'static, HashMap<ActorId, Arc<ActorAgent>>>>,
	hash_map::Keys<'static, ActorId, Arc<ActorAgent>>,
);

impl Iterator for Iter {
	type Item = ActorId;
	#[inline(always)]
	fn next(&mut self) -> Option<Self::Item> {
		self.2.next().cloned()
	}

	#[inline(always)]
	fn size_hint(&self) -> (usize, Option<usize>) {
		self.2.size_hint()
	}
}

impl ExactSizeIterator for Iter {
	#[inline(always)]
	fn len(&self) -> usize {
		self.2.len()
	}
}

impl FusedIterator for Iter {}

impl Drop for Iter {
	#[inline(always)]
	fn drop(&mut self) {
		unsafe {
			ManuallyDrop::drop(&mut self.1);
			ManuallyDrop::drop(&mut self.0);
		}
	}
}
