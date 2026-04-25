use std::{
    fs,
    path::{Component, Path, PathBuf},
};

use crate::AppState;
use axum::{
    Json,
    extract::{Path as AxumPath, State},
    http::{
        HeaderMap, HeaderValue, StatusCode,
        header::{self, InvalidHeaderValue},
    },
    response::{IntoResponse, Response},
};
use chrono::Utc;
use statement_pdf::{StatementAssets, StatementInput};

const TMP_DIR: &str = "tmp";
const STAMP_PATH: &str = "stamp.png";
const ROUTE_TARGET: &str = "benefits_backend::routes::request";

pub async fn request(
    State(_state): State<AppState>,
    Json(statement): Json<StatementInput>,
) -> Response {
    tracing::info!(target: ROUTE_TARGET, ?statement, "request received");

    let stamp_png = match read_stamp() {
        Ok(stamp_png) => stamp_png,
        Err(error) => {
            tracing::error!(%error, "failed to read stamp.png");
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "stamp.png is missing or unreadable",
            );
        }
    };

    let pdf = match render_pdf(statement.clone(), stamp_png) {
        Ok(pdf) => pdf,
        Err(error) => {
            tracing::error!(%error, "failed to generate statement PDF");
            return json_error(StatusCode::BAD_REQUEST, error.to_string());
        }
    };

    let file_name = build_file_name(&statement.statement_number);
    let file_path = match save_pdf(&file_name, pdf) {
        Ok(path) => path,
        Err(error) => {
            tracing::error!(%error, file_name, "failed to persist generated PDF");
            return json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to persist generated PDF",
            );
        }
    };

    tracing::info!(
        target: ROUTE_TARGET,
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
        return json_error(StatusCode::BAD_REQUEST, "invalid file name");
    };

    match fs::read(&file_path) {
        Ok(pdf) => {
            let Ok(headers) = download_headers(&file_name) else {
                return json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "failed to build download headers",
                );
            };

            (StatusCode::OK, headers, pdf).into_response()
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            json_error(StatusCode::NOT_FOUND, "file not found")
        }
        Err(error) => {
            tracing::error!(%error, path = %file_path.display(), "failed to read generated PDF");
            json_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "failed to read generated PDF",
            )
        }
    }
}

fn read_stamp() -> std::io::Result<Vec<u8>> {
    fs::read(Path::new(STAMP_PATH))
}

fn render_pdf(
    statement: StatementInput,
    stamp_png: Vec<u8>,
) -> Result<Vec<u8>, statement_pdf::StatementPdfError> {
    statement_pdf::generate_pdf_with_assets(
        statement,
        StatementAssets {
            stamp_png: Some(stamp_png),
            signature_images: Default::default(),
        },
    )
}

fn save_pdf(file_name: &str, pdf: Vec<u8>) -> std::io::Result<PathBuf> {
    let tmp_dir = Path::new(TMP_DIR);
    fs::create_dir_all(tmp_dir)?;

    let file_path = tmp_dir.join(file_name);
    fs::write(&file_path, pdf)?;

    Ok(file_path)
}

fn download_headers(file_name: &str) -> Result<HeaderMap, InvalidHeaderValue> {
    let mut headers = HeaderMap::new();
    let content_disposition = format!("attachment; filename=\"{file_name}\"");

    headers.insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static("application/pdf"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        HeaderValue::from_str(&content_disposition)?,
    );

    Ok(headers)
}

fn json_error(status: StatusCode, message: impl Into<String>) -> Response {
    (
        status,
        Json(serde_json::json!({
            "error": message.into()
        })),
    )
        .into_response()
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
