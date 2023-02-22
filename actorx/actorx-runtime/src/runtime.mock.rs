use super::CallingCx as WasmCallingCx;
use once_cell::sync::OnceCell;
use tea_actorx_core::ActorId;
use tea_actorx_host::actor::{BudgetCheckWrapper, CallingCx, NativeActor};
pub use tea_actorx_host::*;
use tea_codec::{
    errorx::Scope,
    serde::{
        handle::{Handle, HandleList, HandleSend, Handles, Request},
        FromBytes, ToBytes,
    },
    ResultExt,
};

use crate::{error::Error, interface::NoCallingCxWrapper};

static HOST: OnceCell<ActorHost> = OnceCell::new();

fn host() -> ActorHostRef {
    HOST.get()
        .expect("Mocked host must be initialized")
        .downgrade()
}

pub fn init_host(host: ActorHost) {
    HOST.set(host).expect("Mocked host is already initialized")
}

pub async fn call<C, S>(actor_id: impl Into<ActorId>, arg: C) -> Result<C::Response, Error<S>>
where
    C: Request + ToBytes,
    C::Response: for<'a> FromBytes<'a>,
    S: Scope,
{
    let actor_id = actor_id.into();
    host()
        .registry(&actor_id.reg)?
        .actor(&actor_id.inst)
        .await?
        .call(arg)
        .await
        .err_into()
}

pub async fn post<C, S>(actor_id: impl Into<ActorId>, arg: C) -> Result<(), Error<S>>
where
    C: Request<Response = ()> + ToBytes,
    S: Scope,
{
    let actor_id = actor_id.into();
    host()
        .registry(&actor_id.reg)?
        .actor(&actor_id.inst)
        .await?
        .post(arg)
        .err_into()
}

pub async fn post_with_budget<C, S>(
    actor_id: impl Into<ActorId>,
    arg: C,
    #[allow(unused_variables)] budget: u64,
) -> Result<(), Error<S>>
where
    C: Request<Response = ()> + ToBytes,
    S: Scope,
{
    let actor_id = actor_id.into();
    host()
        .registry(&actor_id.reg)?
        .actor(&actor_id.inst)
        .await?
        .post(arg)
        .err_into()
}

pub trait MockedActorName {
    const NAME: &'static [u8];
}

pub struct AsNative<T>(T);

struct AutoTraitWrapper<T>(T);
auto trait NotSliceU8 {}
impl !NotSliceU8 for AutoTraitWrapper<&[u8]> {}

impl<T, Req> Handle<CallingCx, Req> for &mut AsNative<T>
where
    T: Handle<WasmCallingCx, Req> + Clone,
    Req: Request,
    AutoTraitWrapper<Req>: NotSliceU8,
{
    async fn handle(
        self,
        req: Req,
        cx: CallingCx,
    ) -> Result<<Req as Request>::Response, tea_codec::serde::error::Error<impl Scope>> {
        self.0.clone().handle(req, cx).await
    }
}

impl<'a, T> Handles<CallingCx> for &'a mut AsNative<T>
where
    T: Handles<WasmCallingCx>,
    T::List: HandleList<&'a mut AsNative<T>, CallingCx>,
{
    type List = T::List;
}

impl<T> NativeActor for AsNative<T>
where
    T: MockedActorName + Send + Sync + 'static,
    for<'a> &'a mut Self: Handles<CallingCx>,
    for<'a> HandleSend<'a, &'a mut Self, CallingCx>: Send,
{
    const NAME: &'static [u8] = <T as MockedActorName>::NAME;

    fn kind(&self) -> actor::ActorKind {
        actor::ActorKind::Wasm
    }
}

pub trait RegisterMocked {
    fn register_mocked<T>(&self, v: T) -> tea_actorx_host::error::Result<()>
    where
        T: MockedActorName + Send + Sync + Clone + 'static,
        for<'a> &'a mut AsNative<NoCallingCxWrapper<T>>: Handles<CallingCx>,
        for<'a> BudgetCheckWrapper<&'a mut AsNative<NoCallingCxWrapper<T>>>: Handles<CallingCx>,
        for<'a> HandleSend<'a, &'a mut AsNative<NoCallingCxWrapper<T>>, CallingCx>: Send;

    fn register_mocked_with_caller<T>(&self, v: T) -> tea_actorx_host::error::Result<()>
    where
        T: MockedActorName + Send + Sync + Clone + 'static,
        for<'a> &'a mut AsNative<T>: Handles<CallingCx>,
        for<'a> BudgetCheckWrapper<&'a mut AsNative<T>>: Handles<CallingCx>,
        for<'a> HandleSend<'a, &'a mut AsNative<T>, CallingCx>: Send;
}

impl RegisterMocked for ActorHost {
    fn register_mocked<T>(&self, v: T) -> tea_actorx_host::error::Result<()>
    where
        T: MockedActorName + Send + Sync + Clone + 'static,
        for<'a> &'a mut AsNative<NoCallingCxWrapper<T>>: Handles<CallingCx>,
        for<'a> BudgetCheckWrapper<&'a mut AsNative<NoCallingCxWrapper<T>>>: Handles<CallingCx>,
        for<'a> HandleSend<'a, &'a mut AsNative<NoCallingCxWrapper<T>>, CallingCx>: Send,
    {
        self.register_mocked_with_caller(NoCallingCxWrapper(v))
    }

    fn register_mocked_with_caller<T>(&self, v: T) -> tea_actorx_host::error::Result<()>
    where
        T: MockedActorName + Send + Sync + Clone + 'static,
        for<'a> &'a mut AsNative<T>: Handles<CallingCx>,
        for<'a> BudgetCheckWrapper<&'a mut AsNative<T>>: Handles<CallingCx>,
        for<'a> HandleSend<'a, &'a mut AsNative<T>, CallingCx>: Send,
    {
        self.register_native(move |_| Ok(AsNative(v.clone())))
    }
}

impl<T> MockedActorName for NoCallingCxWrapper<T>
where
    T: MockedActorName,
{
    const NAME: &'static [u8] = T::NAME;
}
