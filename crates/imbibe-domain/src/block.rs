use core::{
	fmt::{self, Debug, Formatter},
	num::NonZeroU64,
};

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
	height: NonZeroU64,
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

pub struct BlockDissolved<T> {
	pub header: Header,
	pub gas_used: u64,
	pub hash: Sha256,
	pub data: BlockData<T>,
}

pub struct HeaderDissolved {
	pub chain_id: String,
	pub height: NonZeroU64,
	pub time: Timestamp,
	pub validators_hash: Sha256,
	pub next_validators_hash: Sha256,
	pub consensus_hash: Sha256,
	pub app_hash: AppHash,
	pub proposer: Address,
	pub last_commit_hash: Option<Sha256>,
	pub data_hash: Option<Sha256>,
	pub last_results_hash: Option<Sha256>,
	pub evidence_hash: Option<Sha256>,
}

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AppHash(Vec<u8>);

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct BlockData<T>(Vec<NonEmptyBz<T>>);

impl<T> Block<T> {
	pub fn header(&self) -> &Header {
		&self.header
	}

	pub fn data(&self) -> &BlockData<T> {
		&self.data
	}

	pub fn gas_used(&self) -> u64 {
		self.gas_used
	}

	pub fn hash(&self) -> &Sha256 {
		&self.hash
	}

	pub fn dissolve(self) -> BlockDissolved<T> {
		BlockDissolved {
			header: self.header,
			gas_used: self.gas_used,
			hash: self.hash,
			data: self.data,
		}
	}
}

impl Header {
	pub fn chain_id(&self) -> &str {
		&self.chain_id
	}

	pub fn height(&self) -> NonZeroU64 {
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

	pub fn dissolve(self) -> HeaderDissolved {
		HeaderDissolved {
			chain_id: self.chain_id,
			height: self.height,
			time: self.time,
			validators_hash: self.validators_hash,
			next_validators_hash: self.next_validators_hash,
			consensus_hash: self.consensus_hash,
			app_hash: self.app_hash,
			proposer: self.proposer,
			last_commit_hash: self.last_commit_hash,
			data_hash: self.data_hash,
			last_results_hash: self.last_results_hash,
			evidence_hash: self.evidence_hash,
		}
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

	pub fn into_inner(self) -> Vec<NonEmptyBz<T>> {
		self.0
	}
}

impl Debug for AppHash {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		write!(f, "AppHash({})", const_hex::encode(self.get()))
	}
}
