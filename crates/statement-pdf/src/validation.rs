use crate::{
    error::{ValidationError, ValidationErrors},
    model::StatementInput,
};

pub(crate) fn validate_statement(input: &StatementInput) -> Result<(), ValidationErrors> {
    let mut errors = Vec::new();

    require_non_blank(
        &mut errors,
        "statement_number",
        &input.statement_number,
        "must not be empty",
    );
    require_non_blank(
        &mut errors,
        "recipient.position",
        &input.recipient.position,
        "must not be empty",
    );
    require_non_blank(
        &mut errors,
        "recipient.company_name",
        &input.recipient.company_name,
        "must not be empty",
    );
    require_non_blank(
        &mut errors,
        "recipient.full_name",
        &input.recipient.full_name,
        "must not be empty",
    );
    require_non_blank(
        &mut errors,
        "applicant.department",
        &input.applicant.department,
        "must not be empty",
    );
    require_non_blank(
        &mut errors,
        "applicant.full_name",
        &input.applicant.full_name,
        "must not be empty",
    );
    require_non_blank(&mut errors, "body", &input.body, "must not be empty");

    if input.body.chars().count() > 4_000 {
        errors.push(ValidationError::new(
            "body",
            "must be at most 4000 characters",
        ));
    }

    if let Some(signature) = &input.signature
        && let Some(signer_name) = &signature.signer_name
    {
        require_non_blank(
            &mut errors,
            "signature.signer_name",
            signer_name,
            "must not be blank when present",
        );
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(ValidationErrors::new(errors))
    }
}

fn require_non_blank(
    errors: &mut Vec<ValidationError>,
    field: &'static str,
    value: &str,
    message: &'static str,
) {
    if value.trim().is_empty() {
        errors.push(ValidationError::new(field, message));
    }
}
