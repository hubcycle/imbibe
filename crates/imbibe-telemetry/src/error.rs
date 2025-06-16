use opentelemetry_otlp::ExporterBuildError;
use tracing::subscriber::SetGlobalDefaultError;

pub type Result<T, E = TelemetryError> = core::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
	#[error("set global default error: {0}")]
	SetGlobalDefault(#[from] SetGlobalDefaultError),

	#[error("exporter build error: {0}")]
	ExporterBuild(#[from] ExporterBuildError),
}
