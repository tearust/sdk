use serde::{Deserialize, Serialize};
use tea_codec::{pricing::Priced, serde::TypeId};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct PreInvoke;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct PostInvoke;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TypeId, Priced)]
#[price(100)] // TODO: change this
#[response(())]
pub struct Activate;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct Deactivate;
