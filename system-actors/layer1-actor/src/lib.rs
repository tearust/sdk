#![feature(min_specialization)]

use serde::{Deserialize, Serialize};
use solc_codec::{
    queries::AsyncQuery,
    txns::{AsyncTxn, SingleSign},
    BlockNumber,
};
use tapp_common::{
    cml::{CmlId, CmlIntrinsic},
    Account,
};
use tappstore_actor_codec::txns::TopupEventItem;
use tea_actorx_core::ActorId;
use tea_codec::pricing::Priced;
use tea_codec::serde::TypeId;

pub mod error;

extern crate tea_codec as tea_sdk;

pub const NAME: &[u8] = b"tea:layer1";

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(100)]
#[response(())]
pub struct RegisterEventRequest;

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(1000)]
#[response(())]
pub struct UpdateTappStartupNodesRequest(pub Vec<(Vec<u8>, CmlId, String)>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(1000)]
#[response(())]
pub struct UpdateCmlInfoRequest(pub Vec<CmlIntrinsic>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(1000)]
pub struct TappStartupNodesFromCacheRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct TappStartupNodesFromCacheResponse(pub Option<Vec<(Vec<u8>, CmlId, String)>>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct CmlInfoFromCacheRequest(pub CmlId);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct CmlInfoFromCacheResponse(pub Option<CmlIntrinsic>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct TappStartupNodesRequest(pub AsyncQuery);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct TappStartupNodesResponse(pub Vec<(CmlId, String)>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetCmlInfoRequest(pub AsyncQuery);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetCmlInfoResponse(pub Vec<CmlIntrinsic>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct SendMultisigTxnRequest(pub AsyncTxn);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct SignMultisigTxnRequest(pub AsyncTxn);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct SignMultisigTxnResponse(pub SingleSign);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct MultisigThresholdRequest(pub AsyncQuery);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct MultisigThresholdResponse(pub u8);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct CurrentBlockNumberRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct CurrentBlockNumberResponse(pub BlockNumber);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(100)]
pub struct RegisteredActorsRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct RegisteredActorsResponse(pub Vec<ActorId>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(100)]
pub struct TappstoreOwnerRequest(pub AsyncQuery);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct TappstoreOwnerResponse(pub Account);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(100)]
pub struct TopupSinceRequest(pub BlockNumber);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct TopupSinceResponse(pub Vec<TopupEventItem>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(100)]
#[response(())]
pub struct Layer1Event(pub Vec<u8>);
