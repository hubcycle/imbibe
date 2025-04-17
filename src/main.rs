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

	let subscriber = telemetry::make_subscriber(
		config.app.name,
		config.telemetry.trace_exporter,
		Duration::from_millis(config.telemetry.timeout_millis),
		EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
	)?;

	telemetry::init_subscriber(subscriber)?;

	let pool = persistence::establish_pool(
		Url::parse(config.db.db_url.expose_secret())?,
		config.db.max_conn,
	)
	.await?;

	let (client, driver) = WebSocketClient::new(config.tm.url.as_str()).await?;
	let driver_handle = tokio::spawn(driver.run());

	let (tx, rx) = oneshot::channel();

	let live_indexer_handle = tokio::spawn(indexer::live::start(pool.clone(), client.clone(), tx));

	if let Ok(height) = rx.await {
		tracing::info!("first live block height received: {}", height);

		if height.get() > 1 {
			tokio::spawn(indexer::historical::backfill(
				client,
				pool,
				height,
				config.app.batch,
				config.db.max_conn,
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
