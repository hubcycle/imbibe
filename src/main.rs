use std::time::Duration;

use imbibe::{config, indexer, persistence, telemetry};
use secrecy::ExposeSecret;
use tendermint_rpc::WebSocketClient;
use tokio::sync::oneshot;
use tracing_subscriber::EnvFilter;
use url::Url;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let config = config::get_configuration()?;

	let subscriber = telemetry::make_subscriber()
		.name(config.app.name)
		.exporter_endpoint(config.telemetry.trace_exporter)
		.timeout(Duration::from_millis(config.telemetry.timeout_millis))
		.env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
		.call()?;

	telemetry::init_subscriber(subscriber)?;

	let pool = persistence::establish_pool()
		.url(Url::parse(config.db.db_url.expose_secret())?)
		.max_size(config.db.max_conn)
		.call()
		.await?;

	let (client, driver) = WebSocketClient::new(config.tm.url.as_str()).await?;
	let driver_handle = tokio::spawn(driver.run());

	let (tx, rx) = oneshot::channel();

	let hrp = config.chain.hrp;

	let live_indexer_handle = tokio::spawn(indexer::live::start(
		pool.clone(),
		client.clone(),
		hrp.clone(),
		tx,
	));

	if let Ok(height) = rx.await {
		tracing::info!("first live block height received: {}", height);

		if height.get() > 1 {
			tracing::info!("backfilling missing blocks upto {}", height);

			tokio::spawn(indexer::historical::backfill(
				client,
				pool,
				height,
				config.app.batch,
				config.db.max_conn,
				hrp.into(),
			))
			.await??;

			tracing::info!("backfilling finished");
		} else {
			tracing::info!("backfilling skipped as live subscriber started with block 1");
		}
	}

	live_indexer_handle.await??;
	driver_handle.await??;

	Ok(())
}
