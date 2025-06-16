use imbibe_persistence::pool;
use imbibed::config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let config = tokio::task::spawn_blocking(config::get_configuration).await??;

	#[cfg(not(feature = "disable-telemetry"))]
	imbibe_telemetry::make_tracing_subscriber(
		config.app.name,
		config.telemetry.trace_exporter,
		core::time::Duration::from_millis(config.telemetry.timeout_millis),
		tracing_subscriber::EnvFilter::try_from_default_env()
			.unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
	)
	.and_then(imbibe_telemetry::init_subscriber)?;

	let pool = pool::establish_pool(config.db.db_url, config.db.max_conn).await?;

	let indexer = imbibed::indexer::run(
		config.indexer.tm_ws_url,
		pool,
		config.indexer.batch,
		config.indexer.workers,
	);

	let indexer_handle = tokio::spawn(indexer);

	indexer_handle.await??;

	Ok(())
}
