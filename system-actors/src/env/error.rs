use tea_actorx::error::ActorX;
use tea_codec::define_scope;
use tea_runtime_codec::solc::error::SolcCodec;
use tea_runtime_codec::vmh::error::VmhCodec;

define_scope! {
	EnvActor: pub ActorX, VmhCodec, SolcCodec {
		RoundingError;
	}
}
