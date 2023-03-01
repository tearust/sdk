use solc_codec::error::SolcCodec;
use tea_actorx_core::error::ActorX;
use tea_codec::define_scope;
use vmh_codec::error::VmhCodec;

define_scope! {
    EnvActor: pub ActorX, VmhCodec, SolcCodec {
        RoundingError;
    }
}
