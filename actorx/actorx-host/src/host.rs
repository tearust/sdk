use std::{
    fmt::Debug,
    future::Future,
    sync::{Arc, Weak},
};

use dashmap::DashMap;
use tea_actorx_core::{ActorId, InstanceId, RegId};
use tea_actorx_signer::Metadata;
use tea_codec::{
    serde::{handle::Request, FromBytes, ToBytes},
    ArcIterExt, OptionExt, ResultExt,
};
use tokio::{sync::RwLock, task::JoinHandle};
use wasmer::wasmparser::Operator;

use crate::{
    actor::{looped::ActorFactory, ActorAgent, NativeActorFactory, WasmActorFactory},
    billing::{
        get_billing_account_raw, get_gas_limit, with_billing_account, with_billing_account_raw,
        with_gas_limit, AccountId,
    },
    error::{NativeActorNotExists, Result},
    registry,
};

#[derive(Debug, Clone)]
pub struct ActorHost {
    state: Arc<State>,
}

type PrintHandler = Arc<dyn Fn(&str) + Send + Sync>;

struct State {
    registries: DashMap<RegId, registry::Registry>,
    wasm_print_handler: RwLock<PrintHandler>,
}

impl ActorHost {
    pub fn new() -> Self {
        let registries = DashMap::new();
        Self {
            state: Arc::new(State {
                registries,
                wasm_print_handler: RwLock::new(Arc::new(|s| print!("{s}"))),
            }),
        }
    }

    pub async fn set_wasm_print_handler<T>(&self, new_handler: T)
    where
        T: Fn(&str) + Send + Sync + 'static,
    {
        let mut handler = self.state.wasm_print_handler.write().await;
        *handler = Arc::new(new_handler);
    }

    pub(crate) async fn wasm_print_handler(&self) -> Arc<dyn Fn(&str) + Send + Sync> {
        let handler = self.state.wasm_print_handler.read().await;
        handler.clone()
    }

    pub fn downgrade(&self) -> ActorHostRef {
        ActorHostRef {
            state: Arc::downgrade(&self.state),
        }
    }

    pub fn register_wasm<C>(&self, wasm: Vec<u8>, cost: C) -> Result<()>
    where
        C: Fn(&Operator) -> u64 + Clone + Send + Sync + 'static,
    {
        let (factory, metadata) = WasmActorFactory::new(cost, wasm)?;
        self.state.registries.insert(
            metadata.id.clone().into(),
            registry::Registry::new(Box::new(factory), metadata.id.clone().into()),
        );
        Ok(())
    }

    pub fn register_native<F>(&self, factory: F) -> Result<()>
    where
        F: NativeActorFactory,
    {
        let id: RegId = F::NAME.into();
        self.state
            .registries
            .insert(id.clone(), registry::Registry::new(Box::new(factory), id));
        Ok(())
    }

    pub fn register_custom<F>(&self, id: impl Into<RegId>, factory: F) -> Result<()>
    where
        F: ActorFactory,
    {
        let id = id.into();
        self.state
            .registries
            .insert(id.clone(), registry::Registry::new(Box::new(factory), id));
        Ok(())
    }

    pub fn registry_inner(&self, reg: &[u8]) -> Result<registry::Registry> {
        self.state
            .registries
            .get(reg)
            .ok_or(NativeActorNotExists(RegId::from(reg.to_vec())).into())
            .map(|x| x.clone())
    }

    pub fn registry(&self, reg: &[u8]) -> Result<Registry> {
        self.registry_inner(reg).map(|registry| Registry {
            host: self.downgrade(),
            registry,
        })
    }

    pub fn registries(&self) -> impl Iterator<Item = Registry> + Send + Sync {
        let host = self.downgrade();
        self.state
            .arc_iter::<_, DashMap<_, _>>(|x| &x.registries)
            .map(move |registry| Registry {
                registry,
                host: host.clone(),
            })
    }

    pub fn actors(&self) -> impl Iterator<Item = Actor> + Send + Sync {
        self.registries().flat_map(|x| x.actors())
    }

    pub async fn multicast_0(&self) -> Result<Actor> {
        let mut agents = Vec::new();
        for reg in &self.state.registries {
            agents.push(reg.actor(&InstanceId::ZERO, self.downgrade()).await?);
        }
        Ok(Actor {
            actor: ActorAgent::multicast(agents),
        })
    }
}

impl Debug for State {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        Debug::fmt(&self.registries, f)
    }
}

impl Default for ActorHost {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone)]
pub struct Registry {
    host: ActorHostRef,
    registry: registry::Registry,
}

