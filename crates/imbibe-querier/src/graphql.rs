mod object;

use core::{num::NonZeroU64, ops::Bound};

use async_graphql::{Context, Object, Result};
use bon::Builder;
use futures::TryStreamExt;
use imbibe_domain::Sha256;

use crate::server::Querier;

use self::object::{BlockGql, FeeSummaryGql, TxGql, TxSummaryGql};

const SHA256_LEN: usize = 32;

#[derive(Builder)]
pub struct QueryRoot {
	querier: Querier,
}

#[Object(rename_fields = "snake_case")]
impl QueryRoot {
	async fn block_by_height(
		&self,
		_ctx: &Context<'_>,
		height: NonZeroU64,
	) -> Result<Option<BlockGql>> {
		self.querier
			.get_block_by_height(height)
			.await
			.inspect_err(|e| tracing::error!("{e}"))
			.map(|block| block.map(From::from))
			.map_err(From::from)
	}

	async fn latest_block(&self, _ctx: &Context<'_>) -> Result<Option<BlockGql>> {
		self.querier
			.get_latest_block()
			.await
			.inspect_err(|e| tracing::error!("{e}"))
			.map(|block| block.map(From::from))
			.map_err(From::from)
	}

	async fn blocks_in_height_range(
		&self,
		_ctx: &Context<'_>,
		lo: Option<NonZeroU64>,
		hi: Option<NonZeroU64>,
	) -> Result<Vec<BlockGql>> {
		let range = (
			lo.map(Bound::Included).unwrap_or(Bound::Unbounded),
			hi.map(Bound::Included).unwrap_or(Bound::Unbounded),
		);

		self.querier
			.get_blocks_in_height_range(range)
			.await?
			.map_ok(From::from)
			.inspect_err(|e| tracing::error!("{e}"))
			.map_err(From::from)
			.try_collect()
			.await
	}

	async fn txs_by_block_height(
		&self,
		_ctx: &Context<'_>,
		height: NonZeroU64,
	) -> Result<Vec<TxGql>> {
		self.querier
			.get_txs_by_height(height)
			.await
			.inspect_err(|e| tracing::error!("{e}"))?
			.into_iter()
			.map(TryFrom::try_from)
			.collect::<Result<_>>()
	}

	async fn block_by_hash(
		&self,
		_ctx: &Context<'_>,
		hash: [u8; SHA256_LEN],
	) -> Result<Option<BlockGql>> {
		self.querier
			.get_block_by_block_hash(&Sha256::new(hash))
			.await
			.inspect_err(|e| tracing::error!("{e}"))?
			.map(From::from)
			.map(Ok)
			.transpose()
	}

	async fn tx_by_hash(
		&self,
		_ctx: &Context<'_>,
		hash: [u8; SHA256_LEN],
	) -> Result<Option<TxGql>> {
		self.querier
			.get_tx_by_tx_hash(&Sha256::new(hash))
			.await
			.inspect_err(|e| tracing::error!("{e}"))?
			.map(TryFrom::try_from)
			.transpose()
	}

	async fn tx_summaries(&self, _ctx: &Context<'_>) -> Result<Vec<TxSummaryGql>> {
		self.querier
			.get_tx_summaries()
			.await?
			.map_ok(From::from)
			.inspect_err(|e| tracing::error!("{e}"))
			.map_err(From::from)
			.try_collect()
			.await
	}

	async fn fee_summaries(&self, _ctx: &Context<'_>) -> Result<Vec<FeeSummaryGql>> {
		self.querier
			.get_fee_summaries()
			.await?
			.map_ok(From::from)
			.inspect_err(|e| tracing::error!("{e}"))
			.map_err(From::from)
			.try_collect()
			.await
	}
}
