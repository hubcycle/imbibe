mod error;

pub use self::error::QuerierError;

use core::num::NonZeroU64;

use bon::Builder;
use imbibe_domain::{Sha256, block::Block, tx::Tx};
use imbibe_persistence::{pool::DbPool, store};

use self::error::Result;

#[derive(Clone, Builder)]
pub struct Querier {
	pool: DbPool,
}

impl Querier {
	pub async fn get_block_by_height(&self, height: NonZeroU64) -> Result<Block> {
		store::fetch_block_by_height(&mut self.pool.get().await?, height)
			.await
			.map_err(From::from)
			.inspect_err(|e| tracing::error!("{e}"))
	}

	pub async fn get_block_by_block_hash(&self, block_hash: &Sha256) -> Result<Block> {
		store::fetch_block_by_block_hash(&mut self.pool.get().await?, block_hash)
			.await
			.map_err(From::from)
			.inspect_err(|e| tracing::error!("{e}"))
	}

	pub async fn get_tx_by_block_height_and_tx_idx_in_block(
		&self,
		height: NonZeroU64,
		tx_idx_in_block: u64,
	) -> Result<Tx> {
		store::fetch_tx_by_block_height_and_tx_idx_in_block(
			&mut self.pool.get().await?,
			height,
			tx_idx_in_block,
		)
		.await
		.map_err(From::from)
		.inspect_err(|e| tracing::error!("{e}"))
	}

	pub async fn get_tx_by_tx_hash(&self, tx_hash: &Sha256) -> Result<Tx> {
		store::fetch_tx_by_tx_hash(&mut self.pool.get().await?, tx_hash)
			.await
			.map_err(From::from)
			.inspect_err(|e| tracing::error!("{e}"))
	}
}
