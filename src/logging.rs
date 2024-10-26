use anyhow::Result;
use tracing::Level;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

pub fn init_logging() -> Result<WorkerGuard> {
    let file_appender = tracing_appender::rolling::daily(".", "glint.log");

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::from_default_env().add_directive(Level::INFO.into());

    let registry = tracing_subscriber::registry().with(
        fmt::layer()
            .with_span_events(fmt::format::FmtSpan::ENTER | fmt::format::FmtSpan::EXIT)
            .event_format(fmt::format().pretty())
            .with_writer(non_blocking)
            .with_filter(filter),
    );

    registry.init();

    Ok(guard)
}
