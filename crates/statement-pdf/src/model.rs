use std::path::PathBuf;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StatementInput {
    pub statement_number: String,
    pub recipient: RecipientInput,
    pub applicant: ApplicantInput,
    pub body: String,
    pub date: NaiveDate,

    #[serde(default)]
    pub signature: Option<SignatureInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RecipientInput {
    pub position: String,
    pub company_name: String,
    pub full_name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ApplicantInput {
    pub department: String,
    pub full_name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignatureInput {
    #[serde(default)]
    pub image_path: Option<PathBuf>,

    #[serde(default)]
    pub signer_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct StatementView {
    pub statement_number: String,
    pub recipient: RecipientInput,
    pub applicant: ApplicantInput,
    pub body: String,
    pub date: String,
    pub signature: Option<SignatureView>,
    pub stamp: ImageView,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SignatureView {
    pub image: Option<ImageView>,
    pub signer_name: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ImageView {
    pub path: String,
}

impl StatementInput {
    pub(crate) fn into_view(self, stamp_path: String) -> StatementView {
        StatementView {
            statement_number: self.statement_number,
            recipient: self.recipient,
            applicant: self.applicant,
            body: self.body,
            date: self.date.format("%d.%m.%Y").to_string(),
            signature: self.signature.map(|signature| SignatureView {
                image: signature.image_path.map(|path| ImageView {
                    path: path.to_string_lossy().into_owned(),
                }),
                signer_name: signature.signer_name,
            }),
            stamp: ImageView { path: stamp_path },
        }
    }
}
