use tea_codec::define_scope;
use thiserror::Error;

use crate::actor::ActorId;

define_scope! {
	ActorX2Core {
		GasFeeExhausted => GasFeeExhausted;
	}
}

#[derive(Debug, Error)]
#[error("Gas fee is exhausted within wasm actor {0}")]
pub struct GasFeeExhausted(pub ActorId);
