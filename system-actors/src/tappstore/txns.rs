use serde::{Deserialize, Serialize};
use std::convert::TryFrom;
use strum::Display;
use tea_codec::errorx::Global;
use tea_codec::{deserialize, serialize};
use tea_runtime_codec::actor_txns::{IntoSerial, Transferable, Tsid, Txn, TxnSerial};
use tea_runtime_codec::tapp::{
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
		all: bool,
	},
	Topup {
		item: TopupEventItem,
	},
	Withdraw {
		token_id: TokenId,
		acct: Account,
		amount: Balance,
		all: bool,
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
	UniversalBasicIncome {
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
	RefillUsdtDeposit {
		seat_id: SeatId,
		user: Account,
		amount: Balance,
	},
	AdminAddSeat {
		seat_id: SeatId,
		maintainer: Account,
		tea_id: Vec<u8>,
		auth_b64: String,
	},
	AdminDeleteSeat {
		seat_id: SeatId,
		auth_b64: String,
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
		tea_id: Vec<u8>,
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
		reward_credit: bool,
	},
	FluencerAddCandidate {
		r#type: String,
		url: String,
		key: String,
		user: Account,
		token_id: String,
		reward_credit: bool,
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
		modules: Vec<(String, String)>,
		pcrs: Vec<(PcrType, String)>,
		auth_b64: String,
		expire_at: TimestampShort,
	},
	UpgradeClientVersion {
		url: String,
		version: String,
		auth_b64: String,
		expire_at: TimestampShort,
	},
	RegisterMultisigInfo {
		send_at: Tsid,
		multisig_type: String,
		payload: Vec<u8>,
		validators: Vec<Vec<u8>>,
	},
	UpdateMutisigInfo {
		txn_hash: Hash,
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
	ResetVersionExpiration {
		auth_b64: String,
	},
	AddVersionPcrs {
		version: String,
		pcrs: Vec<(PcrType, String)>,
		auth_b64: String,
		expire_at: Option<TimestampShort>,
	},
	RemoveVersionPcrs {
		version: String,
		auth_b64: String,
	},
	DoubleCheckTopup {
		timestamp: TimestampShort,
	},
	AdminUpdateTxnGasFee {
		auth_b64: String,
		txn_name: String,
		fee: Balance,
	},
	UpdateTAppForDevportal {
		user: Account,
		auth_b64: String,
		token_id: TokenId,
		cid: Option<String>,
		actor_cid: Option<String>,
		state_actor_cid: Option<String>,
		actor_version: Option<u64>,
		state_actor_version: Option<u64>,
		dev_status: Option<String>,
		actor_name: Option<String>,
		state_actor_name: Option<String>,
	},
	ActivateActor {
		token_id: TokenId,
	},
	AddErrorLogForTApp {
		token_id: TokenId,
		actor_type: String,
		tea_id: Option<Vec<u8>>,
		details: String,
	},
	RemoveErrorLogForTApp {
		token_id: TokenId,
		actor_type: String,
	},
	AdminStartCreditEvent {
		amt: Balance,
		end_time: TimestampShort,
	},
	CheckToEndCreditSystem {
		timestamp: TimestampShort,
	},
	PaymentGasSecondStep {
		timestamp: TimestampShort,
	},
	AdminAddGlobalCredit {
		amt: Balance,
	},
	AdminTransferBalance {
		from: Account,
		to: Account,
		auth_b64: String,
		amt: Balance,
	},
	AddReferenceAccountForReward {
		acct: Account,
		reward_acct: Account,
	},
	BurnTeaCarefully {
		token_id: TokenId,
		acct: Account,
		amt: Balance,
	},
	AirdropAddTask {
		token_id: TokenId,
		owner: Account,
		name: String,
		task_type: String,
		reward_type: String,
		reward_text: String,
		description: String,
		tea_id: Vec<u8>,
		game_url: String,
		doc_url: String,
	},
	AirdropUpdateTask {
		token_id: TokenId,
		owner: Account,
		name: String,
		task_type: String,
		reward_type: String,
		reward_text: String,
		description: String,
		tea_id: Vec<u8>,
		game_url: String,
		doc_url: String,
	},
	AirdropRemoveTask {
		token_id: TokenId,
		owner: Account,
	},
	AirdropLoadTokenWithCredit {
		token_id: TokenId,
		from: Account,
	},
	AirdropLoadToken {
		token_id: TokenId,
		from: Account,
		token_amount: Balance,
	},
	AirdropUnloadToken {
		token_id: TokenId,
		to: Account,
		token_amount: Balance,
	},
	AirdropUnloadBatchToken {
		token_id: TokenId,
		to_list: Vec<Account>,
		amount_list: Vec<Balance>,
	},
	TransferToken {
		token_id: TokenId,
		from: Account,
		to: Account,
		amount: Balance,
		all: bool,
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
	type Error = Global;

	fn try_from(value: TxnSerial) -> Result<Self, Self::Error> {
		deserialize(value.bytes())
	}
}

impl IntoSerial for TappstoreTxn {
	type Error = Global;

	fn into_serial(self, nonce: u64, extra: u32, gas_limit: u64) -> Result<TxnSerial, Self::Error> {
		Ok(TxnSerial::new(
			super::NAME.to_vec(),
			serialize(&self)?,
			nonce,
			extra,
			gas_limit,
		))
	}
}

impl Txn<'static> for TappstoreTxn {}
