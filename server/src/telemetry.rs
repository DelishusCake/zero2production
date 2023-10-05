use anyhow::Context;

use tracing::{subscriber::set_global_default, Subscriber};

use tracing_log::LogTracer;

use tracing_subscriber::fmt::{self, format::FmtSpan, MakeWriter};
use tracing_subscriber::EnvFilter;

pub fn create_subscriber<Sink>(env_filter: String, sink: Sink) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    fmt::fmt()
        .with_env_filter(EnvFilter::new(env_filter))
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .with_writer(sink)
        .finish()
}

pub fn set_subscriber(subscriber: impl Subscriber + Send + Sync) -> anyhow::Result<()> {
    LogTracer::init().context("Failed to initalize logging")?;

    set_global_default(subscriber).context("Failed to set global subscriber")
}
