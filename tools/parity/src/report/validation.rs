use crate::*;

mod phase26;
mod phase28;
mod phase30;

pub(crate) use phase26::*;
pub(crate) use phase28::*;
pub(crate) use phase30::*;

pub(crate) fn clean_markdown_cell(cell: &str) -> String {
    cell.trim().replace('`', "")
}

pub(crate) fn is_safety_critical(row: &ChecklistRow) -> bool {
    if row.id.starts_with("EVD-") {
        return false;
    }

    let haystack = format!(
        "{} {} {} {}",
        row.id, row.surface, row.rust_owned_target, row.notes
    )
    .to_ascii_lowercase();

    haystack.contains("safety-critical")
        || row.id.starts_with("PWR-")
        || row.id.starts_with("THR-")
        || row.id.starts_with("SELF-")
        || [
            "voltage",
            "frequency",
            "frequency transition",
            "fan",
            "thermal",
            "power",
            "self-test hardware",
            "hardware-control",
            "runtime input",
            "runtime display",
        ]
        .iter()
        .any(|term| haystack.contains(term))
        || haystack.contains("asic initialization")
        || (row.id.starts_with("ASIC") && haystack.contains("initialization"))
}

pub(crate) fn has_hardware_evidence(row: &ChecklistRow) -> bool {
    has_evidence_token(row, "hardware-smoke") || has_evidence_token(row, "hardware-regression")
}

pub(crate) fn validate_live_asic_mining_verified_row(row: &ChecklistRow) -> Vec<ValidationError> {
    if !is_live_asic_or_mining_row(row) {
        return Vec::new();
    }

    let mut errors = Vec::new();

    if !has_live_asic_mining_evidence(row) {
        errors.push(ValidationError {
            id: row.id.clone(),
            message: "live ASIC/mining verified row requires hardware-smoke or soak evidence"
                .to_owned(),
        });
    }

    if row_contains_live_evidence_blocker(row) {
        errors.push(live_asic_mining_blocker_error(row));
    }

    if row.id == "ASIC-007" && !has_bounded_frequency_transition_regression(row) {
        errors.push(ValidationError {
            id: row.id.clone(),
            message:
                "ASIC-007 verified row requires hardware-regression evidence with a bounded frequency-transition hardware artifact"
                    .to_owned(),
        });
    }

    if row.id == "STR-008" && !has_mining_smoke_or_soak_details(row) {
        errors.push(ValidationError {
            id: row.id.clone(),
            message: "STR-008 verified row requires mining smoke or soak details".to_owned(),
        });
    }

    errors
}

pub(crate) fn is_live_asic_or_mining_row(row: &ChecklistRow) -> bool {
    matches!(
        row.id.as_str(),
        "ASIC-002" | "ASIC-003" | "ASIC-004" | "ASIC-005" | "ASIC-007" | "STR-006" | "STR-008"
    )
}

pub(crate) fn has_live_asic_mining_evidence(row: &ChecklistRow) -> bool {
    has_hardware_evidence(row) || has_evidence_token(row, "soak")
}

pub(crate) fn has_bounded_frequency_transition_regression(row: &ChecklistRow) -> bool {
    let haystack = row_haystack(row);

    has_evidence_token(row, "hardware-regression")
        && haystack.contains("bounded")
        && (haystack.contains("frequency-transition") || haystack.contains("frequency transition"))
        && haystack.contains("hardware")
}

pub(crate) fn has_mining_smoke_or_soak_details(row: &ChecklistRow) -> bool {
    let haystack = row_haystack(row);
    let has_live_share_outcome =
        haystack.contains("accepted share") || haystack.contains("rejected share");
    let has_approved_controlled_no_share_soak = has_evidence_token(row, "soak")
        && haystack.contains("approved")
        && haystack.contains("bounded")
        && haystack.contains("controlled no-share")
        && haystack.contains("soak");
    let has_required_metadata = [
        "board",
        "port",
        "firmware commit",
        "reference commit",
        "redaction",
        "conclusion",
    ]
    .iter()
    .all(|term| haystack.contains(term));

    !row_contains_live_evidence_blocker(row)
        && (has_live_share_outcome || has_approved_controlled_no_share_soak)
        && has_required_metadata
}

pub(crate) fn is_active_safety_control(row: &ChecklistRow) -> bool {
    matches!(
        row.id.as_str(),
        "PWR-001"
            | "PWR-002"
            | "PWR-003"
            | "PWR-005"
            | "ASIC-007"
            | "THR-001"
            | "THR-002"
            | "SELF-001"
            | "UI-003"
    )
}

