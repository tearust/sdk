use serde::{Deserialize, Serialize};
use strum::Display;
use tea_runtime_codec::tapp::{Balance, ChannelId, ChannelItem};

#[derive(Debug, Serialize, Deserialize, Display)]
pub enum PaymentChannelTxn {
	OpenChannel {
		item: ChannelItem,
		payer_acc_sig: Vec<u8>,
		payee_acc_sig: Vec<u8>,
	},
	PayerEarlyTerminate {
		channel_id: ChannelId,
		payer_sig: Vec<u8>,
	},
	PayerTerminate {
		channel_id: ChannelId,
		payer_sig: Vec<u8>,
	},
	PayerRefill {
		channel_id: ChannelId,
		refill_amount: Balance,
		payer_sig: Vec<u8>,
	},
	UpdatePayment {
		channel_id: ChannelId,
		payment_update_sig: Vec<u8>,
		new_fund_remaining: Balance,
		close_channel: bool,
	},
}
