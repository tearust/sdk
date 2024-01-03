use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::actor::ActorId;

#[derive(Debug, Error, Serialize, Deserialize)]
#[error("Gas fee is exhausted within wasm actor {0}")]
pub struct GasFeeExhausted(pub ActorId);
