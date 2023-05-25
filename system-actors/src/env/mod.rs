use std::{collections::HashMap, time::SystemTime};

use serde::{Deserialize, Serialize};
use tea_actorx::{ActorId, CallingStack};
use tea_codec::pricing::Priced;
use tea_codec::serde::TypeId;
use tea_runtime_codec::solc::{BlockNumber, ContractAddresses};
use tea_runtime_codec::vmh::io::VersionInfo;

pub mod error;

/// Actor name for env native actor.
pub const NAME: &[u8] = b"tea:env";

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetRequest(pub String);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetResponse(pub Option<String>);

/// Base requst to return system time.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetSystemTimeRequest;

/// Base response to return system time.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetSystemTimeResponse(pub SystemTime);

/// Base request to return current timestamp.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetCurrentTimestampRequest(pub Precision, pub i64);

/// Base response to return current timestamp.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetCurrentTimestampResponse(pub i64);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct PprofProfileRequest {
	pub sec: u64, // Profile Second
	pub seq: u64, // Sequence Number
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct JemallocProfileRequest {
	pub sec: u64, // Profile Second
	pub seq: u64, // Sequence Number
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct WorkersTrackingRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct WorkersTrackingResponse(pub HashMap<(ActorId, u64), HashMap<u64, CallingStack>>);

/// Base request to return the current miner's tea_id.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetTeaIdRequest;

/// Base response to return the current miner's tea_id.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetTeaIdResponse(pub String);

/// Base request to return the current miner's owner.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(0)]
pub struct GetMachineOwnerRequest;

/// Base response to return the current miner's owner.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetMachineOwnerResponse(pub String);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetReplicaTestModeRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetReplicaTestModeResponse(pub bool);

/// Base request to check current node is a validator or not.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10)]
pub struct GetApplyValidatorRequest;

/// Base response to check current node is a validator or not.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetApplyValidatorResponse(pub bool);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10)]
pub struct IsTestModeRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct IsTestModeResponse(pub bool);

/// Base request to return tappstore token_id.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetTappstoreTokenIdRequest;

/// Base response to return tappstore token_id.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetTappstoreTokenIdResponse(pub String);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetTeaContractsRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetTeaContractsResponse(pub ContractAddresses);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetMiningStartupRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetMiningStartupResponse(pub Vec<([u8; 32], u64, String)>);

/// Base request to return system version.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetSystemVersionsRequest;

/// Base response to return system version.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetSystemVersionsResponse(pub VersionInfo, pub VersionInfo);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetSeqNumberRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetSeqNumberResponse(pub u64);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct InitialLatestTopupHeightRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct InitialLatestTopupHeightResponse(pub BlockNumber);

#[doc(hidden)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Precision {
	Second,
	Minute,
	Hour,
	Day,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct ProfileCast {
	pub profile: Vec<u8>,
	pub seq_num: u64,
}

/// Base request to get a wasm token_id in manifest.
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetWasmActorTokenIdRequest;

/// Base response to get a wasm token_id in manifest.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWasmActorTokenIdResponse(pub Option<String>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GenesisEnclavePcrsRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GenesisEnclavePcrsResponse(pub HashMap<String, String>);

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct RaSettingsRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct RaSettingsResponse {
	pub default_validators_count: usize,
	pub min_validators_count: usize,
}

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct RuntimeInitializedRequest;

#[doc(hidden)]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct RuntimeInitializedResponse(pub bool);
