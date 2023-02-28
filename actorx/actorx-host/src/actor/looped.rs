use std::{any::Any, fmt::Debug, future::Future, sync::Arc, time::Duration};

use futures::future;
use tea_actorx_core::{
    hook::{Activate, Deactivate},
    ActorId,
};
use tea_actorx_signer::Metadata;
use tea_codec::{
    errorx::{Global, Scope},
    serde::ToBytes,
    ResultExt,
};
use tokio::{
    sync::{
        mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender},
        oneshot::{channel as quote, Sender as Quote},
    },
    task_local,
};

use crate::{
    billing::{get_billing_account_raw, with_billing_account_raw, AccountId},
    error::{Error, NativeActorCallingWasmActor, Result, RingInvocation},
};

use super::{with_calling_context, ActorAgent, ActorContext, ActorKind, CallingContext, DynFuture};

pub struct Looped {
    sender: UnboundedSender<Message>,
    context: ActorContext,
    kind: ActorKind,
}

impl Looped {
    pub fn new<T>(mut origin: T, context: ActorContext) -> Self
    where
        T: Actor,
    {
        let (tx, mut rx) = unbounded_channel::<Message>();
        let kind = origin.kind();

        let context_clone = context.clone();
        tokio::spawn(async move {
            let mut activated = false;
            let mut deactivated = false;
            let mut last = None;

            while let Some(msg) = {
                if context_clone.id.inst == 0 || !T::AUTO_DEACTIVATE {
                    rx.recv().await
                } else {
                    let recv = if let Some(recv) = last.take() {
                        recv
                    } else {
                        Box::pin(
                            unsafe { &mut *(&mut rx as *mut UnboundedReceiver<Message>) }.recv(),
                        )
                    };
                    let sleep = tokio::time::sleep(Duration::from_millis(5000));
                    tokio::pin!(sleep);

                    match future::select(recv, sleep).await {
                        future::Either::Left((r, _)) => r,
                        future::Either::Right((_, l)) => {
                            last = Some(l);
                            let (tx, _) = quote();
                            Some(Message::Deactivate(
                                tx,
                                None,
                                Arc::new(CallingStack {
                                    id: context_clone.id.clone(),
                                    last: None,
                                    kind,
                                }),
                                None,
                            ))
                        }
                    }
                }
            } {
                match msg {
                    Message::Invoke(msg, quote, caller, stack, billing) => {
                        let caller = caller.or_else(|| stack.last.as_deref().map(|x| x.id.clone()));
                        let exec = CALLING_STACK.scope(stack, async {
                            let context = CallingContext {
                                actor: context_clone.clone(),
                                caller: caller.clone(),
                            };

                            if !activated {
                                with_calling_context(
                                    context.clone(),
                                    origin.activate(caller.clone()),
                                )
                                .await?;
                                activated = true;
                            }

                            if deactivated {
                                return context_clone
                                    .host
                                    .registry(&context_clone.id.reg)?
                                    .actor(&context_clone.id.inst)
                                    .await?
                                    .call_with_caller_raw(msg, caller)
                                    .await
                                    .err_into();
                            }

                            with_calling_context(context, origin.invoke(msg, caller)).await
                        });

                        let result = if let Some(billing) = billing {
                            with_billing_account_raw(billing, exec).await
                        } else {
                            exec.await
                        };

                        if let Some(quote) = quote {
                            _ = quote.send(result.err_into());
                        } else if let Err(e) = result {
                            let id = &context_clone.id;
                            warn!("Actor {id} throws an error during post: {e}");
                        }
                    }

                    Message::Activate(quote, caller, stack, billing) => {
                        let caller = caller.or_else(|| stack.last.as_deref().map(|x| x.id.clone()));
                        let context = CallingContext {
                            actor: context_clone.clone(),
                            caller: caller.clone(),
                        };
                        let exec = CALLING_STACK.scope(stack, async {
                            if activated {
                                return Ok(());
                            }

                            if deactivated {
                                return context_clone
                                    .host
                                    .registry(&context_clone.id.reg)?
                                    .actor(&context_clone.id.inst)
                                    .await?
                                    .activate_with_caller(caller.clone())
                                    .await
                                    .err_into();
                            }

                            with_calling_context(context, origin.activate(caller)).await
                        });

                        let result = if let Some(billing) = billing {
                            with_billing_account_raw(billing, exec).await
                        } else {
                            exec.await
                        };

                        activated = true;
                        _ = quote.send(result.err_into());
                    }

                    Message::Deactivate(quote, caller, stack, billing) => {
                        if deactivated {
                            continue;
                        }

                        let caller = caller.or_else(|| stack.last.as_deref().map(|x| x.id.clone()));
                        let context = CallingContext {
                            actor: context_clone.clone(),
                            caller: caller.clone(),
                        };

                        let exec = with_calling_context(context, origin.deactivate(caller));

                        let result = if let Some(billing) = billing {
                            with_billing_account_raw(billing, exec).await
                        } else {
                            exec.await
                        };

                        if let Err(e) = &result {
                            warn!("{:?}", e);
                        }

                        if let Some(host) = context_clone.host.upgrade() {
                            if let Ok(reg) = host.registry_inner(&context_clone.id.reg) {
                                reg.drop_actor(&context_clone.id.inst);
                            }
                        }

                        deactivated = true;
                        _ = quote.send(result.err_into());
                    }
                }
            }
        });

        Self {
            sender: tx,
            context,
            kind,
        }
    }

