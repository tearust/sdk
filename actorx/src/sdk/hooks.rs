use serde::{Deserialize, Serialize};
use tea_codec::serde::TypeId;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct Activate;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct Deactivate;
