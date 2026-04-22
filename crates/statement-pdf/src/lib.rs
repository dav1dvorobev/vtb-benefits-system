mod error;
mod model;
mod render;
mod validation;

pub use error::{StatementPdfError, ValidationError, ValidationErrors};
pub use model::{ApplicantInput, RecipientInput, SignatureInput, StatementInput};
pub use render::StatementAssets;

use std::{fs, path::Path};

use validation::validate_statement;

pub fn generate_pdf_from_json(json: &[u8]) -> Result<Vec<u8>, StatementPdfError> {
    let statement = serde_json::from_slice(json)?;
    generate_pdf_with_assets(statement, StatementAssets::default())
}

pub fn generate_pdf(statement: StatementInput) -> Result<Vec<u8>, StatementPdfError> {
    generate_pdf_with_assets(statement, StatementAssets::default())
}

pub fn generate_pdf_with_assets(
    statement: StatementInput,
    assets: StatementAssets,
) -> Result<Vec<u8>, StatementPdfError> {
    generate_pdf_internal(statement, assets)
}

pub fn generate_pdf_from_json_file(
    input_path: impl AsRef<Path>,
) -> Result<Vec<u8>, StatementPdfError> {
    let input_path = input_path.as_ref();
    let json = fs::read(input_path)?;
    let statement: StatementInput = serde_json::from_slice(&json)?;
    validate_statement(&statement).map_err(StatementPdfError::Validation)?;
    let assets = render::read_assets_for_file_input(&statement, input_path)?;
    let document = render::build_document(statement, assets)?;

    compile_document(document)
}

pub fn generate_pdf_to_file(
    input_path: impl AsRef<Path>,
    output_path: impl AsRef<Path>,
) -> Result<(), StatementPdfError> {
    let pdf = generate_pdf_from_json_file(input_path)?;
    fs::write(output_path, pdf)?;

    Ok(())
}

fn generate_pdf_internal(
    statement: StatementInput,
    assets: StatementAssets,
) -> Result<Vec<u8>, StatementPdfError> {
    validate_statement(&statement).map_err(StatementPdfError::Validation)?;
    let document = render::build_document(statement, assets)?;

    compile_document(document)
}

fn compile_document(document: render::TypstDocument) -> Result<Vec<u8>, StatementPdfError> {
    typst_engine::compile_pdf(document)
}

mod typst_engine;
