use bigdecimal::{BigDecimal, ToPrimitive};
use bon::Builder;
use chrono::{DateTime, Utc};
use cosmrs::{
	Any, Coin,
	tx::{SignatureBytes, SignerPublicKey},
};
use diesel::prelude::Queryable;
use imbibe_domain::{
	Address, NonEmptyBz, Sha256,
	block::{AppHash, Block, BlockData, Header},
	tx::{Codespace, Fees, Memo, Tx},
};
use serde_json::Value;

use super::error::InvalidValueError;

#[derive(Debug, Queryable)]
pub struct BlockWithDataRecord {
	block: BlockRecord,
	data: Vec<Vec<u8>>,
}

#[derive(Debug, Queryable)]
pub struct BlockRecord {
	height: i64,
	block_hash: Vec<u8>,
	chain_id: String,
	time: DateTime<Utc>,
	app_hash: Vec<u8>,
	validators_hash: Vec<u8>,
	next_validators_hash: Vec<u8>,
	consensus_hash: Vec<u8>,
	proposer: Vec<u8>,
	gas_used: i64,
	last_commit_hash: Option<Vec<u8>>,
	data_hash: Option<Vec<u8>>,
	last_results_hash: Option<Vec<u8>>,
	evidence_hash: Option<Vec<u8>>,
}

#[derive(Debug, Builder)]
pub struct TxWithDetailsRecord {
	tx: TxRecord,
	signatures: Vec<SignatureBytes>,
	fees: Vec<Coin>,
	msgs: Vec<Any>,
}

#[derive(Debug, Queryable)]
pub struct TxRecord {
	block_height: i64,
	tx_idx_in_block: i64,
	tx_hash: Vec<u8>,
	memo: Option<String>,
	timeout_height: Option<i64>,
	signers: Value,
	payer: Vec<u8>,
	granter: Option<Vec<u8>>,
	gas_limit: i64,
	gas_wanted: i64,
	gas_used: i64,
	code: i32,
	codespace: Option<String>,
	data_bz: Option<Vec<u8>>,
	tx_bz: Vec<u8>,
}

#[derive(Debug, Queryable)]
pub struct SignatureRecord {
	bz: Vec<u8>,
}

#[derive(Debug, Queryable)]
pub struct FeeRecord {
	amount: BigDecimal,
	denom: String,
}

#[derive(Debug, Queryable)]
pub struct MsgRecord {
	type_url: String,
	value: Vec<u8>,
}

impl TxRecord {
	pub fn block_height(&self) -> i64 {
		self.block_height
	}

	pub fn tx_idx_in_block(&self) -> i64 {
		self.tx_idx_in_block
	}
}

impl SignatureRecord {
	pub fn into_bytes(self) -> SignatureBytes {
		self.bz
	}
}

impl TryFrom<BlockWithDataRecord> for Block {
	type Error = InvalidValueError;

	fn try_from(record: BlockWithDataRecord) -> Result<Self, Self::Error> {
		let block_record = record.block;

		let header = Header::builder()
			.chain_id(block_record.chain_id)
			.height(block_record.height.try_into()?)
			.time(super::chrono_to_jiff(&block_record.time))
			.validators_hash(Sha256::new(
				block_record.validators_hash.as_slice().try_into()?,
			))
			.next_validators_hash(Sha256::new(
				block_record.next_validators_hash.as_slice().try_into()?,
			))
			.consensus_hash(Sha256::new(
				block_record.consensus_hash.as_slice().try_into()?,
			))
			.app_hash(AppHash::new(block_record.app_hash))
			.proposer(Address::new(block_record.proposer.as_slice().try_into()?))
			.maybe_last_commit_hash(
				block_record
					.last_commit_hash
					.map(|h| h.as_slice().try_into().map(Sha256::new))
					.transpose()?,
			)
			.maybe_data_hash(
				block_record
					.data_hash
					.map(|h| h.as_slice().try_into().map(Sha256::new))
					.transpose()?,
			)
			.maybe_last_results_hash(
				block_record
					.last_results_hash
					.map(|h| h.as_slice().try_into().map(Sha256::new))
					.transpose()?,
			)
			.maybe_evidence_hash(
				block_record
					.evidence_hash
					.map(|h| h.as_slice().try_into().map(Sha256::new))
					.transpose()?,
			)
			.build();

		let block = Block::builder()
			.header(header)
			.hash(Sha256::new(block_record.block_hash.as_slice().try_into()?))
			.gas_used(block_record.gas_used.try_into()?)
			.data(
				record
					.data
					.into_iter()
					.map(From::from)
					.map(NonEmptyBz::new)
					.collect::<Option<_>>()
					.and_then(BlockData::new)
					.ok_or(InvalidValueError::Empty)?,
			)
			.build();

		Ok(block)
	}
}

