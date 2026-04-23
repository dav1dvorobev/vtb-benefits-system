use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use crate::AppState;
use axum::{
    Json,
    extract::{Path as AxumPath, State},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use chrono::Utc;
use statement_pdf::{StatementAssets, StatementInput};

const TMP_DIR: &str = "tmp";

pub async fn request(
    State(_state): State<AppState>,
    Json(statement): Json<StatementInput>,
) -> Response {
    tracing::info!(
        target: "benefits_api::routes::request",
        ?statement,
        "request received"
    );

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

    let pdf = match statement_pdf::generate_pdf_with_assets(
        statement.clone(),
        StatementAssets {
            stamp_png: Some(stamp_png),
            signature_images: Default::default(),
        },
    ) {
        Ok(pdf) => pdf,
        Err(error) => {
            tracing::error!(%error, "failed to generate statement PDF");
            return (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({
                    "error": error.to_string()
                })),
            )
                .into_response();
        }
    };

    let file_name = build_file_name(&statement.statement_number);
    let tmp_dir = Path::new(TMP_DIR);

    if let Err(error) = fs::create_dir_all(tmp_dir) {
        tracing::error!(%error, dir = %tmp_dir.display(), "failed to create tmp directory");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "failed to create tmp directory"
            })),
        )
            .into_response();
    }

    let file_path = tmp_dir.join(&file_name);
    if let Err(error) = fs::write(&file_path, pdf) {
        tracing::error!(%error, path = %file_path.display(), "failed to write generated PDF");
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "failed to persist generated PDF"
            })),
        )
            .into_response();
    }

    tracing::info!(
        target: "benefits_api::routes::request",
        statement_number = %statement.statement_number,
        path = %file_path.display(),
        "statement PDF saved"
    );

    (
        StatusCode::OK,
        Json(serde_json::json!({
            "statement_number": statement.statement_number,
            "file_name": file_name,
            "download_url": format!("/api/v1/request/download/{file_name}"),
        })),
    )
        .into_response()
}

pub async fn download(
    State(_state): State<AppState>,
    AxumPath(file_name): AxumPath<String>,
) -> Response {
    let Some(file_path) = resolve_download_path(&file_name) else {
        return (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "invalid file name"
            })),
        )
            .into_response();
    };

    match fs::read(&file_path) {
        Ok(pdf) => {
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("application/pdf"));

            let content_disposition = format!("attachment; filename=\"{file_name}\"");
            let Ok(content_disposition) = HeaderValue::from_str(&content_disposition) else {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(serde_json::json!({
                        "error": "failed to build download headers"
                    })),
                )
                    .into_response();
            };

            headers.insert(header::CONTENT_DISPOSITION, content_disposition);

            (StatusCode::OK, headers, pdf).into_response()
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({
                "error": "file not found"
            })),
        )
            .into_response(),
        Err(error) => {
            tracing::error!(%error, path = %file_path.display(), "failed to read generated PDF");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({
                    "error": "failed to read generated PDF"
                })),
            )
                .into_response()
        }
    }
}

fn build_file_name(statement_number: &str) -> String {
    let timestamp = Utc::now().format("%Y%m%d%H%M%S");
    let safe_number = statement_number
        .chars()
        .map(|ch| match ch {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' => ch,
            _ => '_',
        })
        .collect::<String>();

    format!("{safe_number}_{timestamp}.pdf")
}

fn resolve_download_path(file_name: &str) -> Option<PathBuf> {
    let path = Path::new(file_name);
    if path.extension().and_then(|ext| ext.to_str()) != Some("pdf") {
        return None;
    }

    if path
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return None;
    }

    Some(Path::new(TMP_DIR).join(path))
}
