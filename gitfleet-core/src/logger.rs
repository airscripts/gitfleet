use tracing_subscriber::{EnvFilter, fmt};

use crate::constants::GITFLEET_LOG_ENV;

pub fn init(debug: bool) {
    let level = if debug { "debug" } else { "warn" };
    let filter = std::env::var(GITFLEET_LOG_ENV)
        .ok()
        .and_then(|value| EnvFilter::try_new(value).ok())
        .unwrap_or_else(|| EnvFilter::new(level));

    fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
}
