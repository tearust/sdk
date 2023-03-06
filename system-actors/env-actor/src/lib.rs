#![feature(min_specialization)]

use std::{collections::HashMap, time::SystemTime};

use serde::{Deserialize, Serialize};
use tea_codec::pricing::Priced;
use tea_codec::serde::TypeId;
use tea_solc_codec::{BlockNumber, ContractAddresses};
use tea_vmh_codec::io::VersionInfo;

pub mod error;

extern crate tea_codec as tea_sdk;

pub const NAME: &[u8] = b"tea:env";

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetRequest(pub String);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetResponse(pub Option<String>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetSystemTimeRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetSystemTimeResponse(pub SystemTime);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetCurrentTimestampRequest(pub Precision, pub i64);
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetCurrentTimestampResponse(pub i64);

#[cfg(not(feature = "nitro"))]
#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct PprofProfileRequest {
	pub sec: u64, // Profile Second
	pub seq: u64, // Sequence Number
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct JemallocProfileRequest {
	pub sec: u64, // Profile Second
	pub seq: u64, // Sequence Number
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetTeaIdRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetTeaIdResponse(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(0)]
pub struct GetMachineOwnerRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetMachineOwnerResponse(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetReplicaTestModeRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetReplicaTestModeResponse(pub bool);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10)]
pub struct GetApplyValidatorRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetApplyValidatorResponse(pub bool);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10)]
pub struct IsTestModeRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct IsTestModeResponse(pub bool);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetTappstoreTokenIdRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetTappstoreTokenIdResponse(pub String);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetTeaContractsRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetTeaContractsResponse(pub ContractAddresses);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetMiningStartupRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetMiningStartupResponse(pub Vec<([u8; 32], u64, String)>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetSystemVersionsRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetSystemVersionsResponse(pub VersionInfo, pub VersionInfo);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetSeqNumberRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GetSeqNumberResponse(pub u64);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct InitialLatestTopupHeightRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct InitialLatestTopupHeightResponse(pub BlockNumber);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Precision {
	Second,
	Minute,
	Hour,
	Day,
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
#[response(())]
pub struct ProfileCast {
	pub profile: Vec<u8>,
	pub seq_num: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GetWasmActorTokenIdRequest;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetWasmActorTokenIdResponse(pub Option<String>);

#[derive(Debug, Clone, Serialize, Deserialize, TypeId, Priced)]
#[price(10000)]
pub struct GenesisEnclavePcrsRequest;
#[derive(Debug, Clone, Serialize, Deserialize, TypeId)]
pub struct GenesisEnclavePcrsResponse(pub HashMap<String, String>);
