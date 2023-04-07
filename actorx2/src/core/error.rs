use tea_codec::define_scope;
use tea_sdk::serde::error::Serde;
use thiserror::Error;

use crate::core::actor::ActorId;

define_scope! {
	ActorX2Core: Serde {
		GasFeeExhausted => GasFeeExhausted;
	}
}

#[derive(Debug, Error)]
#[error("Gas fee is exhausted within wasm actor {0}")]
pub struct GasFeeExhausted(pub ActorId);
