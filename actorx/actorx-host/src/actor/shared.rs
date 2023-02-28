use std::{any::Any, future::Future, pin::Pin, sync::Arc};

use tea_actorx_core::{
    hook::{Activate, Deactivate},
    ActorId,
};
use tea_codec::{errorx::Global, serde::ToBytes, ResultExt};
use tokio::sync::Mutex;

use crate::error::{Error, Result};

use super::{with_calling_context, ActorContext, ActorKind, CallingContext, DynFuture};

pub trait Actor: Send + Sync + Any {
    fn auto_deactivate(&self) -> bool {
        true
    }

    fn kind(&self) -> ActorKind;

    fn invoke(&self, msg: Vec<u8>, caller: Option<ActorId>) -> DynFuture<Result<Vec<u8>, Error>>;
}

pub struct Shared {
    actor: Arc<dyn Actor>,
    context: ActorContext,
    activated: Mutex<bool>,
    deactivated: Arc<Mutex<bool>>,
}

impl Shared {
    pub fn new<T>(origin: T, context: ActorContext) -> Self
    where
        T: Actor,
    {
        Self {
            actor: Arc::new(origin),
            context,
            activated: Mutex::new(false),
            deactivated: Arc::new(Mutex::new(false)),
        }
    }

    pub async fn invoke(&self, msg: Vec<u8>, caller: Option<ActorId>) -> Result<Vec<u8>> {
        let deactivated_guard = self.deactivated.lock().await;
        let deactivated = *deactivated_guard;
        drop(deactivated_guard);

        if deactivated {
            return Self::host_invoke_dyn(
                &self
                    .context
                    .host
                    .registry(&self.context.id.reg)?
                    .actor(&self.context.id.inst)
                    .await?,
                msg,
                caller,
            )
            .await
            .err_into();
        }

        let context = CallingContext {
            actor: self.context.clone(),
            caller: caller.clone(),
        };

        self.activate(caller.clone()).await?;
        with_calling_context(context, self.actor.invoke(msg, caller)).await
    }

    fn host_invoke_dyn(
        actor: &crate::Actor,
        msg: Vec<u8>,
        caller: Option<ActorId>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>>> + Send + '_>> {
        Box::pin(actor.call_with_caller_raw(msg, caller))
    }

    pub fn post(&self, msg: Vec<u8>, caller: Option<ActorId>) -> Result<()> {
        let actor = self.actor.clone();
        let id = self.context.id.clone();

        crate::spawn(async move {
            if let Err(e) = actor.invoke(msg, caller).await {
                warn!("Actor {id} throws an error during post: {e:?}");
            }
        });
        Ok(())
    }

    pub async fn activate(&self, caller: Option<ActorId>) -> Result<()> {
        let mut activated_guard = self.activated.lock().await;
        let activated = *activated_guard;
        *activated_guard = true;
        drop(activated_guard);

        if activated {
            return Ok(());
        }

        let context = CallingContext {
            actor: self.context.clone(),
            caller: caller.clone(),
        };

        if let Err(e) =
            with_calling_context(context, self.actor.invoke(Activate.to_bytes()?, caller)).await
        {
            if e.name() == Global::UnexpectedType {
                Ok(())
            } else {
                Err(e)
            }
        } else {
            Ok(())
        }
    }

    pub fn id(&self) -> &ActorId {
        &self.context.id
    }

    pub fn kind(&self) -> ActorKind {
        self.actor.kind()
    }
}

impl Drop for Shared {
    fn drop(&mut self) {
        let actor = self.actor.clone();
        let id = self.context.id.clone();
        let deactivated = self.deactivated.clone();
        crate::spawn(async move {
            let mut deactivated_guard = deactivated.lock().await;
            let deactivated = *deactivated_guard;
            *deactivated_guard = true;
            drop(deactivated_guard);

            if deactivated {
                return;
            }

            let result = async move {
                if let Err(e) = actor.invoke(Deactivate.to_bytes()?, None).await {
                    if e.name() == Global::UnexpectedType {
                        Ok(())
                    } else {
                        Err(e)
                    }
                } else {
                    Ok(())
                }
            }
            .await as Result<()>;

            if let Err(e) = result {
                warn!("Actor {id} throws an error during deactivate: {e}");
            }
        });
    }
}
