pub use crate::core::actor::*;

use std::{borrow::Cow, future::Future, pin::Pin, sync::Arc};

use tea_codec::{
	errorx::Scope,
	serde::{error::Serde, handle::HandleBytes},
	ResultExt,
};

use crate::{
	core::metadata::Metadata,
	error::{Error, NotSupported, Result},
};

pub trait Actor: 'static {
	async fn invoke(&self, req: &[u8]) -> Result<Vec<u8>, Error<impl Scope>>;

	async fn metadata(&self) -> Result<Arc<Metadata>> {
		Err(NotSupported("metadata").into())
	}

	fn id(&self) -> Option<ActorId> {
		None
	}

	async fn size(&self) -> Result<u64>;

	async fn instance_count(&self) -> Result<u8> {
		Ok(1)
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

	async fn size(&self) -> Result<u64> {
		Ok(0)
	}

	async fn instance_count(&self) -> Result<u8> {
		Ok(1)
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

	type SizeScope: Scope;
	type Size<'a>: Future<Output = Result<u64, Error<Self::SizeScope>>> + 'a
	where
		Self: 'a;
	fn size(&self) -> Self::Size<'_>;

	type InstanceCountScope: Scope;
	type InstanceCount<'a>: Future<Output = Result<u8, Error<Self::InstanceCountScope>>> + 'a
	where
		Self: 'a;
	fn instance_count(&self) -> Self::InstanceCount<'_>;
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

	type SizeScope = impl Scope;
	type Size<'a> = impl Future<Output = Result<u64, Error<Self::SizeScope>>> + 'a
	where
		Self: 'a;
	#[inline(always)]
	fn size(&self) -> Self::Size<'_> {
		Actor::size(self)
	}

	type InstanceCountScope = impl Scope;
	type InstanceCount<'a> = impl Future<Output = Result<u8, Error<Self::InstanceCountScope>>> + 'a
	where
		Self: 'a;
	#[inline(always)]
	fn instance_count(&self) -> Self::InstanceCount<'_> {
		Actor::instance_count(self)
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

	type SizeScope: Scope;
	type Size<'a>: Future<Output = Result<u64, Error<Self::SizeScope>>> + Send + 'a
	where
		Self: 'a;
	fn size(&self) -> Self::Size<'_>;

	type InstanceCountScope: Scope;
	type InstanceCount<'a>: Future<Output = Result<u8, Error<Self::InstanceCountScope>>> + Send + 'a
	where
		Self: 'a;
	fn instance_count(&self) -> Self::InstanceCount<'_>;
}

impl<T> ActorSend for T
where
	T: ActorTAIT + Send + Sync,
	for<'a> T::Invoke<'a>: Send,
	for<'a> T::Metadata<'a>: Send,
	for<'a> T::Size<'a>: Send,
	for<'a> T::InstanceCount<'a>: Send,
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

	type SizeScope = impl Scope;
	type Size<'a> = impl Future<Output = Result<u64, Error<Self::SizeScope>>>
		+ Send
		+ 'a
	where
		Self: 'a;
	fn size(&self) -> Self::Size<'_> {
		ActorTAIT::size(self)
	}

	type InstanceCountScope = impl Scope;
	type InstanceCount<'a> = impl Future<Output = Result<u8, Error<Self::InstanceCountScope>>>
		+ Send
		+ 'a
	where
		Self: 'a;
	fn instance_count(&self) -> Self::InstanceCount<'_> {
		ActorTAIT::instance_count(self)
	}
}

pub trait ActorSendDyn: Send + Sync + 'static {
	fn invoke<'a>(
		&'a self,
		req: &'a [u8],
	) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + 'a>>;

	fn metadata(&self) -> Pin<Box<dyn Future<Output = Result<Arc<Metadata>>> + Send + '_>>;

	fn size(&self) -> Pin<Box<dyn Future<Output = Result<u64>> + Send + '_>>;

	fn instance_count(&self) -> Pin<Box<dyn Future<Output = Result<u8>> + Send + '_>>;

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

	fn size(&self) -> Pin<Box<dyn Future<Output = Result<u64>> + Send + '_>> {
		Box::pin(async move { ActorSend::size(self).await.err_into() })
	}

	fn instance_count(&self) -> Pin<Box<dyn Future<Output = Result<u8>> + Send + '_>> {
		Box::pin(async move { ActorSend::instance_count(self).await.err_into() })
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

	async fn size(&self) -> Result<u64> {
		self.as_ref().size().await
	}

	async fn instance_count(&self) -> Result<u8> {
		self.as_ref().instance_count().await
	}
}
