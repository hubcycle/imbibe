use config::{ConfigError, Environment, File, FileFormat};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
	pub app: AppConfig,

	#[cfg(feature = "persistence")]
	pub db: DbConfig,

	#[cfg(feature = "indexer")]
	pub indexer: IndexerConfig,

	#[cfg(feature = "querier")]
	pub querier: QuerierConfig,

	#[cfg(not(feature = "disable-telemetry"))]
	pub telemetry: TelemetryConfig,
}

#[derive(Deserialize)]
pub struct AppConfig {
	pub name: String,
}

#[cfg(feature = "persistence")]
#[derive(Deserialize)]
pub struct DbConfig {
	pub db_url: String,
	pub max_conn: core::num::NonZeroUsize,
}

#[cfg(feature = "indexer")]
#[derive(Deserialize)]
pub struct IndexerConfig {
	pub tm_ws_url: String,
	pub batch: core::num::NonZeroUsize,
	pub workers: core::num::NonZeroUsize,
}

#[cfg(feature = "querier")]
#[derive(Deserialize)]
pub struct QuerierConfig {
	pub listen: String,
}

#[cfg(not(feature = "disable-telemetry"))]
#[derive(Deserialize)]
pub struct TelemetryConfig {
	pub trace_exporter: String,
	pub timeout_millis: u64,
}

pub fn get_configuration() -> Result<Config, ConfigError> {
	config::Config::builder()
		.add_source(File::from_str(
			include_str!("base_config.ron"),
			FileFormat::Ron,
		))
		.add_source(
			Environment::with_prefix(Config::CONFIG_ENV_PREFIX)
				.prefix_separator("_")
				.separator("__"),
		)
		.build()?
		.try_deserialize()
}

impl Config {
	const CONFIG_ENV_PREFIX: &str = "IMBIBED";
}
