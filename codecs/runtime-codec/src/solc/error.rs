use tea_sdk::errorx::Global;

pub type Error = Global;
pub type Result<T, E = Error> = std::result::Result<T, E>;
