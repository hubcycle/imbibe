use core::num::NonZeroU64;

use bon::Builder;
use futures::StreamExt;
use imbibe_persistence::{pool::DbPool, store};
use tendermint_rpc::{SubscriptionClient, WebSocketClient, event::EventData, query::EventType};
use tokio::sync::oneshot;

use crate::error::Result;

#[derive(Builder)]
pub struct LiveIndexer {
	pool: DbPool,
	client: WebSocketClient,
	first_block_transmitter: Option<oneshot::Sender<NonZeroU64>>,
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

		tracing::info!("received first live block {}", first_block_height);

		if let Some(transmitter) = self.first_block_transmitter.take() {
			let height = first_block_height.value().try_into().unwrap();

			if let Err(height) = transmitter.send(height) {
				tracing::error!("no receiver to receive first live block height {}", height);
			}
		}

		let (first_block, tx_results) = super::process_block(
			first_block.header,
			first_block_id.hash,
			first_block.data,
			result.tx_results,
		)?;

		store::save_block_with_txs(&mut self.pool.get().await?, &first_block, &tx_results).await?;

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

				store::save_block_with_txs(&mut self.pool.get().await?, &block, &tx_results)
					.await?;
			}
		}

		tracing::info!("indexing finished");

		Ok(())
	}
}
