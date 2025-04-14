use bon::Builder;
use jiff::Timestamp;

use crate::types::Sha256;

use super::{AppHash, ValidatorAddress};

#[derive(Debug, Clone, Builder)]
pub struct Block {
	header: Header,
	data: Vec<Vec<u8>>,
	hash: Sha256,
}

#[derive(Debug, Clone, Builder)]
pub struct Header {
	chain_id: String,
	height: u64,
	time: Timestamp,
	validators_hash: Sha256,
	next_validators_hash: Sha256,
	consensus_hash: Sha256,
	app_hash: AppHash,
	proposer_address: ValidatorAddress,
	last_commit_hash: Option<Sha256>,
	data_hash: Option<Sha256>,
	last_results_hash: Option<Sha256>,
	evidence_hash: Option<Sha256>,
}

impl Block {
	pub fn header(&self) -> &Header {
		&self.header
	}

	pub fn data(&self) -> &[impl AsRef<[u8]>] {
		&self.data
	}

	pub fn hash(&self) -> &Sha256 {
		&self.hash
	}
}

impl Header {
	pub fn chain_id(&self) -> &str {
		&self.chain_id
	}

	pub fn height(&self) -> u64 {
		self.height
	}

	pub fn time(&self) -> &Timestamp {
		&self.time
	}

	pub fn validators_hash(&self) -> &Sha256 {
		&self.validators_hash
	}

	pub fn next_validators_hash(&self) -> &Sha256 {
		&self.next_validators_hash
	}

	pub fn consensus_hash(&self) -> &Sha256 {
		&self.consensus_hash
	}

	pub fn app_hash(&self) -> &AppHash {
		&self.app_hash
	}

	pub fn proposer_address(&self) -> &ValidatorAddress {
		&self.proposer_address
	}

	pub fn last_commit_hash(&self) -> Option<&Sha256> {
		self.last_commit_hash.as_ref()
	}

	pub fn data_hash(&self) -> Option<&Sha256> {
		self.data_hash.as_ref()
	}

	pub fn last_results_hash(&self) -> Option<&Sha256> {
		self.last_results_hash.as_ref()
	}

	pub fn evidence_hash(&self) -> Option<&Sha256> {
		self.evidence_hash.as_ref()
	}
}
