use tea_codec::define_scope;
use thiserror::Error;
use wasm_actor_utils::error::Actor;

define_scope! {
    ClientUtilityActor: Actor {
        Errors => ClientUtilityActor, @Display, @Debug;
    }
}

#[derive(Debug, Error)]
pub enum Errors {
    #[error("Unknown request")]
    UnknownRequest,

    #[error("Unknown action {0}")]
    UnknownAction(String),

    #[error("failed to parse address from string")]
    ParseAddressError,

    #[error("u128 length should be {0}")]
    U128Length(usize),
}
