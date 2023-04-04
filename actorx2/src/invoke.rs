use std::{borrow::Borrow, future::Future};

use tea_actorx2_core::actor::ActorId;
use tea_codec::{
	serde::{handle::Request, FromBytes, ToBytes},
	ResultExt,
};

#[cfg(feature = "host")]
use crate::context::WithCallingStack;
use crate::error::Result;

#[cfg(feature = "host")]
mod host;
#[cfg(feature = "wasm")]
#[allow(dead_code)]
mod wasm;

pub trait ActorIdExt {
	fn invoke_raw<'a>(&self, req: &'a [u8]) -> impl Future<Output = Result<Vec<u8>>> + Send + 'a;

	fn call<Req>(
		&self,
		req: impl Borrow<Req>,
	) -> impl Future<Output = Result<Req::Response>> + Send + 'static
	where
		Req: Request + ToBytes,
		Req::Response: for<'x> FromBytes<'x>;

	fn activate(&self) -> impl Future<Output = Result<()>> + Send + 'static;

	fn deactivate(&self) -> impl Future<Output = Result<()>> + Send + 'static;
}

impl ActorIdExt for ActorId {
	#[inline(always)]
	fn invoke_raw<'a>(&self, req: &'a [u8]) -> impl Future<Output = Result<Vec<u8>>> + Send + 'a {
		invoke(self.clone(), req)
	}

	fn call<Req>(
		&self,
		req: impl Borrow<Req>,
	) -> impl Future<Output = Result<Req::Response>> + Send + 'static
	where
		Req: Request + ToBytes,
		Req::Response: for<'x> FromBytes<'x>,
	{
		let target = self.clone();
		let req = req.borrow().to_bytes();
		async move {
			let req = req?;
			let resp = invoke(target, &req).await?;
			FromBytes::from_bytes(&resp).err_into()
		}
	}

	fn activate(&self) -> impl Future<Output = Result<()>> + Send + 'static {
		activate(self.clone())
	}

	fn deactivate(&self) -> impl Future<Output = Result<()>> + Send + 'static {
		deactivate(self.clone())
	}
}

#[inline(always)]
fn invoke(target: ActorId, req: &[u8]) -> impl Future<Output = Result<Vec<u8>>> + Send + '_ {
	#[cfg(feature = "host")]
	return host::invoke(req).invoke_target(target);
	#[cfg(not(feature = "host"))]
	return wasm::invoke(target, req);
}

#[inline(always)]
fn activate(target: ActorId) -> impl Future<Output = Result<()>> + Send + 'static {
	#[cfg(feature = "host")]
	let dispatch = host::activate().invoke_target(target);
	#[cfg(not(feature = "host"))]
	let dispatch = wasm::activate(target);
	async move { dispatch.await.map(|_| ()) }
}

#[inline(always)]
fn deactivate(target: ActorId) -> impl Future<Output = Result<()>> + Send + 'static {
	#[cfg(feature = "host")]
	let dispatch = host::deactivate().invoke_target(target);
	#[cfg(not(feature = "host"))]
	let dispatch = wasm::deactivate(target);
	async move { dispatch.await.map(|_| ()) }
}
