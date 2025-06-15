use std::sync::OnceLock;

use imbibe_persistence::pool;
use imbibed::config;
use tracing::{Subscriber, subscriber};
use tracing_subscriber::{EnvFilter, Registry, fmt::format::FmtSpan, layer::SubscriberExt};

static INIT_SUBSCRIBER: OnceLock<()> = OnceLock::new();

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let config = tokio::task::spawn_blocking(config::get_configuration).await??;

	let subscriber = Registry::default()
		.with(
			tracing_subscriber::fmt::layer()
				.with_target(false)
				.with_line_number(true)
				.with_span_events(FmtSpan::NEW | FmtSpan::CLOSE),
		)
		.with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")));

	init_subscriber(subscriber);

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

fn init_subscriber<S>(s: S)
where
	S: Subscriber + Send + Sync,
{
	INIT_SUBSCRIBER.get_or_init(|| {
		subscriber::set_global_default(s).expect("failed to initialize tracing subcriber")
	});
}
