#[cfg(any(not(feature = "mock"), feature = "no-mock"))]
mod wasm;
#[cfg(any(not(feature = "mock"), feature = "no-mock"))]
pub use wasm::*;

use tea_actorx_core::{
	error::{Error, Result},
	ActorId,
};
use tea_codec::{
	errorx::Scope,
	serde::handle::{Handle, HandleList, Handles, Request},
};

pub type CallingCx = Option<ActorId>;

#[derive(Default, Clone)]
pub struct NoCallingCxWrapper<T>(pub T);

struct AutoTraitWrapper<T>(T);
auto trait NotSliceU8 {}
impl !NotSliceU8 for AutoTraitWrapper<&[u8]> {}

impl<Req, T> Handle<CallingCx, Req> for NoCallingCxWrapper<T>
where
	T: Handle<(), Req>,
	Req: Request,
	AutoTraitWrapper<Req>: NotSliceU8,
{
	async fn handle(
		self,
		req: Req,
		_: CallingCx,
	) -> Result<<Req as Request>::Response, Error<impl Scope>> {
		self.0.handle(req, ()).await
	}
}

impl<T> Handles<CallingCx> for NoCallingCxWrapper<T>
where
	T: Handles<()>,
	T::List: HandleList<Self, CallingCx>,
{
	type List = T::List;
}
