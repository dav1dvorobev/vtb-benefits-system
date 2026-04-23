mod health;
mod request;

use axum::{Router, routing::get, routing::post};

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/health", get(health::health))
        .route("/api/v1/request", post(request::request))
        .route("/api/v1/request/download/{file_name}", get(request::download))
}
