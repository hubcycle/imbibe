use core::num::NonZeroUsize;

use std::path::PathBuf;

use anyhow::Context;
use secrecy::SecretString;
use serde::Deserialize;
use url::Url;

#[derive(Deserialize)]
pub struct Config {
	pub db: DbConfig,
	pub app: AppConfig,
	pub tm: TmRpcConfig,
	pub telemetry: TelemetryConfig,
}

#[derive(Deserialize)]
pub struct DbConfig {
	pub db_url: SecretString,
	pub max_conn: NonZeroUsize,
}

#[derive(Deserialize)]
pub struct AppConfig {
	pub name: String,
	pub batch: NonZeroUsize,
}

#[derive(Deserialize)]
pub struct TmRpcConfig {
	pub url: Url,
}

#[derive(Deserialize)]
pub struct TelemetryConfig {
	pub trace_exporter: Url,
	pub timeout_millis: u64,
}

pub fn get_configuration() -> anyhow::Result<Config> {
	config::Config::builder()
		.add_source(config::File::from(Config::base_config_ron()?))
		.add_source(
			config::Environment::with_prefix(Config::CONFIG_ENV_PREFIX)
				.prefix_separator("_")
				.separator("__"),
		)
		.build()?
		.try_deserialize()
		.context("failed to deserialize config")
}

impl Config {
	const CONFIG_ENV_PREFIX: &str = "IMBIBE";

	const DEFAULT_RON: &str = "config/config.ron";

	fn base_config_ron() -> anyhow::Result<PathBuf> {
		Ok(std::env::current_dir()?.join(Self::DEFAULT_RON))
	}
}
