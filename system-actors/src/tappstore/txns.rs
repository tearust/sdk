use super::error::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::convert::TryFrom;
use strum_macros::Display;
use tea_actor_txns::{IntoSerial, Transferable, Txn, TxnSerial};
use tea_codec::ResultExt;
use tea_codec::{deserialize, serialize};
use tea_tapp_common::{
	entity::EntitySettings,
	machine::{IssuerId, TappStartupItem},
	ra::{PcrType, TeaNodeProfile},
	seat::SeatId,
	sys::FreezeRequest,
	Account, AccountId, AuthKey, Balance, BlockNumber, Hash, TimestampShort, TokenId,
};

#[derive(Debug, Serialize, Deserialize, Display)]
pub enum TappstoreTxn {
	GenTappstoreKey,
	TAppStoreInit,
	GenAesKey {
		token_id: TokenId,
	},
	GenSessionKey {
		token_id: TokenId,
		acct: Account,
		pk: Vec<u8>,
		tea_id: Vec<u8>,
		data: String,
		signature: String,
	},
	TransferTea {
		token_id: TokenId,
		from: Account,
		to: Account,
		amount: Balance,
		auth_b64: String,
	},
	Topup {
		item: TopupEventItem,
	},
	Withdraw {
		token_id: TokenId,
		acct: Account,
		amount: Balance,
		auth_b64: String,
	},
	PayToApp {
		token_id: TokenId,
		pay_to_token_id: TokenId,
		amount: Balance,
		auth_b64: String,
		reference: Vec<u8>, //reference to this transacction. it is bytes
	},
	SqlTest {
		token_id: TokenId,
		sql: String,
	},
	ExtendAuthkey {
		auth_key: AuthKey,
		new_expire: u128,
	},
	NewTApp {
		entity_settings: EntitySettings,
		auth_b64: String,
	},
	UpdateTApp {
		token_id: TokenId,
		ticker: String,
		detail: String,
		link: String,
		max_allowed_hosts: u32,
		tapp_type: String,
		auth_b64: String,
		cid: String,
	},
	BuyTappToken {
		token_id: TokenId,
		account: Account,
		amount: Balance,
		auth_b64: String,
	},
	SellTappToken {
		token_id: TokenId,
		account: Account,
		amount: Balance,
		auth_b64: String,
	},
	SellAllToken {
		token_id: TokenId,
		account: Account,
		auth_b64: String,
	},
	ConsumeTappToken {
		token_id: TokenId,
		account: Account,
		tea_amount: Balance,
		auth_b64: String,
	},
	ExpenseTappToken {
		token_id: TokenId,
		tea_amount: Balance,
		timestamp: TimestampShort,
	},
	ScheduledExpenseTappToken {
		timestamp: TimestampShort,
	},
	StartMining {
		cml_id: u64,
		tea_id: Vec<u8>,
		owner: Account,
		miner_ip: String,
		orbitdb_id: Option<String>,
	},
	StopMining {
		cml_id: u64,
		owner: Account,
	},
	ResumeMining {
		cml_id: u64,
		owner: Account,
	},
	CmlScheduleDown {
		cml_id: u64,
		owner: Account,
	},
	CmlScheduleUp {
		cml_id: u64,
		owner: Account,
	},
	CmlMigrate {
		cml_id: u64,
		owner: Account,
		tea_id: Option<Vec<u8>>,
		miner_ip: Option<String>,
	},
	UpdateNodeProfile {
		profile: TeaNodeProfile,
		owner: Account,
	},
	UpdateSeatProfile {
		profile: TeaNodeProfile,
		owner: Account,
	},
	RaRequest {
		profile: TeaNodeProfile,
		request_at: TimestampShort,
		is_seat: bool,
	},
	RaResponse {
		tea_id: Vec<u8>,
		profile: Option<TeaNodeProfile>,
		request_at: TimestampShort,
		valid: bool,
		is_seat: bool,
		validators: Vec<Account>,
	},
	ResetTappStartup {
		items: Vec<TappStartupItem>,
		old_items: Vec<TappStartupItem>,
		owner: Account,
		timestamp: TimestampShort,
	},
	RAReward {
		amount: Balance,
		timestamp: TimestampShort,
	},
	FavTapp {
		user: String,
		token_id: TokenId,
	},
	UnFavTapp {
		user: String,
		token_id: TokenId,
	},

