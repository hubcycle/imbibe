use core::num::NonZeroU64;

use anyhow::Context;
use futures::StreamExt;
use tendermint_rpc::{SubscriptionClient, WebSocketClient, event::EventData, query::EventType};
use tokio::sync::oneshot;

use crate::{
	domain::block::Block,
	persistence::{DbPool, store},
};

#[tracing::instrument(skip_all)]
pub async fn start<S>(pool: DbPool, client: WebSocketClient, tx: S) -> anyhow::Result<()>
where
	S: Into<Option<oneshot::Sender<NonZeroU64>>>,
{
	let mut subscription = client.subscribe(EventType::NewBlock.into()).await?;

	let (first_block, first_block_id) = loop {
		if let Some(Ok(event)) = subscription.next().await {
			if let EventData::NewBlock {
				block: Some(block),
				block_id,
				..
			} = event.data
			{
				break (block, block_id);
			}
		}
	};

	let first_live_block_height = first_block.header.height;
	tracing::info!("received first live block {}", first_live_block_height);
	if let Some(tx) = tx.into() {
		let height =
			first_live_block_height.value().try_into().context("height must be positive")?;
		if let Err(height) = tx.send(height) {
			tracing::error!("no receiver to accept first live block height {}", height);
		}
	}

	{
		let first_live_block = Block::builder()
			.header(super::make_header(first_block.header)?)
			.hash(super::make_sha256(first_block_id.hash).context("missing block hash")?)
			.data(first_block.data)
			.build();

		store::save_block(&mut pool.get().await?, first_live_block).await?;
	}

	while let Some(Ok(event)) = subscription.next().await {
		if let EventData::NewBlock {
			block: Some(block),
			block_id,
			..
		} = event.data
		{
			tracing::info!("received live block {}", block.header.height);

			let block = Block::builder()
				.header(super::make_header(block.header)?)
				.hash(super::make_sha256(block_id.hash).context("missing block hash")?)
				.data(block.data)
				.build();

			store::save_block(&mut pool.get().await?, block).await?;
		}
	}

	tracing::info!("indexing finished");

	Ok(())
}
