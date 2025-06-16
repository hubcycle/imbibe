pub mod error;

mod backfill;
mod live;

pub use tendermint_rpc::WebSocketClient;

pub use self::{backfill::BackfillIndexer, live::LiveIndexer};

use core::num::NonZeroU64;

use base64::{Engine, prelude::BASE64_STANDARD};
use bytes::Bytes;
use cosmrs::{
	AccountId,
	tendermint::{Hash, abci::types::ExecTxResult, block::header::Header as TendermintBlockHeader},
	tx::SignerPublicKey,
};
use imbibe_domain::{
	Address, NonEmptyBz, Sha256,
	block::{AppHash, Block, BlockData, Header},
	tx::{Codespace, Fees, Memo, Msgs, Tx},
};
use jiff::Timestamp;
use sha2::Digest;

use self::error::{IndexerError, Result};

#[allow(private_bounds)]
pub trait Indexer: Sealed {
	type Output;

	fn start(self) -> impl Future<Output = Result<Self::Output>> + Send;
}

trait Sealed {}

fn process_block(
	header: TendermintBlockHeader,
	hash: Hash,
	data: Vec<Vec<u8>>,
	exec_tx_results: Vec<ExecTxResult>,
) -> Result<(Block, Vec<Tx>)> {
	if data.len() != exec_tx_results.len() {
		return Err(IndexerError::BlockData(format!(
			"mismatch in number of tx included in block {}",
			header.height
		)));
	}

	let height = header.height.value().try_into().map_err(|_| IndexerError::Height)?;
	let data = data
		.into_iter()
		.map(Bytes::from)
		.map(NonEmptyBz::new)
		.collect::<Option<Vec<_>>>()
		.and_then(BlockData::new)
		.ok_or(IndexerError::BlockData("invalid block data".into()))?;

	let (total_gas_used, tx_results) =
		process_block_txs(height, data.get().iter().cloned().zip(exec_tx_results))?;

	let block = Block::builder()
		.header(make_header(header)?)
		.hash(make_sha256(hash).ok_or(IndexerError::BlockHash)?)
		.data(data)
		.gas_used(total_gas_used)
		.build();

	Ok((block, tx_results))
}

fn process_block_txs<I>(height: NonZeroU64, tbr: I) -> Result<(u64, Vec<Tx>)>
where
	I: Iterator<Item = (NonEmptyBz<Bytes>, ExecTxResult)>,
{
	let mut total_gas_used = 0u64;
	let txrs = tbr
		.into_iter()
		.enumerate()
		.map(|(idx, (bz, res))| {
			idx.try_into()
				.map_err(|_| IndexerError::TxsInBlock)
				.and_then(|idx| make_tx(height, idx, bz, res))
		})
		.map(|tx| {
			let tx = tx?;
			total_gas_used = total_gas_used.checked_add(tx.gas_used()).ok_or(IndexerError::Gas)?;
			Ok(tx)
		})
		.collect::<Result<_>>()?;

	Ok((total_gas_used, txrs))
}

fn make_header(tm_header: TendermintBlockHeader) -> Result<Header> {
	let header = Header::builder()
		.chain_id(tm_header.chain_id.into())
		.height(tm_header.height.value())
		.time(Timestamp::from_nanosecond(
			tm_header.time.unix_timestamp_nanos(),
		)?)
		.validators_hash(make_sha256(tm_header.validators_hash).ok_or(IndexerError::ValidatorHash)?)
		.next_validators_hash(
			make_sha256(tm_header.next_validators_hash).ok_or(IndexerError::NextValidatorsHash)?,
		)
		.consensus_hash(make_sha256(tm_header.consensus_hash).ok_or(IndexerError::ConsensusHash)?)
		.app_hash(AppHash::new(tm_header.app_hash.into()))
		.proposer(
			tm_header
				.proposer_address
				.as_bytes()
				.try_into()
				.map(Address::new)
				.expect("proposer address must be exactly 20 bytes long"),
		)
		.maybe_last_commit_hash(tm_header.last_commit_hash.and_then(make_sha256))
		.maybe_data_hash(tm_header.data_hash.and_then(make_sha256))
		.maybe_last_results_hash(tm_header.last_results_hash.and_then(make_sha256))
		.maybe_evidence_hash(tm_header.evidence_hash.and_then(make_sha256))
		.build();

	Ok(header)
}

fn make_sha256(hash: Hash) -> Option<Sha256> {
	match hash {
		Hash::Sha256(h) => Some(Sha256::new(h)),
		Hash::None => None,
	}
}

