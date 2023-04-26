use tea_actorx2::error::ActorX2;
use tea_codec::define_scope;
use tea_runtime_codec::solc::error::SolcCodec;
use tea_runtime_codec::vmh::error::VmhCodec;

define_scope! {
	EnvActor: pub ActorX2, VmhCodec, SolcCodec {
		RoundingError;
	}
}