impl TryFrom<TxWithDetailsRecord> for Tx {
	type Error = InvalidValueError;

	fn try_from(record: TxWithDetailsRecord) -> Result<Self, Self::Error> {
		let txr = record.tx;
		let tx = Tx::builder()
			.block_height(u64::try_from(txr.block_height).and_then(|h| h.try_into())?)
			.tx_idx_in_block(txr.tx_idx_in_block.try_into()?)
			.tx_hash(
				txr.tx_hash
					.as_slice()
					.try_into()
					.map(Sha256::new)
					.map_err(InvalidValueError::from)?,
			)
			.msgs(imbibe_domain::tx::Msgs::new(record.msgs).ok_or(InvalidValueError::Empty)?)
			.maybe_memo(txr.memo.and_then(Memo::new))
			.maybe_timeout_height(
				txr.timeout_height
					.map(u64::try_from)
					.transpose()?
					.map(TryFrom::try_from)
					.transpose()?,
			)
			.signatures(record.signatures)
			.maybe_fees(Fees::new(record.fees))
			.signers(json_to_signer_keys(txr.signers)?)
			.payer(txr.payer.as_slice().try_into().map(Address::new)?)
			.maybe_granter(
				txr.granter.as_deref().map(TryFrom::try_from).transpose()?.map(Address::new),
			)
			.code(u32::try_from(txr.code)?.into())
			.maybe_codespace(txr.codespace.and_then(Codespace::new))
			.gas_limit(txr.gas_limit.try_into()?)
			.gas_wanted(txr.gas_wanted.try_into()?)
			.gas_used(txr.gas_used.try_into()?)
			.maybe_data_bz(txr.data_bz.map(From::from).and_then(NonEmptyBz::new))
			.tx_bz(NonEmptyBz::new(txr.tx_bz.into()).ok_or(InvalidValueError::Empty)?)
			.build();

		Ok(tx)
	}
}

impl TryFrom<&FeeRecord> for Coin {
	type Error = InvalidValueError;

	fn try_from(FeeRecord { amount, denom, .. }: &FeeRecord) -> Result<Self, Self::Error> {
		amount
			.to_u128()
			.ok_or(InvalidValueError::AmountError)
			.and_then(|amount| Coin::new(amount, denom).map_err(From::from))
	}
}

impl From<MsgRecord> for Any {
	fn from(MsgRecord { type_url, value, .. }: MsgRecord) -> Self {
		Any { type_url, value }
	}
}

fn json_to_signer_keys(json: Value) -> Result<Vec<SignerPublicKey>, InvalidValueError> {
	match json {
		Value::Array(keys) => keys
			.into_iter()
			.map(serde_json::from_value::<Any>)
			.map(|any| any.map(SignerPublicKey::try_from).map_err(From::from))
			.map(|spk| spk.and_then(|spk| spk.map_err(InvalidValueError::from)))
			.collect(),
		_ => Err(InvalidValueError::Other(
			"signer keys must be array of keys".into(),
		))?,
	}
}
