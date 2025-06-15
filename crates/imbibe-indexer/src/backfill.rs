use core::num::{NonZeroU64, NonZeroUsize};

use cosmrs::tendermint::block::Height;
use futures::{StreamExt, TryStreamExt};
use imbibe_persistence::{pool::DbPool, store};
use tendermint_rpc::{Client, WebSocketClient};

use crate::error::{IndexerError, Result};

pub struct BackfillIndexer {
	pool: DbPool,
	client: WebSocketClient,
	batch: NonZeroUsize,
	workers: NonZeroUsize,
	lo: NonZeroU64,
	hi: NonZeroU64,
}

#[bon::bon]
impl BackfillIndexer {
	#[builder]
	pub fn new(
		pool: DbPool,
		client: WebSocketClient,
		batch: NonZeroUsize,
		workers: NonZeroUsize,
		lo: NonZeroU64,
		hi: NonZeroU64,
	) -> Result<Self> {
		if lo >= hi {
			return Err(IndexerError::Other(
				"hi must be strictly greater than lo".into(),
			));
		}

		Ok(BackfillIndexer { pool, client, batch, workers, lo, hi })
	}
}

impl BackfillIndexer {
	#[tracing::instrument(skip_all)]
	pub async fn start(self) -> Result<()> {
		tracing::info!("backfilling blocks from {} upto {}", self.lo, self.hi);

		store::fetch_missing_block_heights(&mut self.pool.get().await?, self.lo, self.hi)
			.await?
			.inspect_ok(|h| tracing::info!("backfilling block {h}"))
			.inspect_err(|e| tracing::error!("store error: {e}"))
			.map_ok(NonZeroU64::get)
			.map_ok(Height::try_from)
			.map_ok(|res| res.map_err(|_| IndexerError::RpcHeight))
			.map(|height| height?)
			.map_ok(|height| (height, self.client.clone()))
			.and_then(async |(height, client)| {
				let block_resp = client.block(height).await?;
				let block_results_resp = client.block_results(height).await?;

				let header = block_resp.block.header;
				let hash = block_resp.block_id.hash;
				let data = block_resp.block.data;
				let exec_tx_results = block_results_resp.txs_results.unwrap_or_default();

				super::process_block(header, hash, data, exec_tx_results)
			})
			.try_chunks(self.batch.get())
			.map_err(|e| IndexerError::Other(e.into()))
			.and_then(async |blocks| Ok((self.pool.get().await?, blocks)))
			.try_for_each_concurrent(self.workers.get(), async |(mut conn, tbrs)| {
				store::save_blocks_with_txs(&mut conn, &tbrs).await.map_err(From::from)
			})
			.await?;

		tracing::info!(
			"finished backfilling blocks from {} upto {}",
			self.lo,
			self.hi
		);

		Ok(())
	}
}
