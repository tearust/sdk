use std::time::{Duration, UNIX_EPOCH};

use actorx_example_codec::{
    GetSystemTimeRequest, GetSystemTimeResponse, PostPrint, TIME_ACTOR_NAME,
};
use tea_sdk::{
    actorx::host::actor::{ActorContext, CallingCx, NativeActor},
    serde::handle::{Handle, Handles},
    Handle,
};
use tokio::time::sleep;

use crate::error::Result;

#[derive(Debug)]
pub struct TimeActor {
    _context: ActorContext,
}

impl TimeActor {
    pub fn new(context: ActorContext) -> Self {
        Self { _context: context }
    }

    async fn get_system_time(&self) -> Result<u128> {
        sleep(Duration::from_millis(1000)).await;
        Ok(std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_millis())
    }
}

impl NativeActor for TimeActor {
    const NAME: &'static [u8] = TIME_ACTOR_NAME;
}

impl Handles<CallingCx> for &mut TimeActor {
    type List = Handle![GetSystemTimeRequest, PostPrint];
}

impl<'a> Handle<CallingCx, GetSystemTimeRequest> for &'a mut TimeActor {
    async fn handle(self, _: GetSystemTimeRequest, _: CallingCx) -> Result<GetSystemTimeResponse> {
        self.get_system_time().await.map(GetSystemTimeResponse)
    }
}

impl<'a> Handle<CallingCx, PostPrint> for &'a mut TimeActor {
    async fn handle(self, PostPrint(s): PostPrint, _: CallingCx) -> Result<()> {
        println!("Post print: {s}");
        Ok(())
    }
}
