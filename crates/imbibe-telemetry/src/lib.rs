mod error;

pub use self::error::TelemetryError;

use core::time::Duration;

use std::borrow::Cow;

use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing::{Subscriber, subscriber};
use tracing_subscriber::{EnvFilter, Registry, fmt::format::FmtSpan, layer::SubscriberExt};

use self::error::Result;

pub fn init_subscriber<S>(s: S) -> Result<()>
where
	S: Subscriber + Send + Sync,
{
	subscriber::set_global_default(s).map_err(From::from)
}

pub fn make_tracing_subscriber<N, U>(
	name: N,
	exporter_endpoint: U,
	timeout: Duration,
	env_filter: EnvFilter,
) -> Result<impl Subscriber + Send + Sync>
where
	Cow<'static, str>: From<N>,
	String: From<U>,
{
	let exporter = SpanExporter::builder()
		.with_tonic()
		.with_endpoint(exporter_endpoint)
		.with_timeout(timeout)
		.build()?;

	let tracer = SdkTracerProvider::builder().with_batch_exporter(exporter).build().tracer(name);

	let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

	let subscriber = Registry::default()
		.with(telemetry)
		.with(
			tracing_subscriber::fmt::layer()
				.with_target(false)
				.with_line_number(true)
				.with_span_events(FmtSpan::NEW | FmtSpan::CLOSE),
		)
		.with(env_filter);

	Ok(subscriber)
}