	SeatInit,
	BuySeat {
		seat_id: SeatId,
		user: Account,
		price: Balance,
	},
	UpdateSeatEstimate {
		seat_id: SeatId,
		user: Account,
		price: Balance,
	},
	GiveupSeatOwnership {
		seat_id: SeatId,
		user: Account,
	},
	InternalCronjobAction {
		target: String,
		timestamp: TimestampShort,
	},
	RegisterMachine {
		tea_id: Vec<u8>,
		issuer: IssuerId,
		owner: Account,
		signature: Vec<u8>,
		timestamp: TimestampShort,
	},
	TransferMachine {
		tea_id: Vec<u8>,
		user: Account,
		new_owner: Account,
	},
	RegisterForLeaderboard {
		address: Account,
		ref_code: String,
		auth_b64: String,
		email: String,
		telegram: String,
	},
	SetAllowance {
		address: Account,
		token_id: TokenId,
		amount: Balance,
	},
	PaymentGasFromBillingActor {
		list: Vec<(AccountId, Balance)>,
		from_miner: bool,
		tea_id: Option<String>,
	},
	AdminGenerateNewSeed {
		performance: u64,
		lifespan: u64,
		version: u64,
		base_price: Balance,
		step_price: Balance,
	},
	BidForSeed {
		user: Account,
		cml_key: String,
		price: Balance,
		auth_b64: String,
	},
	ClaimForSeed {
		user: Account,
		cml_key: String,
		auth_b64: String,
	},
	TwitterRetweetFaucetForAccount {
		target: Account,
		twitter_id: String,
		token_id: String,
	},
	FluencerAddCandidate {
		r#type: String,
		url: String,
		key: String,
		user: Account,
		token_id: String,
	},
	ScheduledArrangeRaStatus {
		timestamp: TimestampShort,
	},
	CacheOtpForEmailLogin {
		email: String,
		otp: String,
		token_id: String,
	},
	LoginWithEmailAndOtp {
		email: String,
		otp: String,
		token_id: String,
		data: Vec<u8>,
	},
	UpgradeEnclaveVersion {
		url: String,
		version: String,
		modules: HashMap<String, String>,
		pcrs: HashMap<PcrType, String>,
		auth_b64: String,
	},
	UpgradeClientVersion {
		url: String,
		version: String,
		auth_b64: String,
	},
	RegisterMultisigInfo {
		txn_hash: Hash,
		multisig_type: String,
		payload: Vec<u8>,
		collector: Vec<u8>,
	},
	UnregisterMultisigInfo {
		txn_hash: Hash,
	},
	FreezeState {
		request: FreezeRequest,
		auth_b64: String,
	},
	CancelLastFreeze {
		auth_b64: String,
	},
	RemoveVersionPcrs {
		version: String,
		auth_b64: String,
	},
	DoubleCheckTopup,
	AdminUpdateTxnGasFee {
		auth_b64: String,
		txn_name: String,
		fee: Balance,
	},
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct TopupEventItem {
	pub token_address: Account,
	pub sender: Account,
	pub amount: Balance,
	pub height: BlockNumber,
}

impl Transferable for TappstoreTxn {
	fn get_handler_actor() -> Vec<u8> {
		super::NAME.to_vec()
	}
}

impl TryFrom<TxnSerial> for TappstoreTxn {
	type Error = Error;

	fn try_from(value: TxnSerial) -> Result<Self, Self::Error> {
		deserialize(value.bytes()).err_into()
	}
}

impl IntoSerial for TappstoreTxn {
	type Error = Error;

	fn into_serial(self, nonce: u64, gas_limit: u64) -> Result<TxnSerial, Self::Error> {
		Ok(TxnSerial::new(
			super::NAME.to_vec(),
			serialize(&self)?,
			nonce,
			gas_limit,
		))
	}
}

impl Txn<'static> for TappstoreTxn {}
