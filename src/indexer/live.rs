use anyhow::Context;
use futures::StreamExt;
use tendermint::block::Height;
use tendermint_rpc::{SubscriptionClient, WebSocketClient, event::EventData, query::EventType};
use tokio::sync::oneshot;

use crate::{
	domain::block::Block,
	persistence::{DbPool, store},
};

#[tracing::instrument(level = "info", skip_all)]
pub async fn start<S>(pool: DbPool, client: WebSocketClient, tx: S) -> anyhow::Result<()>
where
	S: Into<Option<oneshot::Sender<Height>>>,
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

	tracing::info!("received first live block {}", first_block.header.height);

	if let Some(tx) = tx.into() {
		if let Err(height) = tx.send(first_block.header.height) {
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
