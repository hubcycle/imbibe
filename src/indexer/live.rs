use core::num::NonZeroU64;

use anyhow::Context;
use futures::StreamExt;
use tendermint_rpc::{SubscriptionClient, WebSocketClient, event::EventData, query::EventType};
use tokio::sync::oneshot;

use crate::persistence::{DbPool, store};

#[tracing::instrument(skip_all)]
pub async fn start<S>(
	pool: DbPool,
	client: WebSocketClient,
	hrp: String,
	tx: S,
) -> anyhow::Result<()>
where
	S: Into<Option<oneshot::Sender<NonZeroU64>>>,
{
	let mut subscription = client.subscribe(EventType::NewBlock.into()).await?;

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
	if let Some(tx) = tx.into() {
		let height = first_block_height.value().try_into().context("height must be positive")?;
		if let Err(height) = tx.send(height) {
			tracing::error!("no receiver to accept first live block height {}", height);
		}
	}

	let (first_block, tx_results) = super::process_block(
		&hrp,
		first_block.header,
		first_block_id.hash,
		first_block.data,
		result.tx_results,
	)?;

	store::save_block_with_tx_results(&mut pool.get().await?, &first_block, &tx_results).await?;

	while let Some(Ok(event)) = subscription.next().await {
		if let EventData::NewBlock {
			block: Some(block),
			block_id,
			result_finalize_block: Some(result),
		} = event.data
		{
			tracing::info!("received live block {}", block.header.height);

			let (block, tx_results) = super::process_block(
				&hrp,
				block.header,
				block_id.hash,
				block.data,
				result.tx_results,
			)?;

			store::save_block_with_tx_results(&mut pool.get().await?, &block, &tx_results).await?;
		}
	}

	tracing::info!("indexing finished");

	Ok(())
}
