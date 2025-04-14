use std::{borrow::Cow, sync::OnceLock, time::Duration};

use anyhow::Context;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::{SpanExporter, WithExportConfig};
use opentelemetry_sdk::trace::SdkTracerProvider;
use tracing::{Subscriber, subscriber};
use tracing_subscriber::{EnvFilter, Registry, fmt::format::FmtSpan, layer::SubscriberExt};
use url::Url;

static INIT_SUBSCRIBER: OnceLock<anyhow::Result<()>> = OnceLock::new();

pub fn init_subscriber<S>(s: S) -> anyhow::Result<()>
where
	S: Subscriber + Send + Sync,
{
	INIT_SUBSCRIBER
		.get_or_init(|| {
			subscriber::set_global_default(s).context("failed to initialize tracing subcriber")
		})
		.as_ref()
		.map_err(|e| anyhow::anyhow!(e.to_string()))
		.cloned()
}

pub fn make_subscriber<N>(
	name: N,
	exporter_endpoint: Url,
	timeout: Duration,
	env_filter: EnvFilter,
) -> anyhow::Result<impl Subscriber + Send + Sync>
where
	N: Into<Cow<'static, str>>,
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
