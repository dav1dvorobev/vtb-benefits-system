use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

pub fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("benefits_api=info,tower_http=info,axum=info"));

    tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt::layer().compact())
        .init();
}
