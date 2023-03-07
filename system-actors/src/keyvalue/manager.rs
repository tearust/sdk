use serde::{Deserialize, Serialize};
use tea_actorx_core::InstanceId;
use tea_codec::pricing::Priced;
use tea_codec::serde::TypeId;

pub const NAME: &[u8] = b"com.tea.keyvalue-actor.manager";

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(0)]
pub struct AssignInstanceRequest();

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct AssignInstanceResponse(pub InstanceId);
