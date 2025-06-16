use bigdecimal::BigDecimal;
use bon::Builder;
use chrono::{DateTime, Utc};
use cosmrs::{Any, tx::SignerPublicKey};
use diesel::prelude::Insertable;
use imbibe_domain::{Address, block::Block, tx::Tx};
use serde_json::Value;

use crate::schema;

use super::error::InvalidValueError;

#[derive(Debug, Insertable, Builder)]
#[diesel(table_name = schema::block)]
pub struct NewBlockRecord<'a> {
	height: i64,
	block_hash: &'a [u8],
	chain_id: &'a str,
	time: DateTime<Utc>,
	app_hash: &'a [u8],
	validators_hash: &'a [u8],
	next_validators_hash: &'a [u8],
	consensus_hash: &'a [u8],
	proposer: &'a [u8],
	gas_used: i64,
	last_commit_hash: Option<&'a [u8]>,
	data_hash: Option<&'a [u8]>,
	last_results_hash: Option<&'a [u8]>,
	evidence_hash: Option<&'a [u8]>,
}

#[derive(Insertable, Builder)]
#[diesel(table_name = schema::tx)]
pub struct NewTxRecord<'a> {
	block_height: i64,
	tx_idx_in_block: i64,

	tx_hash: &'a [u8],

	memo: Option<&'a str>,
	timeout_height: Option<i64>,

	signers: Value,
	payer: &'a [u8],
	granter: Option<&'a [u8]>,
	gas_limit: i64,
	gas_wanted: i64,
	gas_used: i64,

	code: i32,
	codespace: Option<&'a str>,
	data_bz: Option<&'a [u8]>,
	tx_bz: &'a [u8],
}

#[derive(Insertable, Builder)]
#[diesel(table_name = schema::signature)]
pub struct NewSignatureRecord<'a> {
	block_height: i64,
	tx_idx_in_block: i64,
	signature_idx_in_tx: i64,

	bz: &'a [u8],
}

#[derive(Insertable, Builder)]
#[diesel(table_name = schema::fee)]
pub struct NewFeeRecord<'a> {
	block_height: i64,
	tx_idx_in_block: i64,
	fee_idx_in_tx: i64,

	amount: BigDecimal,
	denom: &'a str,
}

#[derive(Insertable, Builder)]
#[diesel(table_name = schema::msg)]
pub struct NewMsgRecord<'a> {
	block_height: i64,
	tx_idx_in_block: i64,
	msg_idx_in_tx: i64,

	type_url: &'a str,
	value: &'a [u8],
}

impl<'a> TryFrom<&'a Block> for NewBlockRecord<'a> {
	type Error = InvalidValueError;

	fn try_from(block: &'a Block) -> Result<Self, Self::Error> {
		let record = Self::builder()
			.height(block.header().height().try_into()?)
			.block_hash(block.hash().get())
			.chain_id(block.header().chain_id())
			.time(super::jiff_to_chrono(block.header().time()).ok_or(InvalidValueError::Time)?)
			.app_hash(block.header().app_hash().get())
			.validators_hash(block.header().validators_hash().get())
			.next_validators_hash(block.header().next_validators_hash().get())
			.consensus_hash(block.header().consensus_hash().get())
			.proposer(block.header().proposer().as_bytes())
			.gas_used(block.gas_used().try_into()?)
			.maybe_last_commit_hash(block.header().last_commit_hash().map(|h| h.get().as_slice()))
			.maybe_data_hash(block.header().data_hash().map(|h| h.get().as_slice()))
			.maybe_last_results_hash(block.header().last_results_hash().map(|h| h.get().as_slice()))
			.maybe_evidence_hash(block.header().evidence_hash().map(|h| h.get().as_slice()))
			.build();

		Ok(record)
	}
}

impl<'a> TryFrom<&'a Tx> for NewTxRecord<'a> {
	type Error = InvalidValueError;

	fn try_from(tx_result: &'a Tx) -> Result<Self, Self::Error> {
		let tx_record = NewTxRecord::builder()
			.block_height(tx_result.block_height().get().try_into()?)
			.tx_idx_in_block(tx_result.tx_idx_in_block().try_into()?)
			.tx_hash(tx_result.tx_hash().get().as_slice())
			.maybe_memo(tx_result.memo().map(AsRef::as_ref))
			.maybe_timeout_height(
				tx_result.timeout_height().map(|th| th.get().try_into()).transpose()?,
			)
			.signers(signer_keys_to_json(tx_result.signers().iter().cloned())?)
			.payer(tx_result.payer().as_bytes())
			.maybe_granter(tx_result.granter().map(Address::as_bytes))
			.gas_limit(tx_result.gas_limit().try_into()?)
			.gas_wanted(tx_result.gas_wanted().try_into()?)
			.gas_used(tx_result.gas_used().try_into()?)
			.code(tx_result.code().value().try_into()?)
			.maybe_codespace(tx_result.codespace().map(AsRef::as_ref))
			.maybe_data_bz(tx_result.data_bz().map(AsRef::as_ref))
			.tx_bz(tx_result.tx_bz().get())
			.build();

		Ok(tx_record)
	}
}

fn signer_keys_to_json<I>(keys: I) -> Result<Value, serde_json::Error>
where
	I: Iterator<Item = SignerPublicKey>,
{
	keys.map(Any::from).map(serde_json::to_value).collect::<Result<_, _>>().map(Value::Array)
}
