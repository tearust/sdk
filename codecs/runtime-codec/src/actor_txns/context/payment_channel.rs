use super::PaymentChannelContext;
use crate::tapp::{Balance, ChannelId, ChannelItem};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaymentChannelContextImpl {
	new_channels: HashMap<ChannelId, ChannelItem>,
	update_payments: HashMap<ChannelId, (Balance, bool)>,
	early_terminate: HashSet<ChannelId>,
	terminate: HashSet<ChannelId>,
	payee_terminate: HashSet<ChannelId>,
	payer_refills: HashMap<ChannelId, Balance>,
}

impl PaymentChannelContext for PaymentChannelContextImpl {
	fn create_channel(&mut self, item: ChannelItem) {
		self.new_channels.insert(item.channel_id.clone(), item);
	}

	fn add_update_payment(&mut self, channel_id: ChannelId, remaining: Balance, close: bool) {
		self.update_payments.insert(channel_id, (remaining, close));
	}

	fn add_payer_early_terminate(&mut self, channel_id: ChannelId) {
		self.early_terminate.insert(channel_id);
	}

	fn add_payer_terminate(&mut self, channel_id: ChannelId) {
		self.terminate.insert(channel_id);
	}

	fn add_payee_terminate(&mut self, channel_id: ChannelId) {
		self.payee_terminate.insert(channel_id);
	}

	fn add_payer_refill(&mut self, channel_id: ChannelId, refill_amount: Balance) {
		self.payer_refills.insert(channel_id, refill_amount);
	}

	fn get_new_channels(&self) -> &HashMap<ChannelId, ChannelItem> {
		&self.new_channels
	}

	fn get_update_payments(&self) -> &HashMap<ChannelId, (Balance, bool)> {
		&self.update_payments
	}

	fn get_payer_early_terminate(&self) -> &HashSet<ChannelId> {
		&self.early_terminate
	}

	fn get_payer_terminate(&self) -> &HashSet<ChannelId> {
		&self.terminate
	}

	fn get_payee_terminate(&self) -> &HashSet<ChannelId> {
		&self.payee_terminate
	}

	fn get_payer_refills(&self) -> &HashMap<ChannelId, Balance> {
		&self.payer_refills
	}
}
