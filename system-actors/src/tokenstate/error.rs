use tea_actorx::error::ActorX;
use tea_codec::define_scope;

define_scope! {
	TokenstateActor: ActorX {
		Errors;
		GluesqlError;
		IoError;
		DbNotFound;
	}
}
