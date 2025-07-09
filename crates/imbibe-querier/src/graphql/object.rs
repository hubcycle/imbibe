use core::num::NonZeroU64;

use async_graphql::SimpleObject;
use bytes::Bytes;
use chrono::{DateTime, Utc};
use cosmrs::{Any, Coin, tx::SignerPublicKey};
use imbibe_domain::{
	NonEmptyBz, Sha256,
	block::{Block, Header},
	summary::{FeeSummary, TxSummary},
	tx::{Codespace, Fees, Memo, Tx},
};
use jiff::Timestamp;
use serde_json::Value;

use super::SHA256_LEN;

#[derive(SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct BlockGql {
	header: HeaderGql,
	gas_used: u64,
	hash: [u8; SHA256_LEN],
	data: Vec<Bytes>,
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct HeaderGql {
	chain_id: String,
	height: NonZeroU64,
	time: DateTime<Utc>,
	validators_hash: [u8; SHA256_LEN],
	next_validators_hash: [u8; SHA256_LEN],
	consensus_hash: [u8; SHA256_LEN],
	app_hash: Vec<u8>,
	proposer: Vec<u8>,
	last_commit_hash: Option<[u8; SHA256_LEN]>,
	data_hash: Option<[u8; SHA256_LEN]>,
	last_results_hash: Option<[u8; SHA256_LEN]>,
	evidence_hash: Option<[u8; SHA256_LEN]>,
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct TxGql {
	block_height: NonZeroU64,
	tx_idx_in_block: u64,
	tx_hash: [u8; SHA256_LEN],
	msgs: Vec<String>,
	memo: Option<String>,
	timeout_height: Option<NonZeroU64>,
	signatures: Vec<Vec<u8>>,
	signers: Vec<Value>,
	fees: Option<Vec<CoinGql>>,
	payer: Vec<u8>,
	granter: Option<Vec<u8>>,
	code: u32,
	codespace: Option<String>,
	gas_limit: u64,
	gas_wanted: u64,
	gas_used: u64,
	data_bz: Option<Bytes>,
	tx_bz: Bytes,
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct TxSummaryGql {
	since_blocks_ago: NonZeroU64,
	start_block_height: NonZeroU64,
	total_txs: u64,
	total_gas_used: u64,
	total_msgs: u64,
	total_signatures: u64,
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct FeeSummaryGql {
	since_blocks_ago: NonZeroU64,
	start_block_height: NonZeroU64,
	total_fees: CoinGql,
}

#[derive(SimpleObject)]
#[graphql(rename_fields = "snake_case")]
pub struct CoinGql {
	amount: String,
	denom: String,
}

impl From<Block> for BlockGql {
	fn from(block: Block) -> Self {
		let block = block.dissolve();

		Self {
			header: block.header.into(),
			gas_used: block.gas_used,
			hash: block.hash.into_inner(),
			data: block.data.into_inner().into_iter().map(NonEmptyBz::into_inner).collect(),
		}
	}
}

impl From<Header> for HeaderGql {
	fn from(header: Header) -> Self {
		let header = header.dissolve();

		Self {
			chain_id: header.chain_id,
			height: header.height,
			time: jiff_to_chrono(&header.time).unwrap(),
			validators_hash: header.validators_hash.into_inner(),
			next_validators_hash: header.next_validators_hash.into_inner(),
			consensus_hash: header.consensus_hash.into_inner(),
			app_hash: header.app_hash.into_bytes(),
			proposer: header.proposer.as_bytes().to_vec(),
			last_commit_hash: header.last_commit_hash.map(Sha256::into_inner),
			data_hash: header.data_hash.map(Sha256::into_inner),
			last_results_hash: header.last_results_hash.map(Sha256::into_inner),
			evidence_hash: header.evidence_hash.map(Sha256::into_inner),
		}
	}
}

impl TryFrom<Tx> for TxGql {
	type Error = async_graphql::Error;

	fn try_from(tx: Tx) -> Result<Self, Self::Error> {
		let tx = tx.dissolve();

		let tx_gql = Self {
			block_height: tx.block_height,
			tx_idx_in_block: tx.tx_idx_in_block,
			tx_hash: tx.tx_hash.into_inner(),
			msgs: tx
				.msgs
				.into_inner()
				.into_iter()
				.map(serde_json::to_value)
				.map(|m| m.map(|m| m.to_string()))
				.collect::<Result<_, _>>()?,
			memo: tx.memo.map(Memo::into_inner),
			timeout_height: tx.timeout_height,
			signatures: tx.signatures,
			signers: signer_keys_to_json(tx.signers.into_iter())?,
			fees: tx
				.fees
				.map(Fees::into_inner)
				.map(|fees| fees.into_iter().map(From::from).collect()),
			payer: tx.payer.as_bytes().to_vec(),
			granter: tx.granter.map(|a| a.as_bytes().to_vec()),
			code: tx.code.into(),
			codespace: tx.codespace.map(Codespace::into_inner),
			gas_limit: tx.gas_limit,
			gas_wanted: tx.gas_wanted,
			gas_used: tx.gas_used,
			data_bz: tx.data_bz.map(NonEmptyBz::into_inner),
			tx_bz: tx.tx_bz.into_inner(),
		};

		Ok(tx_gql)
	}
}

impl From<TxSummary> for TxSummaryGql {
	fn from(summary: TxSummary) -> Self {
		let summary = summary.dissolve();

		Self {
			since_blocks_ago: summary.since_blocks_ago,
			start_block_height: summary.start_block_height,
			total_txs: summary.total_txs,
			total_gas_used: summary.total_gas_used,
			total_msgs: summary.total_msgs,
			total_signatures: summary.total_signatures,
		}
	}
}

impl From<FeeSummary> for FeeSummaryGql {
	fn from(summary: FeeSummary) -> Self {
		let summary = summary.dissolve();

		Self {
			since_blocks_ago: summary.since_blocks_ago,
			start_block_height: summary.start_block_height,
			total_fees: summary.total_fees.into(),
		}
	}
}

impl From<Coin> for CoinGql {
	fn from(Coin { amount, denom }: Coin) -> Self {
		Self { amount: amount.to_string(), denom: denom.to_string() }
	}
}

fn jiff_to_chrono(jiff: &Timestamp) -> Option<DateTime<Utc>> {
	let nanos = jiff.as_nanosecond();

	const NANOS_IN_ONE_SEC: i128 = 1_000_000_000;

	let secs = (nanos / NANOS_IN_ONE_SEC).try_into().ok()?;
	let sub_nanos = (nanos % NANOS_IN_ONE_SEC).try_into().ok()?;

	DateTime::from_timestamp(secs, sub_nanos)
}

fn signer_keys_to_json<I>(keys: I) -> Result<Vec<Value>, serde_json::Error>
where
	I: Iterator<Item = SignerPublicKey>,
{
	keys.map(Any::from).map(serde_json::to_value).collect()
}
