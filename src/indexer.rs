pub mod historical;
pub mod live;

use anyhow::Context;
use jiff::Timestamp;
use tendermint::{Hash, block::header};

use crate::{
	domain::{AppHash, ValidatorAddress, block::Header},
	types::Sha256,
};

fn make_header(header: header::Header) -> anyhow::Result<Header> {
	let header = Header::builder()
		.chain_id(header.chain_id.into())
		.height(header.height.value())
		.time(Timestamp::from_nanosecond(
			header.time.unix_timestamp_nanos(),
		)?)
		.validators_hash(make_sha256(header.validators_hash).context("missing validators hash")?)
		.next_validators_hash(
			make_sha256(header.next_validators_hash).context("missing next validators hash")?,
		)
		.consensus_hash(make_sha256(header.consensus_hash).context("missing consensus hash")?)
		.app_hash(AppHash::new(header.app_hash.into()))
		.proposer_address(ValidatorAddress::new(
			Vec::from(header.proposer_address)
				.try_into()
				.ok()
				.context("invalid validator address")?,
		))
		.maybe_last_commit_hash(header.last_commit_hash.and_then(make_sha256))
		.maybe_data_hash(header.data_hash.and_then(make_sha256))
		.maybe_last_results_hash(header.last_results_hash.and_then(make_sha256))
		.maybe_evidence_hash(header.evidence_hash.and_then(make_sha256))
		.build();

	Ok(header)
}

fn make_sha256(hash: Hash) -> Option<Sha256> {
	match hash {
		Hash::Sha256(h) => Some(Sha256::new(h)),
		Hash::None => None,
	}
}
