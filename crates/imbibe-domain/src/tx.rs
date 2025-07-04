use core::{
	fmt::{self, Formatter},
	num::NonZeroU64,
};

use bon::Builder;
use bytes::Bytes;
use cosmrs::{
	Any, Coin,
	tendermint::abci::Code,
	tx::{SignatureBytes, SignerPublicKey},
};

#[cfg(feature = "serde")]
use serde::{Deserialize, Deserializer, Serialize, Serializer, de};

use super::{Address, NonEmptyBz, Sha256};

#[derive(Debug, Clone, Builder)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Tx<DBZ = Bytes, TBZ = Bytes> {
	block_height: NonZeroU64,
	tx_idx_in_block: u64,
	tx_hash: Sha256,
	msgs: Msgs,
	memo: Option<Memo>,
	timeout_height: Option<NonZeroU64>,
	signatures: Vec<SignatureBytes>,

	#[cfg_attr(
		feature = "serde",
		serde(
			serialize_with = "serialize_signers",
			deserialize_with = "deserialize_signers"
		)
	)]
	signers: Vec<SignerPublicKey>,

	fees: Option<Fees>,
	payer: Address,
	granter: Option<Address>,
	code: Code,
	codespace: Option<Codespace>,
	gas_limit: u64,
	gas_wanted: u64,
	gas_used: u64,
	data_bz: Option<NonEmptyBz<DBZ>>,
	tx_bz: NonEmptyBz<TBZ>,
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Memo(String);

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Codespace(String);

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Fees(Vec<Coin>);

#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Msgs(Vec<Any>);

pub struct TxDissolved<DBZ, TBZ> {
	pub block_height: NonZeroU64,
	pub tx_idx_in_block: u64,
	pub tx_hash: Sha256,
	pub msgs: Msgs,
	pub memo: Option<Memo>,
	pub timeout_height: Option<NonZeroU64>,
	pub signatures: Vec<SignatureBytes>,
	pub signers: Vec<SignerPublicKey>,
	pub fees: Option<Fees>,
	pub payer: Address,
	pub granter: Option<Address>,
	pub code: Code,
	pub codespace: Option<Codespace>,
	pub gas_limit: u64,
	pub gas_wanted: u64,
	pub gas_used: u64,
	pub data_bz: Option<NonEmptyBz<DBZ>>,
	pub tx_bz: NonEmptyBz<TBZ>,
}

impl<DBZ, TBZ> Tx<DBZ, TBZ> {
	pub fn block_height(&self) -> NonZeroU64 {
		self.block_height
	}

	pub fn tx_idx_in_block(&self) -> u64 {
		self.tx_idx_in_block
	}

	pub fn tx_hash(&self) -> &Sha256 {
		&self.tx_hash
	}

	pub fn msgs(&self) -> &Msgs {
		&self.msgs
	}

	pub fn memo(&self) -> Option<&Memo> {
		self.memo.as_ref()
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

	pub fn fees(&self) -> Option<&Fees> {
		self.fees.as_ref()
	}

	pub fn payer(&self) -> &Address {
		&self.payer
	}

	pub fn granter(&self) -> Option<&Address> {
		self.granter.as_ref()
	}

	pub fn code(&self) -> Code {
		self.code
	}

	pub fn codespace(&self) -> Option<&Codespace> {
		self.codespace.as_ref()
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

	pub fn data_bz(&self) -> Option<&NonEmptyBz<DBZ>> {
		self.data_bz.as_ref()
	}

	pub fn tx_bz(&self) -> &NonEmptyBz<TBZ> {
		&self.tx_bz
	}

	pub fn dissolve(self) -> TxDissolved<DBZ, TBZ> {
		TxDissolved {
			block_height: self.block_height,
			tx_idx_in_block: self.tx_idx_in_block,
			tx_hash: self.tx_hash,
			msgs: self.msgs,
			memo: self.memo,
			timeout_height: self.timeout_height,
			signatures: self.signatures,
			signers: self.signers,
			fees: self.fees,
			payer: self.payer,
			granter: self.granter,
			code: self.code,
			codespace: self.codespace,
			gas_limit: self.gas_limit,
			gas_wanted: self.gas_wanted,
			gas_used: self.gas_used,
			data_bz: self.data_bz,
			tx_bz: self.tx_bz,
		}
	}
}

impl Memo {
	pub fn new(memo: String) -> Option<Self> {
		(!memo.is_empty()).then_some(memo).map(Self)
	}

	pub fn get(&self) -> &str {
		&self.0
	}

	pub fn into_inner(self) -> String {
		self.0
	}
}

impl Codespace {
	pub fn new(memo: String) -> Option<Self> {
		(!memo.is_empty()).then_some(memo).map(Self)
	}

	pub fn get(&self) -> &str {
		&self.0
	}

	pub fn into_inner(self) -> String {
		self.0
	}
}

impl Fees {
	pub fn new(fees: Vec<Coin>) -> Option<Self> {
		(!fees.is_empty()).then_some(Self(fees))
	}

	pub fn get(&self) -> &[Coin] {
		&self.0
	}

	pub fn into_inner(self) -> Vec<Coin> {
		self.0
	}
}

impl Msgs {
	pub fn new(msgs: Vec<Any>) -> Option<Self> {
		(!msgs.is_empty()).then_some(Self(msgs))
	}

	pub fn get(&self) -> &[Any] {
		&self.0
	}

	pub fn into_inner(self) -> Vec<Any> {
		self.0
	}
}

impl AsRef<str> for Memo {
	fn as_ref(&self) -> &str {
		&self.0
	}
}

impl From<Memo> for String {
	fn from(memo: Memo) -> Self {
		memo.0
	}
}

impl AsRef<str> for Codespace {
	fn as_ref(&self) -> &str {
		&self.0
	}
}

impl From<Codespace> for String {
	fn from(codespace: Codespace) -> Self {
		codespace.0
	}
}

impl fmt::Debug for Msgs {
	fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
		let msgs: Vec<_> = self
			.0
			.iter()
			.map(|any| {
				format!(
					"Any {{ type_url: \"{}\", value: {:?} }}",
					any.type_url,
					Bytes::copy_from_slice(&any.value)
				)
			})
			.collect();
		f.debug_struct("Msgs").field("msgs", &msgs).finish()
	}
}

#[cfg(feature = "serde")]
fn serialize_signers<S>(signers: &[SignerPublicKey], serializer: S) -> Result<S::Ok, S::Error>
where
	S: Serializer,
{
	signers.iter().cloned().map(Any::from).collect::<Vec<_>>().serialize(serializer)
}

#[cfg(feature = "serde")]
fn deserialize_signers<'de, D>(deserializer: D) -> Result<Vec<SignerPublicKey>, D::Error>
where
	D: Deserializer<'de>,
{
	Vec::<Any>::deserialize(deserializer)?
		.into_iter()
		.map(TryFrom::try_from)
		.collect::<Result<_, _>>()
		.map_err(|e| de::Error::custom(format!("failed to convert Any to SignerPublicKey: {e:?}")))
}
