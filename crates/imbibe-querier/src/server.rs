use core::num::NonZeroU64;

use bon::Builder;
use imbibe_domain::{block::Block, tx::Tx};

use crate::QuerierError;

#[derive(Clone, Builder)]
pub struct Querier {
	pool: imbibe_persistence::pool::DbPool,
}

impl Querier {
	pub async fn get_block_by_height(&self, height: NonZeroU64) -> Result<Block, QuerierError> {
		imbibe_persistence::store::fetch_block_by_height(&mut self.pool.get().await?, height)
			.await
			.map_err(From::from)
			.inspect_err(|e| tracing::error!("{e}"))
	}

	pub async fn get_block_by_block_hash(
		&self,
		block_hash: &imbibe_domain::Sha256,
	) -> Result<Block, QuerierError> {
		imbibe_persistence::store::fetch_block_by_block_hash(
			&mut self.pool.get().await?,
			block_hash,
		)
		.await
		.map_err(From::from)
		.inspect_err(|e| tracing::error!("{e}"))
	}

	pub async fn get_tx_by_block_height_and_tx_idx_in_block(
		&self,
		height: core::num::NonZeroU64,
		tx_idx_in_block: u64,
	) -> Result<Tx, QuerierError> {
		imbibe_persistence::store::fetch_tx_by_block_height_and_tx_idx_in_block(
			&mut self.pool.get().await?,
			height,
			tx_idx_in_block,
		)
		.await
		.map_err(From::from)
		.inspect_err(|e| tracing::error!("{e}"))
	}

	pub async fn get_tx_by_tx_hash(
		&self,
		tx_hash: &imbibe_domain::Sha256,
	) -> Result<Tx, QuerierError> {
		imbibe_persistence::store::fetch_tx_by_tx_hash(&mut self.pool.get().await?, tx_hash)
			.await
			.map_err(From::from)
			.inspect_err(|e| tracing::error!("{e}"))
	}
}
