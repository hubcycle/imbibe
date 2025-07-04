use core::num::NonZeroU64;

use std::collections::BTreeSet;

use futures::StreamExt;
use imbibe_persistence::{pool::DbPool, store};
use tendermint_rpc::{SubscriptionClient, WebSocketClient, event::EventData, query::EventType};
use tokio::sync::oneshot;

use crate::error::{IndexerError, Result};

pub struct LiveIndexer {
	pool: DbPool,
	client: WebSocketClient,
	windows: Option<Vec<NonZeroU64>>,
	first_block_transmitter: Option<oneshot::Sender<NonZeroU64>>,
}

#[bon::bon]
impl LiveIndexer {
	#[builder]
	pub fn new<W>(
		pool: DbPool,
		client: WebSocketClient,
		windows: W,
		first_block_transmitter: Option<oneshot::Sender<NonZeroU64>>,
	) -> Self
	where
		W: Iterator<Item = NonZeroU64>,
	{
		let windows: Vec<_> = windows.collect::<BTreeSet<_>>().into_iter().collect();
		let windows = (!windows.is_empty()).then_some(windows);
		Self { pool, client, windows, first_block_transmitter }
	}
}

impl LiveIndexer {
	#[tracing::instrument(skip_all)]
	pub async fn start(mut self) -> Result<()> {
		let mut subscription = self.client.subscribe(EventType::NewBlock.into()).await?;

		let (first_block, first_block_id, result) = loop {
			if let Some(Ok(event)) = subscription.next().await {
				if let EventData::NewBlock {
					block: Some(block),
					block_id,
					result_finalize_block: Some(result),
				} = event.data
				{
					break (block, block_id, result);
				}
			}
		};

		let first_block_height = first_block.header.height;

		tracing::info!("received first live block {first_block_height}");

		if let Some(transmitter) = self.first_block_transmitter.take() {
			let height = first_block_height.value().try_into().unwrap();

			if let Err(height) = transmitter.send(height) {
				tracing::error!("no receiver to receive first live block height {height}");
			}
		}

		let (first_block, tx_results) = super::process_block(
			first_block.header,
			first_block_id.hash,
			first_block.data,
			result.tx_results,
		)?;

		let conn = &mut self.pool.get().await?;

		store::save_block_with_txs(conn, &first_block, &tx_results).await?;

		while let Some(Ok(event)) = subscription.next().await {
			if let EventData::NewBlock {
				block: Some(block),
				block_id,
				result_finalize_block: Some(result),
			} = event.data
			{
				tracing::info!("received live block {}", block.header.height);

				let (block, tx_results) = super::process_block(
					block.header,
					block_id.hash,
					block.data,
					result.tx_results,
				)?;

				store::save_block_with_txs(conn, &block, &tx_results).await?;

				if let Some(windows) = &self.windows {
					let start_block = if let Some(&largest_window) = windows.last()
						&& first_block
							.header()
							.height()
							.checked_add(largest_window.get())
							.ok_or("window too large".into())
							.map_err(IndexerError::Other)?
							< block.header().height()
					{
						let start_block = block.header().height().get() - largest_window.get();
						NonZeroU64::new(start_block).unwrap()
					} else {
						first_block.header().height()
					};

					store::update_summaries(conn, windows, block.header().height(), start_block)
						.await?;
				}
			}
		}

		tracing::info!("indexing finished");

		Ok(())
	}
}
