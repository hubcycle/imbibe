use std::{
	num::{NonZeroU64, NonZeroUsize},
	sync::Arc,
};

use anyhow::Context;
use futures::{StreamExt, TryStreamExt};
use tendermint::block::Height;
use tendermint_rpc::{Client, WebSocketClient};

use crate::persistence::{DbPool, store};

#[tracing::instrument(skip(pool))]
pub async fn backfill(
	client: WebSocketClient,
	pool: DbPool,
	upto: NonZeroU64,
	batch: NonZeroUsize,
	workers: NonZeroUsize,
	hrp: Arc<str>,
) -> anyhow::Result<()> {
	store::fetch_missing_block_heights(
		&mut pool.get().await?,
		NonZeroU64::MIN,
		upto.get()
			.checked_sub(1)
			.context("at least one missing block must exist to backfill")?
			.try_into()?,
	)
	.await?
	.map_ok(NonZeroU64::get)
	.map_ok(Height::try_from)
	.map_ok(|res| res.map_err(anyhow::Error::from))
	.map(|height| height?)
	.map_ok(|height| (height, client.clone()))
	.and_then(|(height, client)| {
		let hrp = hrp.clone();
		async move {
			let block_resp = client.block(height).await?;
			let block_results_resp = client.block_results(height).await?;

			let header = block_resp.block.header;
			let hash = block_resp.block_id.hash;
			let data = block_resp.block.data;
			let exec_tx_results = block_results_resp.txs_results.unwrap_or_default();

			super::process_block(&hrp, header, hash, data, exec_tx_results)
		}
	})
	.try_chunks(batch.get())
	.map_err(anyhow::Error::from)
	.and_then(async |blocks| Ok((pool.get().await?, blocks)))
	.try_for_each_concurrent(workers.get(), async |(mut conn, tbrs)| {
		store::save_blocks_with_tx_resulsts(&mut conn, &tbrs).await
	})
	.await?;

	Ok(())
}
