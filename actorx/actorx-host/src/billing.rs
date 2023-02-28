use futures::Future;
use std::{
    any::{Any, TypeId},
    fmt::Debug,
};
use tokio::task_local;

task_local! {
    static BILLING_ACCOUNT: Box<dyn AccountId>;
}

pub trait AccountId: Any + Send + Sync + Debug {
    fn dyn_clone(&self) -> Box<dyn AccountId>;
}

impl<T> AccountId for T
where
    T: Clone + Send + Sync + Debug + 'static,
{
    fn dyn_clone(&self) -> Box<dyn AccountId> {
        Box::new(self.clone())
    }
}

#[inline(always)]
pub async fn with_billing_account<O>(
    account: impl AccountId,
    future: impl Future<Output = O>,
) -> O {
    BILLING_ACCOUNT.scope(Box::new(account), future).await
}

#[inline(always)]
pub async fn with_billing_account_raw<O>(
    account: Box<dyn AccountId>,
    future: impl Future<Output = O>,
) -> O {
    BILLING_ACCOUNT.scope(account, future).await
}

#[inline(always)]
pub fn get_billing_account<T>() -> Option<T>
where
    T: AccountId + Clone,
{
    BILLING_ACCOUNT
        .try_with(|x| {
            if Any::type_id(&**x) == TypeId::of::<T>() {
                Some(Clone::clone(unsafe {
                    &*(&**x as *const dyn AccountId as *const T)
                }))
            } else {
                None
            }
        })
        .ok()
        .flatten()
}

pub fn get_billing_account_raw() -> Option<Box<dyn AccountId>> {
    BILLING_ACCOUNT.try_with(|x| (**x).dyn_clone()).ok()
}
