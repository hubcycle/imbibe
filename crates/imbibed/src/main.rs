use imbibe::config;

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

	#[cfg(feature = "persistence")]
	let pool = imbibe_persistence::pool::establish_pool(config.db.db_url, config.db.max_conn).await?;

	#[cfg(feature = "indexer")]
	let indexer_handle = {
		let indexer = imbibe::indexer::run(
			config.indexer.tm_ws_url,
			pool.clone(),
			config.indexer.batch,
			config.indexer.workers,
			config.indexer.windows.into_iter(),
		);

		tokio::spawn(indexer)
	};

	#[cfg(feature = "tarpc-querier")]
	let tarpc_querier_handle = {
		let tarpc_querier = imbibe::tarpc_querier::run(pool.clone(), config.querier.tarpc);

		tokio::spawn(tarpc_querier)
	};

	#[cfg(feature = "graphql-querier")]
	let graphql_querier_handle = {
		let graphql_querier = imbibe::graphql_querier::run(pool, config.querier.graphql);

		tokio::spawn(graphql_querier)
	};

	#[cfg(feature = "indexer")]
	indexer_handle.await??;

	#[cfg(feature = "tarpc-querier")]
	tarpc_querier_handle.await??;

	#[cfg(feature = "graphql-querier")]
	graphql_querier_handle.await??;

	Ok(())
}