    fn track_deadlock(id: ActorId, stack: Arc<CallingStack>) -> Quote<()> {
        const WARN_INTERVAL_SEC: u64 = 60;
        let (track_tx, track_rx) = quote();
        tokio::spawn(async move {
            let mut rx = track_rx;
            let mut count = 0;
            loop {
                match futures::future::select(
                    rx,
                    Box::pin(tokio::time::sleep(Duration::from_secs(WARN_INTERVAL_SEC))),
                )
                .await
                {
                    future::Either::Left((r, _)) => {
                        r.unwrap();
                        return;
                    }
                    future::Either::Right((_, r)) => {
                        rx = r;
                        count += 1;
                        warn!(
                            "{id} has been blocked for {} secs, stack: {stack:?}",
                            count * WARN_INTERVAL_SEC
                        );
                    }
                };
            }
        });
        track_tx
    }

    fn advance_stack(context: &ActorContext, kind: ActorKind) -> Result<Arc<CallingStack>> {
        let last = CALLING_STACK.try_with(|x| x.clone()).ok();
        if let Some(last) = &last {
            if kind == ActorKind::Wasm && last.kind == ActorKind::Native {
                return Err(
                    NativeActorCallingWasmActor(context.id.clone(), last.id.clone()).into(),
                );
            }
        }
        let mut l = last.as_deref();
        while let Some(c) = l {
            if c.id == context.id {
                let mut stack = Vec::new();
                stack.push(context.id.clone());
                let mut l = last.as_deref();
                while let Some(c) = l {
                    stack.push(c.id.clone());
                    l = c.last.as_deref();
                }
                stack.reverse();
                return Err(RingInvocation(
                    unsafe { last.as_deref().unwrap_unchecked() }.id.clone(),
                    context.id.clone(),
                    stack,
                )
                .into());
            }
            l = c.last.as_deref();
        }

        Ok(Arc::new(CallingStack {
            id: context.id.clone(),
            last,
            kind,
        }))
    }

    pub async fn invoke(&self, msg: Vec<u8>, caller: Option<ActorId>) -> Result<Vec<u8>> {
        let (tx, rx) = quote();
        let stack = Self::advance_stack(&self.context, self.kind)?;
        self.sender
            .send(Message::Invoke(
                msg,
                Some(tx),
                caller,
                stack.clone(),
                get_billing_account_raw(),
            ))
            .unwrap();
        let track = Self::track_deadlock(self.context.id.clone(), stack);
        let r = rx.await.unwrap();
        track.send(()).unwrap();
        r
    }

