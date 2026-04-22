use std::{fs, path::Path};

use crate::AppState;
use axum::{
    Json,
    extract::State,
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use statement_pdf::{StatementAssets, StatementInput};

pub async fn request(
    State(_state): State<AppState>,
    Json(statement): Json<StatementInput>,
) -> Response {
    tracing::info!(target: "benefits_api::routes::request", "request received");

    let stamp_png = match fs::read(Path::new("stamp.png")) {
        Ok(stamp_png) => stamp_png,
        Err(error) => {
            tracing::error!(%error, "failed to read stamp.png");
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "stamp.png is missing or unreadable"
                })),
            )
                .into_response();
        }
    };

    match statement_pdf::generate_pdf_with_assets(
        statement,
        StatementAssets {
            stamp_png: Some(stamp_png),
            signature_images: Default::default(),
        },
    ) {
        Ok(pdf) => (
            StatusCode::OK,
            [(header::CONTENT_TYPE, "application/pdf")],
            pdf,
        )
            .into_response(),
        Err(error) => {
            tracing::error!(%error, "failed to generate statement PDF");
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": error.to_string()
                })),
            )
                .into_response()
        }
    }
}
