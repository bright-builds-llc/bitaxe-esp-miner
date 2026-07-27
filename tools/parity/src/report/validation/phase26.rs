use crate::*;

pub(crate) fn validate_deferred_scope_verified_row(row: &ChecklistRow) -> Vec<ValidationError> {
    if !is_deferred_or_non_205_scope(row) || !uses_ultra_205_evidence(row) {
        return Vec::new();
    }

    vec![ValidationError {
        id: row.id.clone(),
        message: "deferred or non-205 verified rows cannot reuse Ultra 205 evidence".to_owned(),
    }]
}

pub(crate) fn validate_phase26_telemetry_verified_row(row: &ChecklistRow) -> Vec<ValidationError> {
    if !is_phase26_telemetry_row(row) {
        return Vec::new();
    }

    let mut errors = Vec::new();
    let haystack = row_haystack(row);

    if !haystack.contains("phase-26-telemetry-and-parity-closure/summary.md") {
        errors.push(ValidationError {
            id: row.id.clone(),
            message: "phase26 verified row missing summary evidence".to_owned(),
        });
    }

    if row_contains_live_evidence_blocker(row) {
        errors.push(ValidationError {
            id: row.id.clone(),
            message: "phase26 blocked verified row must not contain blocker terms".to_owned(),
        });
    }

    if !haystack.contains("redaction-review.md") && !haystack.contains("redaction_status: passed") {
        errors.push(ValidationError {
            id: row.id.clone(),
            message: "phase26 redaction evidence requires redaction-review.md or redaction_status: passed".to_owned(),
        });
    }

    if !haystack.contains("exact_non_claims") {
        errors.push(ValidationError {
            id: row.id.clone(),
            message: "phase26 verified row requires exact_non_claims".to_owned(),
        });
    }

    match row.id.as_str() {
        "STAT-002" if !haystack.contains("no_request_time_fabrication") => {
            errors.push(ValidationError {
                id: row.id.clone(),
                message: "phase26 statistics verified row requires no_request_time_fabrication"
                    .to_owned(),
            });
        }
        "STAT-003" if !haystack.contains("empty_without_parsed_share_outcome") => {
            errors.push(ValidationError {
                id: row.id.clone(),
                message:
                    "phase26 scoreboard verified row requires empty_without_parsed_share_outcome"
                        .to_owned(),
            });
        }
        "EVD-08" => {
            let missing_terms = missing_required_terms(
                row,
                &[
                    RequiredTerm::new("API-11", "api-11"),
                    RequiredTerm::new("API-12", "api-12"),
                    RequiredTerm::new("API-13", "api-13"),
                    RequiredTerm::new("EVD-08", "evd-08"),
                    RequiredTerm::new("redaction_status: passed", "redaction_status: passed"),
                ],
            );

            if !missing_terms.is_empty() {
                errors.push(ValidationError {
                    id: row.id.clone(),
                    message: format!(
                        "EVD-08 verified row requires {}",
                        format_required_terms(&missing_terms)
                    ),
                });
            }
        }
        _ => {}
    }

    errors
}

pub(crate) fn is_phase26_telemetry_row(row: &ChecklistRow) -> bool {
    let row_identity =
        format!("{} {} {}", row.id, row.surface, row.rust_owned_target).to_ascii_lowercase();

    matches!(
        row.id.as_str(),
        "API-002" | "API-006" | "STAT-002" | "STAT-003" | "EVD-08"
    ) || [
        "statistics",
        "scoreboard",
        "websocket telemetry",
        "system info response",
        "phase 26",
    ]
    .iter()
    .any(|term| row_identity.contains(term))
}
