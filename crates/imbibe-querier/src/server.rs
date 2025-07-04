use core::{num::NonZeroU64, ops::RangeBounds};

use bon::Builder;
use futures::{Stream, TryStreamExt};
use imbibe_domain::{
	block::Block,
	summary::{FeeSummary, TxSummary},
	tx::Tx,
};

use crate::QuerierError;

#[derive(Clone, Builder)]
pub struct Querier {
	pool: imbibe_persistence::pool::DbPool,
}

impl Querier {
	pub async fn get_latest_block(&self) -> Result<Option<Block>, QuerierError> {
		imbibe_persistence::store::fetch_latest_block(&mut self.pool.get().await?)
			.await
			.inspect_err(|e| tracing::error!("{e}"))
			.map_err(From::from)
	}

	pub async fn get_block_by_height(
		&self,
		height: NonZeroU64,
	) -> Result<Option<Block>, QuerierError> {
		imbibe_persistence::store::fetch_block_by_height(&mut self.pool.get().await?, height)
			.await
			.inspect_err(|e| tracing::error!("{e}"))
			.map_err(From::from)
	}

	pub async fn get_block_by_block_hash(
		&self,
		block_hash: &imbibe_domain::Sha256,
	) -> Result<Option<Block>, QuerierError> {
		imbibe_persistence::store::fetch_block_by_block_hash(
			&mut self.pool.get().await?,
			block_hash,
		)
		.await
		.inspect_err(|e| tracing::error!("{e}"))
		.map_err(From::from)
	}

	pub async fn get_tx_by_block_height_and_tx_idx_in_block(
		&self,
		height: core::num::NonZeroU64,
		tx_idx_in_block: u64,
	) -> Result<Option<Tx>, QuerierError> {
		imbibe_persistence::store::fetch_tx_by_block_height_and_tx_idx_in_block(
			&mut self.pool.get().await?,
			height,
			tx_idx_in_block,
		)
		.await
		.inspect_err(|e| tracing::error!("{e}"))
		.map_err(From::from)
	}

	pub async fn get_tx_by_tx_hash(
		&self,
		tx_hash: &imbibe_domain::Sha256,
	) -> Result<Option<Tx>, QuerierError> {
		imbibe_persistence::store::fetch_tx_by_tx_hash(&mut self.pool.get().await?, tx_hash)
			.await
			.inspect_err(|e| tracing::error!("{e}"))
			.map_err(From::from)
	}

	pub async fn get_blocks_in_height_range<HR>(
		&self,
		height_range: HR,
	) -> Result<impl Stream<Item = Result<Block, QuerierError>> + use<HR>, QuerierError>
	where
		HR: RangeBounds<NonZeroU64>,
	{
		let mut conn = self.pool.get().await?;
		let stream =
			imbibe_persistence::store::fetch_blocks_in_height_range(&mut conn, height_range)
				.await
				.inspect_err(|e| tracing::error!("{e}"))?
				.map_err(From::from);

		Ok(stream)
	}

	pub async fn get_txs_by_height(&self, height: NonZeroU64) -> Result<Vec<Tx>, QuerierError> {
		let conn = &mut self.pool.get().await?;

		imbibe_persistence::store::fetch_txs_by_height(conn, height)
			.await
			.inspect_err(|e| tracing::error!("{e}"))
			.map_err(From::from)
	}

	pub async fn get_tx_summaries(
		&self,
	) -> Result<impl Stream<Item = Result<TxSummary, QuerierError>> + use<>, QuerierError> {
		let stream = imbibe_persistence::store::fetch_tx_summaries(&mut self.pool.get().await?)
			.await
			.inspect_err(|e| tracing::error!("{e}"))?
			.map_err(From::from);

		Ok(stream)
	}

	pub async fn get_fee_summaries(
		&self,
	) -> Result<impl Stream<Item = Result<FeeSummary, QuerierError>> + use<>, QuerierError> {
		let stream = imbibe_persistence::store::fetch_fee_summaries(&mut self.pool.get().await?)
			.await
			.inspect_err(|e| tracing::error!("{e}"))?
			.map_err(From::from);

		Ok(stream)
	}

	pub async fn get_tx_summary_since_blocks_ago(
		&self,
		since_blocks_ago: NonZeroU64,
	) -> Result<Option<TxSummary>, QuerierError> {
		let conn = &mut self.pool.get().await?;

		imbibe_persistence::store::fetch_tx_summary_since_blocks_ago(conn, since_blocks_ago)
			.await
			.inspect_err(|e| tracing::error!("{e}"))
			.map_err(From::from)
	}

	pub async fn get_fee_summary_since_blocks_ago(
		&self,
		since_blocks_ago: NonZeroU64,
	) -> Result<Option<FeeSummary>, QuerierError> {
		let conn = &mut self.pool.get().await?;

		imbibe_persistence::store::fetch_fee_summary_since_blocks_ago(conn, since_blocks_ago)
			.await
			.inspect_err(|e| tracing::error!("{e}"))
			.map_err(From::from)
	}
}
