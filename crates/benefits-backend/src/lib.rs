pub mod config;
pub mod routes;

use axum::Router;
use chrono::{DateTime, Utc};
use tokio::net::TcpListener;
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone, Debug)]
pub struct AppState {
    pub startup_timestamp: DateTime<Utc>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            startup_timestamp: Utc::now(),
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}

pub async fn run(listener: TcpListener, state: AppState) -> std::io::Result<()> {
    let app = Router::new()
        .merge(routes::routes())
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    axum::serve(listener, app).await
}
