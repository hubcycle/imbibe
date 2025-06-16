use core::num::NonZeroU64;

use bon::Builder;
use imbibe_domain::{Sha256, block::Block, tx::Tx};

use crate::server::Querier;

use super::{Query, QueryTarpcError};

#[derive(Clone, Builder)]
pub struct QueryServer {
	querier: Querier,
}

impl Query for QueryServer {
	async fn block_by_height(
		self,
		ctx: tarpc::context::Context,
		height: NonZeroU64,
	) -> Result<Block, QueryTarpcError> {
		tokio::time::timeout(
			ctx.deadline.saturating_duration_since(std::time::Instant::now()),
			self.querier.get_block_by_height(height),
		)
		.await
		.map_err(super::error::QueryTarpcErrorKind::from)
		.and_then(|r| r.map_err(From::from))
		.inspect_err(|e| tracing::error!("{e}"))
		.map_err(From::from)
	}

	async fn block_by_block_hash(
		self,
		ctx: tarpc::context::Context,
		block_hash: Sha256,
	) -> Result<Block, QueryTarpcError> {
		tokio::time::timeout(
			ctx.deadline.saturating_duration_since(std::time::Instant::now()),
			self.querier.get_block_by_block_hash(&block_hash),
		)
		.await
		.map_err(super::error::QueryTarpcErrorKind::from)
		.and_then(|r| r.map_err(From::from))
		.inspect_err(|e| tracing::error!("{e}"))
		.map_err(From::from)
	}

	async fn tx_by_block_height_and_tx_idx_in_block(
		self,
		ctx: tarpc::context::Context,
		height: NonZeroU64,
		tx_idx_in_block: u64,
	) -> Result<Tx, QueryTarpcError> {
		tokio::time::timeout(
			ctx.deadline.saturating_duration_since(std::time::Instant::now()),
			self.querier.get_tx_by_block_height_and_tx_idx_in_block(height, tx_idx_in_block),
		)
		.await
		.map_err(super::error::QueryTarpcErrorKind::from)
		.and_then(|r| r.map_err(From::from))
		.map_err(From::from)
		.inspect_err(|e| tracing::error!("{e}"))
	}

	async fn tx_by_tx_hash(
		self,
		ctx: tarpc::context::Context,
		tx_hash: Sha256,
	) -> Result<Tx, QueryTarpcError> {
		tokio::time::timeout(
			ctx.deadline.saturating_duration_since(std::time::Instant::now()),
			self.querier.get_tx_by_tx_hash(&tx_hash),
		)
		.await
		.map_err(super::error::QueryTarpcErrorKind::from)
		.and_then(|r| r.map_err(From::from))
		.map_err(From::from)
		.inspect_err(|e| tracing::error!("{e}"))
	}
}
