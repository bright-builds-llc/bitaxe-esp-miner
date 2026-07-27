use crate::*;

pub(crate) fn validate_phase28_hardware_promotion_row(
    row: &ChecklistRow,
    phase30_artifact: &Phase30PromotionArtifactState,
) -> Vec<ValidationError> {
    if !is_phase28_hardware_promotion_row(row) {
        return Vec::new();
    }

    let mut errors = Vec::new();
    let haystack = row_haystack(row);

    if normalize(&row.status) != "verified" {
        return errors;
    }

    if !haystack.contains("phase-28-hardware-evidence-and-checklist-promotion/summary.md") {
        errors.push(ValidationError {
            id: row.id.clone(),
            message: "phase28 verified row missing summary evidence".to_owned(),
        });
    }

    if row_contains_live_evidence_blocker(row) {
        errors.push(ValidationError {
            id: row.id.clone(),
            message: "phase28 blocked verified row must not contain blocker terms".to_owned(),
        });
    }

    if !haystack.contains("redaction-review.md") && !haystack.contains("redaction_status: passed") {
        errors.push(ValidationError {
            id: row.id.clone(),
            message:
                "phase28 redaction evidence requires redaction-review.md or redaction_status: passed"
                    .to_owned(),
        });
    }

    if !haystack.contains("exact_non_claims") {
        errors.push(ValidationError {
            id: row.id.clone(),
            message: "phase28 verified row requires exact_non_claims".to_owned(),
        });
    }

    match row.id.as_str() {
        "STR-09" => {
            if haystack.contains("blocked_safe_prerequisite")
                || !has_str09_accepted_rejected_hardware_share_proof(row)
            {
                errors.push(ValidationError {
                    id: row.id.clone(),
                    message: "STR-09 verified requires accepted or rejected hardware share proof without blocked_safe_prerequisite"
                        .to_owned(),
                });
            }
        }
        "CFG-07" if !has_phase30_exact_promotion_proof(row, phase30_artifact) => {
            errors.push(ValidationError {
                id: row.id.clone(),
                message: "CFG-07 must remain below verified; runtime credential handling lacks hardware proof"
                    .to_owned(),
            });
        }
        "SAFE-10" | "SAFE-11" | "SAFE-12" | "SAFE-13"
            if !has_phase28_live_safety_hardware_proof(row) =>
        {
            errors.push(ValidationError {
                id: row.id.clone(),
                message:
                    "phase28 SAFE verified row requires detector-gated live safety hardware proof"
                        .to_owned(),
            });
        }
        "STR-08" | "ASIC-09" | "ASIC-10" | "ASIC-11" | "ASIC-12"
            if !has_phase28_hardware_bridge_socket_proof(row) =>
        {
            errors.push(ValidationError {
                id: row.id.clone(),
                message: "phase28 ASIC/STR verified row requires matching hardware bridge or socket success proof"
                    .to_owned(),
            });
        }
        _ => {}
    }

    errors
}

pub(crate) fn is_phase28_hardware_promotion_row(row: &ChecklistRow) -> bool {
    let row_identity =
        format!("{} {} {}", row.id, row.surface, row.rust_owned_target).to_ascii_lowercase();

    matches!(
        row.id.as_str(),
        "SAFE-10"
            | "SAFE-11"
            | "SAFE-12"
            | "SAFE-13"
            | "STR-08"
            | "STR-09"
            | "CFG-07"
            | "ASIC-09"
            | "ASIC-10"
            | "ASIC-11"
            | "ASIC-12"
    ) || [
        "phase 28",
        "phase-28-hardware-evidence-and-checklist-promotion",
        "hardware promotion",
        "checklist promotion",
    ]
    .iter()
    .any(|term| row_identity.contains(term))
}

pub(crate) fn has_str09_accepted_rejected_hardware_share_proof(row: &ChecklistRow) -> bool {
    let haystack = row_haystack(row);
    has_hardware_evidence(row)
        && (haystack.contains("accepted share hardware")
            || haystack.contains("rejected share hardware")
            || haystack.contains("accepted share proof")
            || haystack.contains("rejected share proof"))
}

pub(crate) fn has_phase28_live_safety_hardware_proof(row: &ChecklistRow) -> bool {
    let haystack = row_haystack(row);
    has_hardware_evidence(row)
        && (haystack.contains("detector-gated live safety")
            || haystack.contains("live safety hardware proof")
            || haystack.contains("active voltage regression")
            || haystack.contains("thermal fault stimulus hardware"))
}

pub(crate) fn has_phase28_hardware_bridge_socket_proof(row: &ChecklistRow) -> bool {
    let haystack = row_haystack(row);
    has_hardware_evidence(row)
        && (haystack.contains("live socket success")
            || haystack.contains("asic bridge correlation")
            || haystack.contains("accepted share hardware")
            || haystack.contains("rejected share hardware"))
}

pub(crate) fn is_deferred_or_non_205_scope(row: &ChecklistRow) -> bool {
    let haystack = row_haystack(row);
    let row_id = normalize(&row.id);

    matches!(
        row_id.as_str(),
        "cfg-002" | "asic-008" | "asic-009" | "asic-010" | "str-005"
    ) || row_id.starts_with("bap-")
        || haystack.contains("bap")
        || haystack.contains("all-board")
        || haystack.contains("all board")
        || haystack.contains("angular")
}

pub(crate) fn uses_ultra_205_evidence(row: &ChecklistRow) -> bool {
    let haystack = row_haystack(row);
    haystack.contains("ultra 205") || haystack.contains("ultra205")
}

pub(crate) fn has_evidence_token(row: &ChecklistRow, expected: &str) -> bool {
    row.evidence
        .split(',')
        .map(normalize)
        .any(|evidence_kind| evidence_kind == expected)
}

pub(crate) fn row_contains_live_evidence_blocker(row: &ChecklistRow) -> bool {
    let haystack = format!("{} {}", row.evidence, row.notes).to_ascii_lowercase();

    [
        "missing live prerequisites",
        "live prerequisites missing",
        "prerequisites were missing",
        "not run",
        "blocked",
        "pending",
        "below verified",
        "no reachable device_url",
        "unverified",
    ]
    .iter()
    .any(|term| haystack.contains(term))
}

pub(crate) fn live_evidence_blocker_error(row: &ChecklistRow) -> ValidationError {
    ValidationError {
        id: row.id.clone(),
        message: "verified live release/OTA/filesystem rows must not contain blocker terms such as not run, blocked, pending, no reachable DEVICE_URL, or unverified".to_owned(),
    }
}

pub(crate) fn live_asic_mining_blocker_error(row: &ChecklistRow) -> ValidationError {
    ValidationError {
        id: row.id.clone(),
        message: "verified live ASIC/mining rows must not contain blocker terms such as missing live prerequisites, not run, blocked, pending, below verified, no reachable DEVICE_URL, or unverified".to_owned(),
    }
}
