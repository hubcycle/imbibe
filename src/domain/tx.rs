use core::num::NonZeroU64;

use bon::Builder;
use bytes::Bytes;
use cosmrs::{
	AccountId, Any, Coin,
	tx::{SignatureBytes, SignerPublicKey},
};
use tendermint::abci::Code;

use crate::types::Sha256;

#[derive(Debug, Clone, Builder)]
pub struct TxResult {
	tx_hash: Sha256,
	block_height: NonZeroU64,
	msgs: Vec<Any>,
	memo: String,
	timeout_height: Option<NonZeroU64>,
	signatures: Vec<SignatureBytes>,
	signers: Vec<SignerPublicKey>,
	fee: Vec<Coin>,
	payer: AccountId,
	granter: Option<AccountId>,
	code: Code,
	codespace: String,
	gas_limit: u64,
	gas_wanted: u64,
	gas_used: u64,
	data_bz: Bytes,
	tx_bz: Bytes,
}

impl TxResult {
	pub fn tx_hash(&self) -> &Sha256 {
		&self.tx_hash
	}

	pub fn block_height(&self) -> NonZeroU64 {
		self.block_height
	}

	pub fn msgs(&self) -> &[Any] {
		&self.msgs
	}

	pub fn memo(&self) -> &str {
		&self.memo
	}

	pub fn timeout_height(&self) -> Option<NonZeroU64> {
		self.timeout_height
	}

	pub fn signatures(&self) -> &[SignatureBytes] {
		&self.signatures
	}

	pub fn signers(&self) -> &[SignerPublicKey] {
		&self.signers
	}

	pub fn fee(&self) -> &[Coin] {
		&self.fee
	}

	pub fn payer(&self) -> &AccountId {
		&self.payer
	}

	pub fn granter(&self) -> Option<&AccountId> {
		self.granter.as_ref()
	}

	pub fn code(&self) -> Code {
		self.code
	}

	pub fn codespace(&self) -> &str {
		&self.codespace
	}

	pub fn gas_limit(&self) -> u64 {
		self.gas_limit
	}

	pub fn gas_wanted(&self) -> u64 {
		self.gas_wanted
	}

	pub fn gas_used(&self) -> u64 {
		self.gas_used
	}

	pub fn data_bz(&self) -> &Bytes {
		&self.data_bz
	}

	pub fn tx_bz(&self) -> &Bytes {
		&self.tx_bz
	}
}
