use crate::tappstore::txns::TopupEventItem;
use serde::{Deserialize, Serialize};
use tea_actorx::ActorId;
use tea_codec::pricing::Priced;
use tea_codec::serde::TypeId;
use tea_runtime_codec::solc::{
	queries::AsyncQuery,
	txns::{AsyncTxn, SingleSign},
	BlockNumber,
};
use tea_runtime_codec::tapp::{
	cml::{CmlId, CmlIntrinsic},
	Account,
};

pub mod error;

pub const NAME: &[u8] = b"tea:layer1";

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(100)]
#[response(())]
pub struct RegisterEventRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(1000)]
#[response(())]
pub struct UpdateTappStartupNodesRequest(pub Vec<(Vec<u8>, CmlId, String)>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(1000)]
#[response(())]
pub struct UpdateCmlInfoRequest(pub Vec<CmlIntrinsic>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(1000)]
pub struct TappStartupNodesFromCacheRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct TappStartupNodesFromCacheResponse(pub Option<Vec<(Vec<u8>, CmlId, String)>>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct CmlInfoFromCacheRequest(pub CmlId);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct CmlInfoFromCacheResponse(pub Option<CmlIntrinsic>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct TappStartupNodesRequest(pub AsyncQuery);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct TappStartupNodesResponse(pub Vec<(CmlId, String)>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetCmlInfoRequest(pub AsyncQuery);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetCmlInfoResponse(pub Vec<CmlIntrinsic>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct SendMultisigTxnRequest(pub AsyncTxn);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct SignMultisigTxnRequest(pub AsyncTxn);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct SignMultisigTxnResponse(pub SingleSign);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct MultisigThresholdRequest(pub AsyncQuery);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct MultisigThresholdResponse(pub u8);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct CurrentBlockNumberRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct CurrentBlockNumberResponse(pub BlockNumber);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(100)]
pub struct RegisteredActorsRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct RegisteredActorsResponse(pub Vec<ActorId>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(100)]
pub struct TappstoreOwnerRequest(pub AsyncQuery);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct TappstoreOwnerResponse(pub Account);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(100)]
pub struct TopupSinceRequest(pub BlockNumber);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct TopupSinceResponse(pub Vec<TopupEventItem>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(100)]
#[response(())]
pub struct Layer1Event(pub Vec<u8>);
