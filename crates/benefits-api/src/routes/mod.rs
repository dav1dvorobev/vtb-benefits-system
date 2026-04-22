mod health;
mod request;

use axum::{Router, routing::get, routing::post};

use crate::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/api/health", get(health::health))
        .route("/api/v1/request", post(request::request))
}
