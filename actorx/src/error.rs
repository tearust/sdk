use tea_sdk::errorx::Global;

pub type Error = ActorX;
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub type ActorX = Global;