impl Registry {
    pub async fn actor(&self, id: &InstanceId) -> Result<Actor> {
        Ok(Actor {
            actor: self.registry.actor(id, self.host.clone()).await?,
        })
    }

    pub fn actors(&self) -> impl Iterator<Item = Actor> + Send + Sync {
        self.registry.actors().map(|actor| Actor { actor })
    }

    pub fn id(&self) -> &RegId {
        self.registry.id()
    }

    pub fn metadata(&self) -> Option<&Arc<Metadata>> {
        self.registry.metadata()
    }

    pub fn drop_actor(&self, id: &InstanceId) {
        self.registry.drop_actor(id)
    }
}

#[derive(Clone)]
pub struct Actor {
    pub actor: ActorAgent,
}

impl Actor {
    pub fn id(&self) -> &ActorId {
        self.actor.id()
    }

    pub async fn call_with_caller_raw(
        &self,
        msg: Vec<u8>,
        caller: Option<ActorId>,
    ) -> Result<Vec<u8>> {
        self.actor
            .invoke(msg, caller)
            .await?
            .into_iter()
            .next()
            .ok_or_err("result")
    }

    pub async fn call<T>(&self, req: T) -> Result<T::Response>
    where
        T: ToBytes + Request,
        T::Response: for<'a> FromBytes<'a>,
    {
        self.call_with_caller(req, None).await
    }

    pub async fn call_with_caller<T>(&self, req: T, caller: Option<ActorId>) -> Result<T::Response>
    where
        T: ToBytes + Request,
        T::Response: for<'a> FromBytes<'a>,
    {
        let msg = req.to_bytes()?;
        let result = self.actor.invoke(msg, caller).await?;
        T::Response::from_bytes(&result.into_iter().next().ok_or_err("result")?).err_into()
    }

    pub fn post<T>(&self, req: T) -> Result<()>
    where
        T: ToBytes + Request<Response = ()>,
    {
        self.post_with_caller(req, None)
    }

    pub fn post_with_caller<T>(&self, req: T, caller: Option<ActorId>) -> Result<()>
    where
        T: ToBytes + Request<Response = ()>,
    {
        let msg = req.to_bytes()?;
        self.post_with_caller_raw(msg, caller)?;
        Ok(())
    }

    pub fn post_with_caller_raw(&self, msg: Vec<u8>, caller: Option<ActorId>) -> Result<()> {
        self.actor.post(msg, caller)
    }

    pub async fn activate(&self) -> Result<()> {
        self.actor.activate(None).await
    }

    pub async fn activate_with_caller(&self, caller: Option<ActorId>) -> Result<()> {
        self.actor.activate(caller).await
    }
}

#[derive(Clone)]
pub struct ActorHostRef {
    state: Weak<State>,
}

impl ActorHostRef {
    pub fn upgrade(&self) -> Option<ActorHost> {
        self.state.upgrade().map(|state| ActorHost { state })
    }
}

impl Debug for ActorHostRef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(host) = self.upgrade() {
            Debug::fmt(&host, f)
        } else {
            f.debug_struct("WeakActorHost").finish()
        }
    }
}

impl ActorHostRef {
    pub fn registry(&self, reg: &[u8]) -> Result<Registry> {
        self.upgrade().ok_or_err("host")?.registry(reg)
    }

    pub fn registries(&self) -> Result<impl Iterator<Item = Registry> + Send + Sync> {
        Ok(self.upgrade().ok_or_err("host")?.registries())
    }

    pub fn actors(&self) -> Result<impl Iterator<Item = Actor> + Send + Sync> {
        Ok(self.upgrade().ok_or_err("host")?.actors())
    }
}

pub fn spawn<T>(future: T) -> JoinHandle<T::Output>
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    if let Some(billing) = get_billing_account_raw() {
        tea_codec::runtime::spawn(with_billing_account_raw(billing, future))
    } else {
        tea_codec::runtime::spawn(future)
    }
}

#[deprecated]
pub fn gas_spawn<T>(
    gas_limit: Option<u64>,
    backup_acct: Option<impl AccountId>,
    future: T,
) -> JoinHandle<T::Output>
where
    T: Future + Send + 'static,
    T::Output: Send + 'static,
{
    let mut limit = get_gas_limit();
    if limit == 0 {
        limit = gas_limit.unwrap_or(0_u64)
    }
    if let Some(billing) = get_billing_account_raw() {
        tea_codec::runtime::spawn(with_billing_account_raw(
            billing,
            with_gas_limit(limit, future),
        ))
    } else if let Some(acct) = backup_acct {
        tea_codec::runtime::spawn(with_billing_account(acct, with_gas_limit(limit, future)))
    } else {
        tea_codec::runtime::spawn(with_gas_limit(limit, future))
    }
}
