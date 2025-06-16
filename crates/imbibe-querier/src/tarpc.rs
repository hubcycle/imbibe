mod error;

pub use self::error::{QueryTarpcError, Result};

use std::{num::NonZeroU64, time::Instant};

use bon::Builder;
use imbibe_domain::{Sha256, block::Block, tx::Tx};
use tarpc::context::Context;

use crate::Querier;

use self::error::QueryTarpcErrorKind;

#[tarpc::service]
pub trait Query {
	async fn block_by_height(height: NonZeroU64) -> Result<Block>;

	async fn block_by_block_hash(block_hash: Sha256) -> Result<Block>;

	async fn tx_by_block_height_and_tx_idx_in_block(
		height: NonZeroU64,
		tx_idx_in_block: u64,
	) -> Result<Tx>;

	async fn tx_by_tx_hash(tx_hash: Sha256) -> Result<Tx>;
}

#[derive(Clone, Builder)]
pub struct QueryServer {
	querier: Querier,
}

impl Query for QueryServer {
	async fn block_by_height(self, ctx: Context, height: NonZeroU64) -> Result<Block> {
		tokio::time::timeout(
			ctx.deadline.saturating_duration_since(Instant::now()),
			self.querier.get_block_by_height(height),
		)
		.await
		.map_err(QueryTarpcErrorKind::from)
		.and_then(|r| r.map_err(From::from))
		.inspect_err(|e| tracing::error!("{e}"))
		.map_err(From::from)
	}

	async fn block_by_block_hash(self, ctx: Context, block_hash: Sha256) -> Result<Block> {
		tokio::time::timeout(
			ctx.deadline.saturating_duration_since(Instant::now()),
			self.querier.get_block_by_block_hash(&block_hash),
		)
		.await
		.map_err(QueryTarpcErrorKind::from)
		.and_then(|r| r.map_err(From::from))
		.inspect_err(|e| tracing::error!("{e}"))
		.map_err(From::from)
	}

	async fn tx_by_block_height_and_tx_idx_in_block(
		self,
		ctx: Context,
		height: NonZeroU64,
		tx_idx_in_block: u64,
	) -> Result<Tx> {
		tokio::time::timeout(
			ctx.deadline.saturating_duration_since(Instant::now()),
			self.querier.get_tx_by_block_height_and_tx_idx_in_block(height, tx_idx_in_block),
		)
		.await
		.map_err(QueryTarpcErrorKind::from)
		.and_then(|r| r.map_err(From::from))
		.map_err(From::from)
		.inspect_err(|e| tracing::error!("{e}"))
	}

	async fn tx_by_tx_hash(self, ctx: Context, tx_hash: Sha256) -> Result<Tx> {
		tokio::time::timeout(
			ctx.deadline.saturating_duration_since(Instant::now()),
			self.querier.get_tx_by_tx_hash(&tx_hash),
		)
		.await
		.map_err(QueryTarpcErrorKind::from)
		.and_then(|r| r.map_err(From::from))
		.map_err(From::from)
		.inspect_err(|e| tracing::error!("{e}"))
	}
}
