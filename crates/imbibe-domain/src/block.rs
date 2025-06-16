use bon::Builder;
use bytes::Bytes;
use jiff::Timestamp;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use super::{Address, NonEmptyBz, Sha256};

#[derive(Debug, Clone, Builder)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Block<T = Bytes> {
	header: Header,
	gas_used: u64,
	hash: Sha256,
	data: BlockData<T>,
}

#[derive(Debug, Clone, Builder)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Header {
	chain_id: String,
	height: u64,
	time: Timestamp,
	validators_hash: Sha256,
	next_validators_hash: Sha256,
	consensus_hash: Sha256,
	app_hash: AppHash,
	proposer: Address,
	last_commit_hash: Option<Sha256>,
	data_hash: Option<Sha256>,
	last_results_hash: Option<Sha256>,
	evidence_hash: Option<Sha256>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AppHash(Vec<u8>);

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BlockData<T>(Vec<NonEmptyBz<T>>);

impl Block {
	pub fn header(&self) -> &Header {
		&self.header
	}

	pub fn data(&self) -> &BlockData<Bytes> {
		&self.data
	}

	pub fn gas_used(&self) -> u64 {
		self.gas_used
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

	pub fn proposer(&self) -> &Address {
		&self.proposer
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

impl AppHash {
	pub const fn new(hash: Vec<u8>) -> Self {
		Self(hash)
	}

	pub fn get(&self) -> &[u8] {
		self.0.as_slice()
	}

	pub fn into_bytes(self) -> Vec<u8> {
		self.0
	}
}

impl<T> BlockData<T> {
	pub fn new(data: Vec<NonEmptyBz<T>>) -> Option<Self> {
		Some(Self(data))
	}

	pub fn get(&self) -> &[NonEmptyBz<T>] {
		&self.0
	}
}
