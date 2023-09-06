use tea_actorx::error::ActorX;
use tea_codec::define_scope;
use thiserror::Error;

define_scope! {
	PaymentChannelActor: ActorX {
		NotSupportedSignContent => NotSupportedSignContent, @Display, @Debug;
	}
}

#[derive(Error, Debug)]
#[error("not supported sign content")]
pub struct NotSupportedSignContent;
