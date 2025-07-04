use core::num::NonZeroU64;

use bon::Builder;
use cosmrs::Coin;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Builder)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct TxSummary {
	since_blocks_ago: NonZeroU64,
	start_block_height: NonZeroU64,
	total_gas_used: u64,
	total_txs: u64,
	total_msgs: u64,
	total_signatures: u64,
}

#[derive(Debug, Clone, Builder)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FeeSummary {
	since_blocks_ago: NonZeroU64,
	start_block_height: NonZeroU64,
	total_fees: Coin,
}

pub struct TxSummaryDissolved {
	pub since_blocks_ago: NonZeroU64,
	pub start_block_height: NonZeroU64,
	pub total_gas_used: u64,
	pub total_txs: u64,
	pub total_msgs: u64,
	pub total_signatures: u64,
}

pub struct FeeSummaryDissolved {
	pub since_blocks_ago: NonZeroU64,
	pub start_block_height: NonZeroU64,
	pub total_fees: Coin,
}

impl TxSummary {
	pub fn since_blocks_ago(&self) -> NonZeroU64 {
		self.since_blocks_ago
	}

	pub fn start_block_height(&self) -> NonZeroU64 {
		self.start_block_height
	}

	pub fn total_gas_used(&self) -> u64 {
		self.total_gas_used
	}

	pub fn total_txs(&self) -> u64 {
		self.total_txs
	}

	pub fn total_msgs(&self) -> u64 {
		self.total_msgs
	}

	pub fn total_signatures(&self) -> u64 {
		self.total_signatures
	}

	pub fn dissolve(self) -> TxSummaryDissolved {
		TxSummaryDissolved {
			since_blocks_ago: self.since_blocks_ago,
			start_block_height: self.start_block_height,
			total_gas_used: self.total_gas_used,
			total_txs: self.total_txs,
			total_msgs: self.total_msgs,
			total_signatures: self.total_signatures,
		}
	}
}

impl FeeSummary {
	pub fn since_blocks_ago(&self) -> NonZeroU64 {
		self.since_blocks_ago
	}

	pub fn start_block_height(&self) -> NonZeroU64 {
		self.start_block_height
	}

	pub fn total_fees(&self) -> &Coin {
		&self.total_fees
	}

	pub fn dissolve(self) -> FeeSummaryDissolved {
		FeeSummaryDissolved {
			since_blocks_ago: self.since_blocks_ago,
			start_block_height: self.start_block_height,
			total_fees: self.total_fees,
		}
	}
}
