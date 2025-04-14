use anyhow::Context;
use secrecy::SecretString;
use serde::Deserialize;
use url::Url;

const CONFIG_ENV_PREFIX: &str = "IMBIBE";

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
	pub max_conn: usize,
}

#[derive(Deserialize)]
pub struct AppConfig {
	pub name: String,
	pub workers: u16,
	pub batch: u16,
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
	let base_path = std::env::current_dir()?;

	let config_dir = base_path.join("config");

	config::Config::builder()
		.add_source(config::File::from(config_dir.join("config.ron")))
		.add_source(
			config::Environment::with_prefix(CONFIG_ENV_PREFIX)
				.prefix_separator("_")
				.separator("__"),
		)
		.build()?
		.try_deserialize()
		.context("failed to deserialize config")
}