pub(crate) fn validate_release_ota_verified_row(row: &ChecklistRow) -> Vec<ValidationError> {
    match row.id.as_str() {
        "FS-001" | "OTA-001" | "OTA-002" | "REL-003" if row_contains_live_evidence_blocker(row) => {
            vec![live_evidence_blocker_error(row)]
        }
        "FS-001" => validate_filesystem_verified_row(row),
        "OTA-001" => validate_firmware_ota_verified_row(row),
        "OTA-002" => validate_otawww_verified_row(row),
        "REL-001" | "REL-002" => validate_release_sensitive_verified_row(row),
        "REL-003" => validate_release_image_verified_row(row),
        _ => Vec::new(),
    }
}

pub(crate) fn validate_filesystem_verified_row(row: &ChecklistRow) -> Vec<ValidationError> {
    let missing_terms = missing_required_terms(
        row,
        &[
            RequiredTerm::new("live static", "live static"),
            RequiredTerm::new("/assets/app.css.gz", "/assets/app.css.gz"),
            RequiredTerm::new("missing static redirect", "missing static redirect"),
            RequiredTerm::new("/recovery", "/recovery"),
        ],
    );

    if has_hardware_evidence(row) && missing_terms.is_empty() {
        return Vec::new();
    }

    vec![ValidationError {
        id: row.id.clone(),
        message: format!(
            "FS-001 verified requires hardware-smoke or hardware-regression evidence with live recovery/static smoke covering {}; package-only evidence is insufficient",
            format_required_terms(&missing_terms)
        ),
    }]
}

pub(crate) fn validate_firmware_ota_verified_row(row: &ChecklistRow) -> Vec<ValidationError> {
    let missing_terms = missing_required_terms(
        row,
        &[
            RequiredTerm::new("valid OTA", "valid ota"),
            RequiredTerm::new("invalid image rejection", "invalid image rejection"),
            RequiredTerm::new("boot-validation", "boot-validation"),
        ],
    );

    if has_hardware_evidence(row) && missing_terms.is_empty() {
        return Vec::new();
    }

    vec![ValidationError {
        id: row.id.clone(),
        message: format!(
            "OTA-001 verified requires hardware-smoke or hardware-regression evidence with {}",
            format_required_terms(&missing_terms)
        ),
    }]
}

pub(crate) fn validate_otawww_verified_row(row: &ChecklistRow) -> Vec<ValidationError> {
    if has_evidence_token(row, "hardware-regression")
        && row_haystack(row).contains("interrupted-update")
    {
        return Vec::new();
    }

    vec![ValidationError {
        id: row.id.clone(),
        message:
            "OTA-002 verified requires hardware-regression evidence with an interrupted-update note"
                .to_owned(),
    }]
}

pub(crate) fn validate_release_sensitive_verified_row(row: &ChecklistRow) -> Vec<ValidationError> {
    if has_hardware_evidence(row) || row_haystack(row).contains("release-gate") {
        return Vec::new();
    }

    vec![ValidationError {
        id: row.id.clone(),
        message: "release-sensitive verified rows require hardware-smoke, hardware-regression, or release-gate evidence beyond unit/workflow/api-compare/package-only evidence".to_owned(),
    }]
}

pub(crate) fn validate_release_image_verified_row(row: &ChecklistRow) -> Vec<ValidationError> {
    let haystack = row_haystack(row);
    let has_release_gate = haystack.contains("release-gate");
    let has_provenance = haystack.contains("provenance");
    let has_package_workflow = has_evidence_token(row, "workflow") && haystack.contains("package");
    let missing_terms = missing_required_terms(
        row,
        &[
            RequiredTerm::new("rollback", "rollback"),
            RequiredTerm::new("recovery", "recovery"),
            RequiredTerm::new("large erase", "large erase"),
            RequiredTerm::new("failed update", "failed update"),
            RequiredTerm::new("interrupted-update", "interrupted-update"),
        ],
    );

    if has_release_gate && has_provenance && has_package_workflow && missing_terms.is_empty() {
        return Vec::new();
    }

    vec![ValidationError {
        id: row.id.clone(),
        message: format!(
            "REL-003 verified requires release-gate, provenance, package workflow, and {} evidence",
            format_required_terms(&missing_terms)
        ),
    }]
}
