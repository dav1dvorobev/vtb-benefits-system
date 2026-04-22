use benefits_api::config::init_logging;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    init_logging();

    let port = std::env::var("BENEFITS_APPLICATION_PORT").unwrap_or("80".to_string());
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .inspect_err(|e| tracing::error!("failed to bind port: {e}"))
        .unwrap();

    tracing::info!(
        version = env!("CARGO_PKG_VERSION"),
        "{} started on http://0.0.0.0:{port}",
        env!("CARGO_PKG_NAME")
    );

    let state = benefits_api::AppState::new();
    benefits_api::run(listener, state)
        .await
        .inspect_err(|e| tracing::error!("failed to run {}: {e}", env!("CARGO_PKG_NAME")))
        .unwrap();
}
