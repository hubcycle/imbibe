use core::num::{NonZeroU64, NonZeroUsize};

use imbibe_indexer::{BackfillIndexer, LiveIndexer, WebSocketClient};
use imbibe_persistence::pool::DbPool;
use tokio::sync::oneshot;

pub async fn run<U>(
	url: U,
	pool: DbPool,
	batch: NonZeroUsize,
	workers: NonZeroUsize,
) -> anyhow::Result<()>
where
	U: AsRef<str>,
{
	let (client, driver) = WebSocketClient::new(url.as_ref()).await?;
	let driver_handle = tokio::spawn(driver.run());

	let (tx, rx) = oneshot::channel();

	let live_indexer = LiveIndexer::builder()
		.pool(pool.clone())
		.client(client.clone())
		.first_block_transmitter(tx)
		.build();

	let live_indexer_handle = tokio::spawn(live_indexer.start());

	if let Ok(Some(hi)) = rx
		.await
		.inspect(|h| tracing::info!("first live block height received: {h}"))
		.map(|h| h.get().checked_sub(1).expect("must not underflow due to positive height"))
		.map(NonZeroU64::new)
	{
		BackfillIndexer::builder()
			.pool(pool)
			.client(client)
			.batch(batch)
			.workers(workers)
			.lo(NonZeroU64::MIN)
			.hi(hi)
			.build()
			.map(|indexer| tokio::spawn(indexer.start()))?
			.await??;
	}

	live_indexer_handle.await??;
	driver_handle.await??;

	Ok(())
}
