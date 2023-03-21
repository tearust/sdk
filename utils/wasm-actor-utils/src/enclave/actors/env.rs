use crate::enclave::error::Result;
#[cfg(feature = "__test")]
use mocktopus::macros::*;
use std::{collections::HashMap, time::SystemTime};
use tea_actorx_core::RegId;
use tea_actorx_runtime::{call, post};
use tea_codec::ResultExt;
use tea_runtime_codec::tapp::{BlockNumber, TokenId};
use tea_system_actors::env::*;

pub use tea_system_actors::tokenstate::{CronjobArgs, RandomTickArgs};

pub async fn get_system_time() -> Result<SystemTime> {
	let time = call(RegId::Static(NAME).inst(0), GetSystemTimeRequest).await?;
	Ok(time.0)
}

pub async fn system_time_as_nanos() -> Result<u128> {
	tea_runtime_codec::vmh::utils::system_time_as_nanos(get_system_time().await?).err_into()
}

pub async fn is_replica_test_mode() -> Result<bool> {
	let b = call(RegId::Static(NAME).inst(0), GetReplicaTestModeRequest).await?;
	Ok(b.0)
}

pub async fn apply_validator() -> Result<bool> {
	let v = call(RegId::Static(NAME).inst(0), GetApplyValidatorRequest).await?;
	Ok(v.0)
}

pub async fn is_test_mode() -> Result<bool> {
	let v = call(RegId::Static(NAME).inst(0), IsTestModeRequest).await?;
	Ok(v.0)
}

pub async fn get_current_wasm_actor_token_id() -> Result<Option<String>> {
	let res = call(RegId::Static(NAME).inst(0), GetWasmActorTokenIdRequest).await?;
	info!("Current caller wasm token_id => {:?}", res.0);
	Ok(res.0)
}

pub async fn get_genesis_enclave_pcrs() -> Result<HashMap<String, String>> {
	let res = call(RegId::Static(NAME).inst(0), GenesisEnclavePcrsRequest).await?;
	Ok(res.0)
}

#[cfg(not(feature = "__test"))]
pub async fn tappstore_id() -> Result<TokenId> {
	let tappstore_id = call(RegId::Static(NAME).inst(0), GetTappstoreTokenIdRequest).await?;
	Ok(TokenId::from_hex(tappstore_id.0)?)
}

#[cfg(feature = "__test")]
#[mockable]
pub async fn tappstore_id() -> Result<TokenId> {
	use tea_runtime_codec::tapp::MOCK_TOKEN_ID_TAPPSTORE;

	Ok(MOCK_TOKEN_ID_TAPPSTORE)
}

#[cfg(feature = "__test")]
#[allow(clippy::forget_copy, clippy::forget_ref, clippy::swap_ptr_to_ref)]
#[mockable]
pub async fn get_env_var(_env_var: &str) -> Result<String> {
	Ok("".into())
}

/// Return empty string is the env var is not set by the OS
#[cfg(not(feature = "__test"))]
pub async fn get_env_var(env_var: &str) -> Result<Option<String>> {
	let v = call(RegId::Static(NAME).inst(0), GetRequest(env_var.to_string())).await?;
	Ok(v.0)
}

pub async fn current_timestamp(precision: Precision, trunc_base: i64) -> Result<i64> {
	let t = call(
		RegId::Static(NAME).inst(0),
		GetCurrentTimestampRequest(precision, trunc_base),
	)
	.await?;
	Ok(t.0)
}

pub async fn initial_latest_topup_height() -> Result<BlockNumber> {
	let r = call(RegId::Static(NAME).inst(0), InitialLatestTopupHeightRequest).await?;
	Ok(r.0)
}

/// register a random tick, `range_start` and `range_end` specifying the min and max tick interval
/// in milliseconds
pub async fn register_random_tick(args: RandomTickArgs) -> Result<()> {
	post(
		RegId::Static(tea_system_actors::tokenstate::NAME).inst(0),
		tea_system_actors::tokenstate::RegisterRandomTickRequest(args),
	)
	.await
}

pub async fn register_cron_job(args: CronjobArgs) -> Result<()> {
	post(
		RegId::Static(tea_system_actors::tokenstate::NAME).inst(0),
		tea_system_actors::tokenstate::RegisterCronjobRequest(args),
	)
	.await
}

pub async fn my_machine_owner() -> Result<String> {
	let owner = call(RegId::Static(NAME).inst(0), GetMachineOwnerRequest).await?;
	Ok(owner.0)
}

pub fn tapp_harberger_token_id() -> Result<TokenId> {
	Ok(TokenId::from_hex(
		"0x1000000000000000000000000000000000000000",
	)?)
}
pub fn tapp_leaderboard_token_id() -> Result<TokenId> {
	Ok(TokenId::from_hex(
		"0x1000000000000000000000000000000000000001",
	)?)
}
pub fn tapp_miner_portal_token_id() -> Result<TokenId> {
	Ok(TokenId::from_hex(
		"0x1000000000000000000000000000000000000002",
	)?)
}
pub fn tapp_seed_auction_token_id() -> Result<TokenId> {
	Ok(TokenId::from_hex(
		"0x1000000000000000000000000000000000000003",
	)?)
}
pub fn tapp_fluencer_token_id() -> Result<TokenId> {
	Ok(TokenId::from_hex(
		"0x1000000000000000000000000000000000000004",
	)?)
}
pub fn tapp_email_wallet_token_id() -> Result<TokenId> {
	Ok(TokenId::from_hex(
		"0x1000000000000000000000000000000000000005",
	)?)
}
pub fn tapp_dev_portal_token_id() -> Result<TokenId> {
	Ok(TokenId::from_hex(
		"0x1000000000000000000000000000000000000006",
	)?)
}
