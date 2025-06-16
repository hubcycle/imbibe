#[cfg(feature = "server")]
pub mod server;

mod error;

pub use self::error::QueryTarpcError;

use core::num::NonZeroU64;

use imbibe_domain::{Sha256, block::Block, tx::Tx};

#[tarpc::service]
pub trait Query {
	async fn block_by_height(height: NonZeroU64) -> Result<Block, QueryTarpcError>;

	async fn block_by_block_hash(block_hash: Sha256) -> Result<Block, QueryTarpcError>;

	async fn tx_by_block_height_and_tx_idx_in_block(
		height: NonZeroU64,
		tx_idx_in_block: u64,
	) -> Result<Tx, QueryTarpcError>;

	async fn tx_by_tx_hash(tx_hash: Sha256) -> Result<Tx, QueryTarpcError>;
}
