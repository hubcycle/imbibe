pub mod historical;
pub mod live;

use core::num::NonZeroU64;

use anyhow::Context;
use bytes::Bytes;
use cosmrs::{AccountId, tx::SignerPublicKey};
use jiff::Timestamp;
use k256::{PublicKey, elliptic_curve::sec1::ToEncodedPoint};
use sha2::Digest;
use sha3::Keccak256;
use tendermint::{Hash, abci::types::ExecTxResult, block::header};

use crate::{
	domain::{
		AppHash, ValidatorAddress,
		block::{Block, Header},
		tx::TxResult,
	},
	proto::{self, ethermint::crypto::v1::ethsecp256k1::PubKey as EthSecps256K1PubKey},
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

fn process_block(
	hrp: &str,
	header: tendermint::block::header::Header,
	hash: Hash,
	data: Vec<Vec<u8>>,
	exec_tx_results: Vec<ExecTxResult>,
) -> anyhow::Result<(Block, Vec<TxResult>)> {
	if data.len() != exec_tx_results.len() {
		anyhow::bail!(
			"mismatch in number of tx included in block {}",
			header.height
		);
	}

	let height = header.height.value().try_into()?;
	let data: Vec<_> = data.into_iter().map(Bytes::from).collect();
	let (total_gas_used, tx_results) =
		process_block_txs(hrp, height, data.iter().cloned().zip(exec_tx_results))?;

	let block = Block::builder()
		.header(make_header(header)?)
		.hash(make_sha256(hash).context("missing block hash")?)
		.data(data)
		.gas_used(total_gas_used)
		.build();

	Ok((block, tx_results))
}

fn process_block_txs<I>(
	hrp: &str,
	height: NonZeroU64,
	tbr: I,
) -> anyhow::Result<(u64, Vec<TxResult>)>
where
	I: Iterator<Item = (Bytes, ExecTxResult)>,
{
	let mut total_gas_used = 0u64;
	let txrs = tbr
		.into_iter()
		.map(|(bz, res)| make_tx_result(hrp, height, bz, res))
		.map(|txr| {
			let txr = txr?;
			total_gas_used = total_gas_used
				.checked_add(txr.gas_used())
				.context("total block gas used must not overflow")?;
			Ok(txr)
		})
		.collect::<anyhow::Result<_>>()?;

	Ok((total_gas_used, txrs))
}

fn make_tx_result(
	hrp: &str,
	block_height: NonZeroU64,
	tx_bz: Bytes,
	exec_tx_result: ExecTxResult,
) -> anyhow::Result<TxResult> {
	let tx = cosmrs::Tx::from_bytes(&tx_bz).map_err(anyhow::Error::msg)?;

	let payer = match tx.auth_info.fee.payer {
		Some(payer) => payer,
		None => match tx.auth_info.signer_infos.first().map(|s| s.public_key.as_ref()) {
			Some(Some(pk)) => signer_address(hrp, pk)?,
			Some(None) => anyhow::bail!("signer must have public key"),

			// Extract signer from tx messages
			None => proto::extract_signers_from_any_msgs(&tx.body.messages)?
				.into_iter()
				.next()
				.map(|s| s.parse().map_err(anyhow::Error::msg))
				.context("at least one msg must contain signer when no signer info provided")??,
		},
	};

	let tx_result = TxResult::builder()
		.tx_hash(sha2::Sha256::digest(&tx_bz).into())
		.block_height(block_height)
		.msgs(tx.body.messages)
		.memo(tx.body.memo)
		.maybe_timeout_height(tx.body.timeout_height.value().try_into().ok())
		.signatures(tx.signatures)
		.signers(tx.auth_info.signer_infos.into_iter().flat_map(|si| si.public_key).collect())
		.fee(tx.auth_info.fee.amount)
		.payer(payer)
		.maybe_granter(tx.auth_info.fee.granter)
		.code(exec_tx_result.code)
		.codespace(exec_tx_result.codespace)
		.gas_limit(tx.auth_info.fee.gas_limit)
		.gas_wanted(exec_tx_result.gas_wanted.try_into()?)
		.gas_used(exec_tx_result.gas_used.try_into()?)
		.data_bz(exec_tx_result.data)
		.tx_bz(tx_bz)
		.build();

	Ok(tx_result)
}

fn signer_address(hrp: &str, signer: &SignerPublicKey) -> anyhow::Result<AccountId> {
	let acc_id = match signer {
		SignerPublicKey::Single(pk) => pk.account_id(hrp).map_err(anyhow::Error::msg)?,
		SignerPublicKey::Any(any) => {
			tracing::info!("got key {any:?}");
			match any.to_msg() {
				Ok(EthSecps256K1PubKey { key }) => {
					tracing::info!("eth secp key {key:?}");
					let encoded_point = PublicKey::from_sec1_bytes(&key)?.to_encoded_point(false);
					let bytes = encoded_point.as_bytes();

					let keccak = Keccak256::digest(bytes.get(1..).context("invalid key")?);
					let eth_addr = keccak.get(12..).context("keccak must be 32 bytes long")?;

					AccountId::new(hrp, eth_addr)
						.inspect(|a| tracing::info!("payer address = {}", a.to_string()))
						.map_err(anyhow::Error::msg)?
				},
				_ => anyhow::bail!("unsupported public key"),
			}
		},
		SignerPublicKey::LegacyAminoMultisig(_) => panic!("unsupported legacyaminomultisig"),
	};

	Ok(acc_id)
}
