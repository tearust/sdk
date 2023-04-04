use std::{borrow::Cow, future::Future, pin::Pin, sync::Arc};

use tea_actorx2_core::{actor::ActorId, metadata::Metadata};
use tea_codec::{
	errorx::Scope,
	serde::{error::Serde, handle2::HandleBytes},
	ResultExt,
};

use crate::error::{Error, NotSupported, Result};

pub trait Actor: 'static {
	async fn invoke(&self, req: &[u8]) -> Result<Vec<u8>, Error<impl Scope>>;

	async fn metadata(&self) -> Result<Arc<Metadata>> {
		Err(NotSupported("metadata").into())
	}

	fn id(&self) -> Option<ActorId> {
		None
	}
}

pub trait HandlerActor: HandleBytes {
	#[inline(always)]
	async fn pre_handle<'a>(&'a self, req: &'a [u8]) -> Result<Cow<'a, [u8]>, Error<impl Scope>> {
		Ok(Cow::Borrowed(req)) as Result<_>
	}

	#[inline(always)]
	async fn post_handle(&self, _req: &[u8], resp: Vec<u8>) -> Result<Vec<u8>, Error<impl Scope>> {
		Ok(resp) as Result<_>
	}

	fn id(&self) -> Option<ActorId> {
		None
	}
}

impl<T> Actor for T
where
	T: HandlerActor + 'static,
{
	#[inline(always)]
	async fn invoke(&self, req: &[u8]) -> Result<Vec<u8>, Error<Serde>> {
		let req = self.pre_handle(req).await?;
		let resp = self.handle_bytes(&req).await?;
		self.post_handle(&req, resp).await.err_into()
	}

	fn id(&self) -> Option<ActorId> {
		HandlerActor::id(self)
	}
}

pub(crate) trait ActorTAIT: Actor {
	type InvokeScope: Scope;
	type Invoke<'a>: Future<Output = Result<Vec<u8>, Error<Self::InvokeScope>>> + 'a
	where
		Self: 'a;
	fn invoke<'a>(&'a self, req: &'a [u8]) -> Self::Invoke<'a>;

	type MetadataScope: Scope;
	type Metadata<'a>: Future<Output = Result<Arc<Metadata>, Error<Self::MetadataScope>>> + 'a
	where
		Self: 'a;
	fn metadata(&self) -> Self::Metadata<'_>;
}

impl<T> ActorTAIT for T
where
	T: Actor,
{
	type InvokeScope = impl Scope;
	type Invoke<'a> = impl Future<Output = Result<Vec<u8>,Error<Self::InvokeScope>>> + 'a
	where
		Self: 'a;
	#[inline(always)]
	fn invoke<'a>(&'a self, req: &'a [u8]) -> Self::Invoke<'a> {
		Actor::invoke(self, req)
	}

	type MetadataScope = impl Scope;
	type Metadata<'a> = impl Future<Output = Result<Arc<Metadata>, Error<Self::MetadataScope>>> + 'a
	where
		Self: 'a;
	#[inline(always)]
	fn metadata(&self) -> Self::Metadata<'_> {
		Actor::metadata(self)
	}
}

pub trait ActorSend: Actor + Send + Sync {
	type Scope: Scope;
	type Invoke<'a>: Future<Output = Result<Vec<u8>, Error<Self::Scope>>> + Send + 'a
	where
		Self: 'a;
	fn invoke<'a>(&'a self, req: &'a [u8]) -> Self::Invoke<'a>;

	type MetadataScope: Scope;
	type Metadata<'a>: Future<Output = Result<Arc<Metadata>, Error<Self::MetadataScope>>>
		+ Send
		+ 'a
	where
		Self: 'a;
	fn metadata(&self) -> Self::Metadata<'_>;
}

impl<T> ActorSend for T
where
	T: ActorTAIT + Send + Sync,
	for<'a> T::Invoke<'a>: Send,
	for<'a> T::Metadata<'a>: Send,
{
	type Scope = impl Scope;
	type Invoke<'a> = impl Future<Output = Result<Vec<u8>,Error<Self::Scope>>> + Send + 'a
	where
		Self: 'a;
	#[inline(always)]
	fn invoke<'a>(&'a self, req: &'a [u8]) -> Self::Invoke<'a> {
		ActorTAIT::invoke(self, req)
	}

	type MetadataScope = impl Scope;
	type Metadata<'a> = impl Future<Output = Result<Arc<Metadata>, Error<Self::MetadataScope>>>
		+ Send
		+ 'a
	where
		Self: 'a;
	fn metadata(&self) -> Self::Metadata<'_> {
		ActorTAIT::metadata(self)
	}
}

pub trait ActorSendDyn: Send + Sync + 'static {
	fn invoke<'a>(
		&'a self,
		req: &'a [u8],
	) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>>;

	fn metadata(&self) -> Pin<Box<dyn Future<Output = Result<Arc<Metadata>>> + Send + '_>>;

	fn id(&self) -> Option<ActorId>;
}

impl<T> ActorSendDyn for T
where
	T: ActorSend,
{
	#[inline(always)]
	fn invoke<'a>(
		&'a self,
		req: &'a [u8],
	) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>> {
		Box::pin(async move { ActorSend::invoke(self, req).await.err_into() })
	}

	fn metadata(&self) -> Pin<Box<dyn Future<Output = Result<Arc<Metadata>>> + Send + '_>> {
		Box::pin(async move { ActorSend::metadata(self).await.err_into() })
	}

	fn id(&self) -> Option<ActorId> {
		Actor::id(self)
	}
}

pub type DynActorSend = Box<dyn ActorSendDyn>;

impl Actor for DynActorSend {
	#[inline(always)]
	async fn invoke(&self, req: &[u8]) -> Result<Vec<u8>> {
		self.as_ref().invoke(req).await
	}

	async fn metadata(&self) -> Result<Arc<Metadata>> {
		self.as_ref().metadata().await
	}

	fn id(&self) -> Option<ActorId> {
		self.as_ref().id()
	}
}