fn make_tx(
	block_height: NonZeroU64,
	tx_idx_in_block: u64,
	tx_bz: NonEmptyBz<Bytes>,
	exec_tx_result: ExecTxResult,
) -> Result<Tx> {
	let cosm_tx =
		cosmrs::Tx::from_bytes(tx_bz.as_ref()).map_err(|_| IndexerError::TxDecodeError)?;

	let payer = match cosm_tx.auth_info.fee.payer {
		Some(payer) => payer
			.to_bytes()
			.as_slice()
			.try_into()
			.map(Address::new)
			.map_err(|_| IndexerError::Address)?,
		None => match cosm_tx.auth_info.signer_infos.first().map(|s| s.public_key.as_ref()) {
			Some(Some(pk)) => signer_address(pk)?,
			Some(None) => return Err(IndexerError::Signer("signer must have public key".into())),

			// Extract first signer from tx messages
			None => cosm_tx
				.body
				.messages
				.iter()
				.map(imbibe_protos::signers_from_any_msg)
				.flat_map(Result::ok)
				.flat_map(|mut signers| signers.next())
				.next()
				.map(|s| s.parse::<AccountId>().map_err(|_| IndexerError::Bech32Address(s)))
				.transpose()?
				.map(|acc_id| acc_id.to_bytes())
				.as_deref()
				.map(|bz| bz.try_into().map(Address::new).map_err(|_| IndexerError::Address))
				.transpose()?
				.ok_or(IndexerError::Signer(
					"at least one msg must contain signer when no signer info provided".into(),
				))?,
		},
	};

	let tx = Tx::builder()
		.block_height(block_height)
		.tx_idx_in_block(tx_idx_in_block)
		.tx_hash(sha2::Sha256::digest(&tx_bz).into())
		.msgs(Msgs::new(cosm_tx.body.messages).ok_or(IndexerError::TxMsgsMissing)?)
		.maybe_memo(Memo::new(cosm_tx.body.memo))
		.maybe_timeout_height(cosm_tx.body.timeout_height.value().try_into().ok())
		.signatures(cosm_tx.signatures)
		.signers(cosm_tx.auth_info.signer_infos.into_iter().flat_map(|si| si.public_key).collect())
		.maybe_fees(Fees::new(cosm_tx.auth_info.fee.amount))
		.payer(payer)
		.maybe_granter(
			cosm_tx
				.auth_info
				.fee
				.granter
				.map(|acc_id| acc_id.to_bytes())
				.as_deref()
				.map(|bz| bz.try_into().map(Address::new).map_err(|_| IndexerError::Address))
				.transpose()?,
		)
		.code(exec_tx_result.code)
		.maybe_codespace(Codespace::new(exec_tx_result.codespace))
		.gas_limit(cosm_tx.auth_info.fee.gas_limit)
		.gas_wanted(exec_tx_result.gas_wanted.try_into().map_err(|_| IndexerError::Gas)?)
		.gas_used(exec_tx_result.gas_used.try_into().map_err(|_| IndexerError::Gas)?)
		.maybe_data_bz(
			BASE64_STANDARD
				.decode(exec_tx_result.data)
				.map(From::from)
				.map(NonEmptyBz::new)
				.map_err(|_| IndexerError::TxDataDecodeError)?,
		)
		.tx_bz(tx_bz)
		.build();

	Ok(tx)
}

fn signer_address(signer: &SignerPublicKey) -> Result<Address> {
	match signer {
		SignerPublicKey::Single(pk) => pk
			.account_id("foo")
			.map_err(|_| IndexerError::UnsupportedPublicKey)?
			.to_bytes()
			.try_into()
			.map(Address::new)
			.map_err(|_| IndexerError::Address),

		#[cfg_attr(not(feature = "ethsecp256k1"), allow(unused_variables))]
		SignerPublicKey::Any(any) => {
			#[cfg(not(feature = "ethsecp256k1"))]
			return Err(IndexerError::UnsupportedPublicKey);

			#[cfg(feature = "ethsecp256k1")]
			any.to_msg().map_err(|_| IndexerError::UnsupportedPublicKey).and_then(
				|imbibe_protos::ethermint::crypto::v1::ethsecp256k1::PubKey { key }| {
					use k256::elliptic_curve::sec1::ToEncodedPoint;

					k256::PublicKey::from_sec1_bytes(&key)
						.map_err(|e| IndexerError::Other(e.into()))?
						.to_encoded_point(false)
						.as_bytes()
						.get(1..)
						.map(sha3::Keccak256::digest)
						.ok_or(IndexerError::Other("invalid key".into()))?
						.get(12..)
						.map(TryFrom::try_from)
						.transpose()
						.map(|addr| addr.expect("keccak must be 32 bytes long"))
						.map(Address::new)
						.map_err(|_| IndexerError::Address)
				},
			)
		},
		_ => Err(IndexerError::UnsupportedPublicKey),
	}
}
