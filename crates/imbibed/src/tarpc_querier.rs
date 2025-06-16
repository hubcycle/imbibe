use core::future;

use futures::StreamExt;
use imbibe_persistence::pool::DbPool;
use imbibe_querier::{
	Querier,
	tarpc::{Query, QueryServer},
};
use tarpc::{
	server::{BaseChannel, Channel, incoming::Incoming},
	tokio_serde::formats::Json,
};
use tokio::net::ToSocketAddrs;

pub async fn run<A>(pool: DbPool, sock_addr: A) -> anyhow::Result<()>
where
	A: ToSocketAddrs,
{
	let listener = tarpc::serde_transport::tcp::listen(sock_addr, Json::default).await?;

	tracing::info!(
		"querier tarpc listening port {}",
		listener.local_addr().port()
	);

	let querier = Querier::builder().pool(pool).build();
	let rpc = listener
		.filter_map(|r| future::ready(r.ok()))
		.map(BaseChannel::with_defaults)
		.max_channels_per_key(1, |t| t.transport().peer_addr().unwrap().ip())
		.map(move |channel| {
			let server = QueryServer::builder().querier(querier.clone()).build();
			channel.execute(server.serve()).for_each(async |r| {
				tokio::spawn(r);
			})
		})
		.buffer_unordered(10)
		.for_each(async |_| {});

	tokio::spawn(rpc).await?;

	Ok(())
}
