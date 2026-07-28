use crate::*;

pub(crate) fn validate_phase30_promotion_row(
    row: &ChecklistRow,
    artifact_state: &Phase30PromotionArtifactState,
) -> Vec<ValidationError> {
    if !is_phase30_promotion_row(row) || normalize(&row.status) != "verified" {
        return Vec::new();
    }

    let mut errors = Vec::new();
    if !row_haystack(row).contains(DEFAULT_PHASE30_PROMOTION_ARTIFACT_PATH) {
        errors.push(ValidationError {
            id: row.id.clone(),
            message: format!(
                "Phase 30 admission requires exact artifact breadcrumb {DEFAULT_PHASE30_PROMOTION_ARTIFACT_PATH}"
            ),
        });
    }

    if let Some(forbidden_category) = maybe_phase30_forbidden_category(row) {
        errors.push(ValidationError {
            id: row.id.clone(),
            message: format!("Phase 30 admission forbids no-proof category {forbidden_category}"),
        });
    }

    let artifact = match artifact_state {
        Phase30PromotionArtifactState::Available(artifact) => artifact,
        Phase30PromotionArtifactState::Unavailable(message)
        | Phase30PromotionArtifactState::Malformed(message) => {
            errors.push(ValidationError {
                id: row.id.clone(),
                message: format!("Phase 30 admission rejected artifact: {message}"),
            });
            return errors;
        }
    };

    if artifact.has_exact_field("phase30_disposition", "no_promotion_no_eligible_evidence") {
        errors.push(ValidationError {
            id: row.id.clone(),
            message: "Phase 30 evidence artifact records no_promotion_no_eligible_evidence"
                .to_owned(),
        });
        return errors;
    }

    let missing_row_fields = phase30_missing_artifact_row_fields(row, artifact);
    if !missing_row_fields.is_empty() {
        errors.push(ValidationError {
            id: row.id.clone(),
            message: format!(
                "Phase 30 {} structured proof requires {}",
                row.id,
                format_required_terms(&missing_row_fields)
            ),
        });
    }

    errors
}

pub(crate) fn is_phase30_promotion_row(row: &ChecklistRow) -> bool {
    matches!(row.id.as_str(), "STR-09" | "CFG-07" | "ASIC-11")
}

pub(crate) fn maybe_phase30_forbidden_category(row: &ChecklistRow) -> Option<&'static str> {
    let haystack = row_haystack(row);

    [
        "no_promotion_no_eligible_evidence",
        "gaps_found",
        "eligible_share_outcome: none",
        "blocked_safe_prerequisite",
        "workflow-only",
        "fake-pool",
        "deterministic-only",
    ]
    .into_iter()
    .find(|category| haystack.contains(category))
}

pub(crate) fn phase30_missing_artifact_row_fields(
    row: &ChecklistRow,
    artifact: &Phase30PromotionArtifact,
) -> Vec<&'static str> {
    let required_fields: &[(&'static str, &'static str)] = match row.id.as_str() {
        "STR-09" => &[
            ("STR-09.live_submit_response_classified", "true"),
            ("STR-09.asic_correlation", "passed"),
            ("STR-09.safe_stop_status", "complete"),
        ],
        "CFG-07" => &[
            ("CFG-07.runtime_credentials_input", "local-owner-supplied"),
            ("CFG-07.live_mining_credentials_consumed", "true"),
            ("CFG-07.committed_credential_values", "none"),
            ("CFG-07.safe_stop_status", "complete"),
        ],
        "ASIC-11" => &[
            ("ASIC-11.asic_result_to_active_work", "correlated"),
            ("ASIC-11.submit_intent_from_correlated_result", "true"),
            ("ASIC-11.safe_stop_status", "complete"),
        ],
        _ => &[],
    };

    required_fields
        .iter()
        .filter_map(|(key, value)| (!artifact.has_exact_field(key, value)).then_some(*key))
        .collect()
}

pub(crate) fn has_phase30_exact_promotion_proof(
    row: &ChecklistRow,
    artifact_state: &Phase30PromotionArtifactState,
) -> bool {
    let Phase30PromotionArtifactState::Available(artifact) = artifact_state else {
        return false;
    };

    is_phase30_promotion_row(row)
        && row_haystack(row).contains(DEFAULT_PHASE30_PROMOTION_ARTIFACT_PATH)
        && maybe_phase30_forbidden_category(row).is_none()
        && artifact.has_exact_field("phase30_disposition", "promoted")
        && phase30_missing_artifact_row_fields(row, artifact).is_empty()
}
