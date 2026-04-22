use std::fs;

use statement_pdf::{StatementPdfError, generate_pdf_from_json, generate_pdf_from_json_file};

#[test]
fn generates_pdf_from_valid_statement_json() {
    let pdf = generate_pdf_from_json_with_stamp().expect("valid statement should render");

    assert!(pdf.starts_with(b"%PDF-"));
    assert!(pdf.len() > 1_000);
}

#[test]
fn rejects_blank_required_fields() {
    let json = r#"{
        "statement_number": "",
        "recipient": {
            "position": "Генеральному директору",
            "company_name": "ООО \"Рога и Копыта\"",
            "full_name": "Кабанову К. К."
        },
        "applicant": {
            "department": "Отдел разработки программного обеспечения",
            "full_name": "Иванов Иван Иванович"
        },
        "body": "Прошу предоставить мне ДМС.",
        "date": "2026-04-20"
    }"#;

    let error =
        generate_pdf_from_json(json.as_bytes()).expect_err("blank statement number must fail");

    match error {
        StatementPdfError::Validation(errors) => {
            assert_eq!(errors.0[0].field, "statement_number");
        }
        other => panic!("expected validation error, got {other:?}"),
    }
}

#[test]
#[should_panic(expected = "required asset is missing or unreadable")]
fn file_generation_panics_without_stamp_next_to_input_json() {
    let temp_dir = tempfile::tempdir().expect("temp dir should be created");
    let statement_path = temp_dir.path().join("statement.json");

    fs::write(
        &statement_path,
        include_bytes!("fixtures/statement.json").as_slice(),
    )
    .expect("statement fixture should be copied");

    let _ = generate_pdf_from_json_file(&statement_path);
}

fn generate_pdf_from_json_with_stamp() -> Result<Vec<u8>, StatementPdfError> {
    let statement = serde_json::from_slice(include_bytes!("fixtures/statement.json"))?;
    statement_pdf::generate_pdf_with_assets(
        statement,
        statement_pdf::StatementAssets {
            stamp_png: Some(include_bytes!("../../../stamp.png").to_vec()),
            signature_images: Default::default(),
        },
    )
}
