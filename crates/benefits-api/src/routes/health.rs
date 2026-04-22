use crate::AppState;
use axum::{Json, extract::State, response::IntoResponse};

/// Returns server health check, including startup timestamp and uptime.
/// JSON with `startup_timestamp` (server start timestamp) and `uptime` (runtime in seconds).
pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let uptime = chrono::Utc::now()
        .signed_duration_since(state.startup_timestamp)
        .num_seconds();

    Json(serde_json::json!({
        "startup_timestamp": state.startup_timestamp.format("%d/%m/%Y %T").to_string(),
        "uptime": format!("{uptime:?}s")
    }))
}