    pub fn post(&self, msg: Vec<u8>, caller: Option<ActorId>) -> Result<()> {
        self.sender
            .send(Message::Invoke(
                msg,
                None,
                caller.or_else(|| CALLING_STACK.try_with(|x| x.id.clone()).ok()),
                Arc::new(CallingStack {
                    id: self.context.id.clone(),
                    kind: self.kind,
                    last: None,
                }),
                get_billing_account_raw(),
            ))
            .unwrap();

        Ok(())
    }

    pub async fn activate(&self, caller: Option<ActorId>) -> Result<()> {
        let (tx, rx) = quote();
        let stack = Self::advance_stack(&self.context, self.kind)?;
        self.sender
            .send(Message::Activate(
                tx,
                caller,
                stack.clone(),
                get_billing_account_raw(),
            ))
            .unwrap();
        let track = Self::track_deadlock(self.context.id.clone(), stack);
        let r = rx.await.unwrap();
        track.send(()).unwrap();
        r
    }

    pub fn id(&self) -> &ActorId {
        &self.context.id
    }

    pub fn kind(&self) -> ActorKind {
        self.kind
    }
}

impl Drop for Looped {
    fn drop(&mut self) {
        let (tx, _) = quote();
        _ = self.sender.send(Message::Deactivate(
            tx,
            None,
            Arc::new(CallingStack {
                id: self.context.id.clone(),
                kind: self.kind,
                last: None,
            }),
            get_billing_account_raw(),
        ))
    }
}

pub trait Actor: Send + Sync + Any {
    const AUTO_DEACTIVATE: bool = true;

    fn kind(&self) -> ActorKind;

    fn invoke(
        &mut self,
        msg: Vec<u8>,
        caller: Option<ActorId>,
    ) -> impl Future<Output = Result<Vec<u8>, Error<impl Scope>>> + Send + '_;
}

trait ActorExt: Actor {
    fn activate(
        &mut self,
        caller: Option<ActorId>,
    ) -> impl Future<Output = Result<(), Error<impl Scope>>> + Send + '_;

    fn deactivate(
        &mut self,
        caller: Option<ActorId>,
    ) -> impl Future<Output = Result<(), Error<impl Scope>>> + Send + '_;
}

impl<T> ActorExt for T
where
    T: Actor,
{
    async fn activate(&mut self, caller: Option<ActorId>) -> Result<(), Error<impl Scope>> {
        if let Err(e) = self.invoke(Activate.to_bytes()?, caller).await {
            if e.name() == Global::UnexpectedType {
                Ok(())
            } else {
                Err(e.into())
            }
        } else {
            Ok(())
        }
    }

    async fn deactivate(&mut self, caller: Option<ActorId>) -> Result<(), Error<impl Scope>> {
        if let Err(e) = self.invoke(Deactivate.to_bytes()?, caller).await {
            if e.name() == Global::UnexpectedType {
                Ok(())
            } else {
                Err(e.into())
            }
        } else {
            Ok(())
        }
    }
}

pub trait ActorFactory: Send + Sync + Any {
    fn create(&self, context: ActorContext) -> DynFuture<Result<ActorAgent>>;
    fn metadata(&self) -> Option<&Arc<Metadata>> {
        None
    }
}

#[derive(Debug)]
enum Message {
    Invoke(
        Vec<u8>,
        Option<Quote<Result<Vec<u8>>>>,
        Option<ActorId>,
        Arc<CallingStack>,
        Option<Box<dyn AccountId>>,
    ),
    Activate(
        Quote<Result<()>>,
        Option<ActorId>,
        Arc<CallingStack>,
        Option<Box<dyn AccountId>>,
    ),
    Deactivate(
        Quote<Result<()>>,
        Option<ActorId>,
        Arc<CallingStack>,
        Option<Box<dyn AccountId>>,
    ),
}

struct CallingStack {
    id: ActorId,
    kind: ActorKind,
    last: Option<Arc<CallingStack>>,
}

impl Debug for CallingStack {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut list = f.debug_list();
        let mut current = self;
        loop {
            list.entry(&current.id);
            if let Some(last) = &current.last {
                current = last;
            } else {
                break;
            }
        }
        list.finish()
    }
}

task_local! {
    static CALLING_STACK: Arc<CallingStack>;
}
