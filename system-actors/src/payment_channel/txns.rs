use super::error::Result;
use serde::{Deserialize, Serialize};
use strum::Display;
use tea_runtime_codec::tapp::{Balance, ChannelId, ChannelItem};

#[derive(Debug, Serialize, Deserialize, Display)]
pub enum PaymentChannelTxn {
	OpenChannel {
		item: ChannelItem,
		payer_acc_sig: Vec<u8>,
		auth_b64: String,
	},
	PayerEarlyTerminate {
		channel_id: ChannelId,
		payer_sig: Vec<u8>,
		auth_b64: String,
	},
	PayerTerminate {
		channel_id: ChannelId,
		payer_sig: Vec<u8>,
		auth_b64: String,
	},
	PayerRefill {
		channel_id: ChannelId,
		refill_amount: Balance,
		payer_sig: Vec<u8>,
		auth_b64: String,
	},
	UpdatePayment {
		channel_id: ChannelId,
		payment_update_sig: Vec<u8>,
		new_fund_remaining: Balance,
		close_channel: bool,
		auth_b64: String,
	},
}

impl PaymentChannelTxn {
	pub fn sign_content(&self) -> Result<Vec<u8>> {
		Ok(match self {
			PaymentChannelTxn::OpenChannel { item, .. } => tea_codec::serialize(item)?,
			PaymentChannelTxn::PayerEarlyTerminate { channel_id, .. } => channel_id.clone(),
			PaymentChannelTxn::PayerTerminate { channel_id, .. } => channel_id.clone(),
			PaymentChannelTxn::PayerRefill {
				channel_id,
				refill_amount,
				..
			} => {
				let mut rtn = channel_id.clone();
				rtn.extend(tea_codec::serialize(refill_amount)?);
				rtn
			}
			PaymentChannelTxn::UpdatePayment {
				channel_id,
				new_fund_remaining,
				close_channel,
				..
			} => {
				let mut rtn = channel_id.clone();
				rtn.extend(tea_codec::serialize(new_fund_remaining)?);
				rtn.extend(tea_codec::serialize(close_channel)?);
				rtn
			}
		})
	}
}
