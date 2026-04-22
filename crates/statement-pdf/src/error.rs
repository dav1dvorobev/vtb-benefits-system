use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum StatementPdfError {
    #[error("failed to read or write file")]
    Io(#[from] std::io::Error),
    #[error("failed to parse statement JSON")]
    Json(#[from] serde_json::Error),
    #[error("statement validation failed: {0}")]
    Validation(ValidationErrors),
    #[error("Typst compilation failed:\n{0}")]
    Typst(String),
    #[error("PDF export failed: {0}")]
    Pdf(String),
    #[error("asset not found: {path}")]
    AssetNotFound { path: PathBuf },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationError {
    pub field: &'static str,
    pub message: String,
}

impl ValidationError {
    pub fn new(field: &'static str, message: impl Into<String>) -> Self {
        Self {
            field,
            message: message.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationErrors(pub Vec<ValidationError>);

impl ValidationErrors {
    pub fn new(errors: Vec<ValidationError>) -> Self {
        Self(errors)
    }
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl std::fmt::Display for ValidationErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (index, error) in self.0.iter().enumerate() {
            if index > 0 {
                write!(f, "; ")?;
            }
            write!(f, "{} {}", error.field, error.message)?;
        }
        Ok(())
    }
}
