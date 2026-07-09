use tracing_subscriber::{fmt, EnvFilter};

pub fn init(debug: bool) {
    let level = if debug { "debug" } else { "warn" };
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level));

    fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
}
