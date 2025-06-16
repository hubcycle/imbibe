use core::num::NonZeroUsize;

use config::{ConfigError, Environment, File, FileFormat};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Config {
	pub app: AppConfig,

	pub db: DbConfig,

	pub indexer: IndexerConfig,

	#[cfg(not(feature = "disable-telemetry"))]
	pub telemetry: TelemetryConfig,
}

#[derive(Deserialize)]
pub struct AppConfig {
	pub name: String,
}

#[derive(Deserialize)]
pub struct DbConfig {
	pub db_url: String,
	pub max_conn: NonZeroUsize,
}

#[derive(Deserialize)]
pub struct IndexerConfig {
	pub tm_ws_url: String,
	pub batch: NonZeroUsize,
	pub workers: NonZeroUsize,
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
