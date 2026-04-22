use std::{collections::BTreeMap, fs, path::Path};

use crate::{error::StatementPdfError, model::StatementInput};

#[derive(Clone, Debug, Default)]
pub struct StatementAssets {
    pub stamp_png: Option<Vec<u8>>,
    pub signature_images: BTreeMap<String, Vec<u8>>,
}

#[derive(Clone, Debug)]
pub(crate) struct TypstDocument {
    pub main_source: String,
    pub statement_json: String,
    pub assets: BTreeMap<String, Vec<u8>>,
}

pub(crate) fn build_document(
    statement: StatementInput,
    assets: StatementAssets,
) -> Result<TypstDocument, StatementPdfError> {
    let stamp_png = assets
        .stamp_png
        .expect("stamp image must be loaded before building a statement document");

    let view = statement.into_view("/stamp.png".to_owned());
    let mut document_assets = assets.signature_images;

    document_assets.insert("/stamp.png".to_owned(), stamp_png);

    Ok(TypstDocument {
        main_source: include_str!("../templates/statement.typ").to_owned(),
        statement_json: serde_json::to_string(&view)?,
        assets: document_assets,
    })
}

pub(crate) fn read_assets_for_file_input(
    statement: &StatementInput,
    input_path: &Path,
) -> Result<StatementAssets, StatementPdfError> {
    let base_dir = input_path.parent().unwrap_or_else(|| Path::new("."));
    let mut assets = StatementAssets {
        stamp_png: Some(read_required_asset(base_dir, Path::new("stamp.png"))),
        ..Default::default()
    };

    if let Some(signature) = &statement.signature
        && let Some(image_path) = &signature.image_path
    {
        let virtual_path = image_path.to_string_lossy().into_owned();
        assets
            .signature_images
            .insert(virtual_path, read_asset(base_dir, image_path)?);
    }

    Ok(assets)
}

fn read_required_asset(base_dir: &Path, path: &Path) -> Vec<u8> {
    let full_path = resolve_asset_path(base_dir, path);

    fs::read(&full_path).unwrap_or_else(|source| {
        panic!(
            "required asset is missing or unreadable: {} ({source})",
            full_path.display()
        )
    })
}

fn read_asset(base_dir: &Path, path: &Path) -> Result<Vec<u8>, StatementPdfError> {
    let full_path = resolve_asset_path(base_dir, path);

    fs::read(&full_path).map_err(|source| {
        if source.kind() == std::io::ErrorKind::NotFound {
            StatementPdfError::AssetNotFound { path: full_path }
        } else {
            StatementPdfError::Io(source)
        }
    })
}

fn resolve_asset_path(base_dir: &Path, path: &Path) -> std::path::PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        base_dir.join(path)
    }
}
