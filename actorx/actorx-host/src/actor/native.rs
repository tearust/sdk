use super::{looped::ActorFactory, wasm::NativeBudget, Actor, ActorAgent, ActorContext, ActorKind};
use crate::error::{Error, Result};
use futures::Future;
use tea_actorx_core::ActorId;
use tea_codec::{
	serde::handle::{Handle, HandleList, HandleSend, Handles, Request},
	FixSend, ResultExt,
};

pub type CallingCx = Option<ActorId>;

pub trait NativeActor: Send + Sync + 'static
where
	for<'a> &'a mut Self: Handles<CallingCx>,
	for<'a> HandleSend<'a, &'a mut Self, CallingCx>: Send,
{
	const NAME: &'static [u8];
	const AUTO_DEACTIVATE: bool = true;
	fn kind(&self) -> ActorKind {
		ActorKind::Native
	}
}

impl<T> Actor for T
where
	T: NativeActor,
	for<'a> &'a mut Self: Handles<CallingCx>,
	for<'a> BudgetCheckWrapper<&'a mut Self>: Handles<CallingCx>,
	for<'a> HandleSend<'a, &'a mut Self, CallingCx>: Send,
{
	const AUTO_DEACTIVATE: bool = <T as NativeActor>::AUTO_DEACTIVATE;

	fn kind(&self) -> ActorKind {
		NativeActor::kind(self)
	}

	fn invoke(
		&mut self,
		msg: Vec<u8>,
		caller: Option<ActorId>,
	) -> impl Future<Output = Result<Vec<u8>, Error>> + Send + '_ {
		FixSend(async move {
			BudgetCheckWrapper(self)
				.handle(&msg, caller)
				.await
				.err_into()
		})
	}
}

impl<T, A> NativeActorFactory for T
where
	for<'a> &'a mut A: Handles<CallingCx>,
	for<'a> BudgetCheckWrapper<&'a mut A>: Handles<CallingCx>,
	for<'a> HandleSend<'a, &'a mut A, CallingCx>: Send,
	T: Fn(ActorContext) -> Result<A, Error> + Send + Sync + 'static,
	A: NativeActor,
{
	type Actor = A;
	const NAME: &'static [u8] = A::NAME;

	async fn create(&self, context: ActorContext) -> Result<A> {
		self(context)
	}
}

pub trait NativeActorFactory: Send + Sync + 'static {
	type Actor: Actor;
	const NAME: &'static [u8];
	fn create(
		&self,
		context: ActorContext,
	) -> impl Future<Output = Result<Self::Actor>> + Send + '_;
}

impl<T> ActorFactory for T
where
	T: NativeActorFactory,
{
	fn create(
		&self,
		context: ActorContext,
	) -> super::DynFuture<crate::error::Result<super::ActorAgent>> {
		Box::pin(async move {
			Ok(ActorAgent::looped(
				self.create(context.clone()).await?,
				context,
			))
		})
	}
}

pub struct BudgetCheckWrapper<T>(T);
struct AutoTraitWrapper<T>(T);
auto trait NotSliceU8 {}
impl !NotSliceU8 for AutoTraitWrapper<&[u8]> {}

impl<X, Cx, Req> Handle<Cx, Req> for BudgetCheckWrapper<X>
where
	Req: Request,
	X: Handle<Cx, Req>,
	AutoTraitWrapper<Req>: NotSliceU8,
{
	async fn handle(
		self,
		req: Req,
		cx: Cx,
	) -> std::result::Result<
		<Req as Request>::Response,
		tea_codec::serde::error::Error<impl tea_codec::errorx::Scope>,
	> {
		if let Err(e) = NativeBudget::take().check(&req) {
			return Err(e.into_scope());
		}
		self.0.handle(req, cx).await
	}
}

impl<X, Cx> Handles<Cx> for BudgetCheckWrapper<X>
where
	X: Handles<Cx>,
	X::List: HandleList<BudgetCheckWrapper<X>, Cx>,
{
	type List = X::List;
}
