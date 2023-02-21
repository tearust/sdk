use tea_actorx_core::error::ActorX;
use tea_codec::define_scope;
use thiserror::Error;

define_scope! {
	Runtime: ActorX {
		ArgsTypeMismatch => @ActorX::ArgsTypeMismatch, @Display, @Debug;
	}
}

#[derive(Debug, Error)]
#[error("Args type does not match {0}")]
pub struct ArgsTypeMismatch(pub &'static str);
