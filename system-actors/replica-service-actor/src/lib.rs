#![feature(min_specialization)]

use actor_txns::tsid::Tsid;
use serde::{Deserialize, Serialize};
use tapp_common::ReplicaId;
use tea_codec::serde::TypeId;

pub mod error;

extern crate tea_codec as tea_sdk;

pub const NAME: &[u8] = b"com.tea.replica-service-actor";

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct ReplicasRemovedRequest(pub Vec<ReplicaId>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct HasInitRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct HasInitResponse(pub bool);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct IsInRoundTableRequest(pub ReplicaId);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct IsInRoundTableResponse(pub bool);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ValidatorsStateRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ValidatorsStateResponse(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ValidatorsMembersRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ValidatorsMembersResponse(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ExportRoundTableRequest(pub Option<Tsid>);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct ExportRoundTableResponse(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct ImportRoundTableRequest(pub Vec<u8>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
#[response(())]
pub struct ResetReplicaMembersRequest(pub Vec<u8>);
