mod block;

use anyhow::Context;
use bon::Builder;
use chrono::{DateTime, Utc};
use diesel::prelude::{Insertable, Queryable};
use jiff::Timestamp;

use crate::{
	domain::{
		AppHash, ValidatorAddress,
		block::{Block, Header},
	},
	types::Sha256,
};

use super::schema;

#[derive(Debug, Queryable, Insertable, Builder)]
#[diesel(table_name = schema::block)]
pub struct BlockRecord {
	height: i64,
	block_hash: Vec<u8>,
	chain_id: String,
	time: DateTime<Utc>,
	app_hash: Vec<u8>,
	validators_hash: Vec<u8>,
	next_validators_hash: Vec<u8>,
	consensus_hash: Vec<u8>,
	proposer_address: Vec<u8>,
	last_commit_hash: Option<Vec<u8>>,
	data_hash: Option<Vec<u8>>,
	last_results_hash: Option<Vec<u8>>,
	evidence_hash: Option<Vec<u8>>,
	data: Option<Vec<Option<Vec<u8>>>>,
}

impl TryFrom<Block> for BlockRecord {
	type Error = anyhow::Error;

	fn try_from(block: Block) -> Result<Self, Self::Error> {
		let record = Self::builder()
			.height(block.header().height().try_into()?)
			.block_hash(block.hash().get().to_vec())
			.chain_id(block.header().chain_id().into())
			.time(jiff_to_chrono(*block.header().time())?)
			.app_hash(block.header().app_hash().get().to_vec())
			.validators_hash(block.header().validators_hash().get().to_vec())
			.next_validators_hash(block.header().next_validators_hash().get().to_vec())
			.consensus_hash(block.header().consensus_hash().get().to_vec())
			.proposer_address(block.header().proposer_address().get().to_vec())
			.maybe_last_commit_hash(block.header().last_commit_hash().map(|h| h.get().to_vec()))
			.maybe_data_hash(block.header().data_hash().map(|h| h.get().to_vec()))
			.maybe_last_results_hash(block.header().last_results_hash().map(|h| h.get().to_vec()))
			.maybe_evidence_hash(block.header().evidence_hash().map(|h| h.get().to_vec()))
			.data(block.data().iter().map(AsRef::as_ref).map(Vec::from).map(Some).collect())
			.build();

		Ok(record)
	}
}

impl TryFrom<BlockRecord> for Block {
	type Error = anyhow::Error;

	fn try_from(record: BlockRecord) -> Result<Self, Self::Error> {
		let header = Header::builder()
			.height(record.height.try_into()?)
			.chain_id(record.chain_id)
			.time(chrono_to_jiff(record.time))
			.validators_hash(bytes_to_sha256(&record.validators_hash)?)
			.next_validators_hash(bytes_to_sha256(&record.next_validators_hash)?)
			.consensus_hash(bytes_to_sha256(&record.consensus_hash)?)
			.app_hash(AppHash::new(record.app_hash))
			.proposer_address(ValidatorAddress::new(
				record.proposer_address.try_into().ok().context("invalid address")?,
			))
			.maybe_last_commit_hash(
				record.last_commit_hash.map(|h| bytes_to_sha256(&h)).transpose()?,
			)
			.maybe_data_hash(record.data_hash.map(|h| bytes_to_sha256(&h)).transpose()?)
			.maybe_last_results_hash(
				record.last_results_hash.map(|h| bytes_to_sha256(&h)).transpose()?,
			)
			.maybe_evidence_hash(record.evidence_hash.map(|h| bytes_to_sha256(&h)).transpose()?)
			.build();

		let data = record
			.data
			.map(|list| list.into_iter().collect::<Option<Vec<_>>>())
			.map(|list| list.context("constituent data must not be null"))
			.transpose()?
			.unwrap_or(vec![]);

		let block = Block::builder()
			.header(header)
			.hash(bytes_to_sha256(&record.block_hash)?)
			.data(data)
			.build();

		Ok(block)
	}
}

fn bytes_to_sha256(bytes: &[u8]) -> anyhow::Result<Sha256> {
	Ok(Sha256::new(
		bytes.try_into().context("sha256 must have exactly 32 bytes")?,
	))
}

fn jiff_to_chrono(jiff: Timestamp) -> anyhow::Result<DateTime<Utc>> {
	let nanos = jiff.as_nanosecond();

	const NANOS_IN_ONE_SEC: i128 = 1_000_000_000;

	let secs = (nanos / NANOS_IN_ONE_SEC).try_into()?;
	let sub_nanos = (nanos % NANOS_IN_ONE_SEC).try_into()?;

	DateTime::from_timestamp(secs, sub_nanos).context("failed to parse timestamp")
}

fn chrono_to_jiff(chrono: DateTime<Utc>) -> Timestamp {
	let secs = chrono.timestamp();
	let sub_nanos = chrono.timestamp_subsec_nanos().try_into().expect("sub nanos must be valid");
	Timestamp::new(secs, sub_nanos).expect("valid datetime must yield valid timestamp")
}
