use super::*;
use bitaxe_automation_contracts::AutomationCommand;

pub(super) fn validate_detector_gate(errors: &mut Vec<String>, manifest: &SafetyAllowManifest) {
    if manifest.board != "205" {
        errors.push("board must be 205".to_owned());
    }

    if manifest.detector_command != DETECTOR_COMMAND {
        errors.push(format!("detector_command must be `{DETECTOR_COMMAND}`"));
    }

    if manifest.detector_port != manifest.port {
        errors.push("detector port mismatch".to_owned());
    }

    if manifest.board_info_status != "passed" {
        errors.push("board-info must pass".to_owned());
    }

    let expected_board_info_command = format!(
        "espflash board-info --chip esp32s3 --port {} --non-interactive",
        manifest.port
    );
    if manifest.board_info_command != expected_board_info_command {
        errors.push(format!(
            "board_info_command must be `{expected_board_info_command}`"
        ));
    }
}

pub(super) fn validate_package_identity(
    errors: &mut Vec<String>,
    manifest: &SafetyAllowManifest,
    package_manifest: &Value,
) {
    let package_source_commit = package_manifest
        .get("source_commit")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let package_reference_commit = package_manifest
        .get("reference_commit")
        .and_then(Value::as_str)
        .unwrap_or_default();

    if package_source_commit == manifest.source_commit
        && package_reference_commit == manifest.reference_commit
    {
        return;
    }

    errors.push(
        "package identity mismatch: package source_commit/reference_commit must match manifest"
            .to_owned(),
    );
}

pub(super) fn validate_surface_and_claim(errors: &mut Vec<String>, manifest: &SafetyAllowManifest) {
    if !ALLOWED_SURFACES.contains(&manifest.surface.as_str()) {
        errors.push(format!("surface `{}` is not allowed", manifest.surface));
    }

    if !ALLOWED_CLAIM_TIERS.contains(&manifest.claim_tier.as_str()) {
        errors.push(format!(
            "claim_tier `{}` is not allowed",
            manifest.claim_tier
        ));
        return;
    }

    let allowed_tiers = allowed_claim_tiers_for_surface(&manifest.surface);
    if !allowed_tiers.is_empty() && !allowed_tiers.contains(&manifest.claim_tier.as_str()) {
        errors.push(format!(
            "surface `{}` does not allow claim_tier `{}`",
            manifest.surface, manifest.claim_tier
        ));
    }

    let expected_evidence_class = expected_evidence_class(&manifest.claim_tier);
    if manifest.evidence_class != expected_evidence_class {
        errors.push(format!(
            "claim_tier `{}` requires evidence_class `{expected_evidence_class}`",
            manifest.claim_tier
        ));
    }
}

pub(super) fn validate_required_procedure_scope(
    errors: &mut Vec<String>,
    manifest: &SafetyAllowManifest,
) {
    validate_workflow_identity(errors, manifest);

    if manifest.constraints.is_null() {
        errors.push("constraints must not be null".to_owned());
    }

    if manifest.evidence_dir.as_str().trim().is_empty() {
        errors.push("evidence_dir must not be empty".to_owned());
    }

    let redaction_reviewer = manifest.redaction_reviewer.trim();
    if redaction_reviewer.is_empty() {
        errors.push("redaction_reviewer must not be empty".to_owned());
    } else if matches!(redaction_reviewer, "pending" | "required-before-citation") {
        errors.push("redaction_reviewer must be completed before citation".to_owned());
    }

    if manifest.checklist_rows.is_empty() {
        errors.push("checklist_rows must not be empty".to_owned());
    }
}

pub(super) fn validate_failure_paths_scope(
    errors: &mut Vec<String>,
    manifest: &SafetyAllowManifest,
) {
    if manifest.surface != "failure-paths" || manifest.claim_tier != "fault-stimulus" {
        return;
    }

    require_string(errors, &manifest.constraints, "stimulus", "fault-stimulus");
    require_string(
        errors,
        &manifest.constraints,
        "expected_fault",
        "fault-stimulus",
    );
    require_string(
        errors,
        &manifest.constraints,
        "abort_condition",
        "fault-stimulus",
    );
    require_string(
        errors,
        &manifest.constraints,
        "restore_path",
        "fault-stimulus",
    );
    require_string(
        errors,
        &manifest.constraints,
        "projection_status",
        "fault-stimulus",
    );
    require_string(
        errors,
        &manifest.constraints,
        "final_safe_state_marker",
        "fault-stimulus",
    );
}

pub(super) fn validate_live_api_websocket_scope(
    errors: &mut Vec<String>,
    manifest: &SafetyAllowManifest,
) {
    if manifest.surface != "live-api-websocket-telemetry" {
        return;
    }

    require_string_value(
        errors,
        &manifest.constraints,
        "network_scan",
        "disabled",
        "live-api-websocket-telemetry",
    );

    if manifest.claim_tier == "unsupported-pending" {
        require_string(
            errors,
            &manifest.constraints,
            "device_url_source",
            "unsupported-pending live-api-websocket-telemetry",
        );
        require_string(
            errors,
            &manifest.constraints,
            "reason",
            "unsupported-pending live-api-websocket-telemetry",
        );
        return;
    }

    if manifest.claim_tier != "api-websocket-projection" {
        return;
    }

    let maybe_device_url_source = manifest
        .constraints
        .get("device_url_source")
        .and_then(Value::as_str)
        .map(str::trim);
    let has_explicit_target = matches!(
        maybe_device_url_source,
        Some("explicit DEVICE_URL" | "trusted raw origin-only target lock")
    );
    if !has_explicit_target {
        errors.push(
            "api-websocket-projection requires constraints.device_url_source to name explicit DEVICE_URL or trusted raw origin-only target lock"
                .to_owned(),
        );
    }

    require_string(
        errors,
        &manifest.constraints,
        "route_path",
        "api-websocket-projection",
    );
    require_positive_integer(
        errors,
        &manifest.constraints,
        "duration_ms",
        "api-websocket-projection",
    );
    require_positive_integer(
        errors,
        &manifest.constraints,
        "max_frames",
        "api-websocket-projection",
    );
}

pub(super) fn validate_active_claim_scope(
    errors: &mut Vec<String>,
    manifest: &SafetyAllowManifest,
) {
    if !is_active_claim_tier(&manifest.claim_tier) {
        return;
    }

    if manifest.recovery_steps.is_empty() {
        errors.push("recovery_steps must not be empty".to_owned());
    }

    if manifest.abort_conditions.is_empty() {
        errors.push("abort_conditions must not be empty".to_owned());
    }

    for required_condition in REQUIRED_ABORT_CONDITIONS {
        if manifest
            .abort_conditions
            .iter()
            .any(|condition| condition == required_condition)
        {
            continue;
        }

        errors.push(format!(
            "abort_conditions must contain `{required_condition}`"
        ));
    }

    if manifest.post_action_safe_state_markers.is_empty() {
        errors.push("post_action_safe_state_markers must not be empty".to_owned());
    }

    for required_marker in REQUIRED_SAFE_STATE_MARKERS {
        if manifest
            .post_action_safe_state_markers
            .iter()
            .any(|marker| marker == required_marker)
        {
            continue;
        }

        errors.push(format!(
            "post_action_safe_state_markers must contain `{required_marker}`"
        ));
    }
}

pub(super) fn validate_filters(
    errors: &mut Vec<String>,
    manifest: &SafetyAllowManifest,
    filters: &SafetyAllowFilters,
) {
    if let Some(expected_surface) = &filters.maybe_surface {
        if &manifest.surface != expected_surface {
            errors.push(format!(
                "surface filter mismatch: manifest `{}` != `{expected_surface}`",
                manifest.surface
            ));
        }
    }

    if let Some(expected_workflow) = &filters.maybe_workflow {
        let actual_workflow = serde_json::to_value(manifest.workflow.command)
            .ok()
            .and_then(|value| value.as_str().map(str::to_owned))
            .unwrap_or_default();
        if &actual_workflow != expected_workflow {
            errors.push(format!(
                "workflow filter mismatch: manifest `{actual_workflow}` != `{expected_workflow}`"
            ));
        }
    }
    if let Some(expected_request_sha256) = &filters.maybe_request_sha256 {
        if &manifest.workflow.request_sha256 != expected_request_sha256 {
            errors.push("request digest filter mismatch".to_owned());
        }
    }
}

fn allowed_claim_tiers_for_surface(surface: &str) -> &'static [&'static str] {
    match surface {
        "safe-baseline" => &[
            "safe-baseline",
            "read-only-observation",
            "safe-unavailable",
            "unsupported-pending",
        ],
        "power-telemetry" => &[
            "read-only-observation",
            "safe-unavailable",
            "unsupported-pending",
        ],
        "voltage-control" => &[
            "bounded-actuation",
            "safe-unavailable",
            "unsupported-pending",
        ],
        "thermal-fan" => &[
            "read-only-observation",
            "bounded-actuation",
            "fault-stimulus",
            "safe-unavailable",
            "unsupported-pending",
        ],
        "self-test-watchdog-load" => &[
            "read-only-observation",
            "self-test-hardware",
            "load-stress",
            "safe-unavailable",
            "unsupported-pending",
        ],
        "display-input" => &[
            "read-only-observation",
            "runtime-display-input",
            "safe-unavailable",
            "unsupported-pending",
        ],
        "failure-paths" => &["fault-stimulus", "safe-unavailable", "unsupported-pending"],
        "live-api-websocket-telemetry" => &[
            "api-websocket-projection",
            "read-only-observation",
            "safe-unavailable",
            "unsupported-pending",
        ],
        "parity-redaction" => &["parity-redaction", "unsupported-pending"],
        _ => &[],
    }
}

fn validate_workflow_identity(errors: &mut Vec<String>, manifest: &SafetyAllowManifest) {
    if manifest.workflow.schema_version != "bitaxe-workflow-identity-v1" {
        errors.push("workflow schema must be bitaxe-workflow-identity-v1".to_owned());
    }
    let expected = if manifest.surface == "parity-redaction" {
        AutomationCommand::VerifyRedaction
    } else if manifest.surface == "safe-baseline" {
        AutomationCommand::CaptureVersionEvidence
    } else {
        AutomationCommand::VerifyHardwareSurface
    };
    if manifest.workflow.command != expected {
        errors.push("workflow command does not match the admitted safety surface".to_owned());
    }
    if manifest.workflow.request_sha256.len() != 64
        || !manifest
            .workflow
            .request_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
    {
        errors
            .push("workflow request_sha256 must be 64 lowercase hexadecimal characters".to_owned());
    }
}

fn require_string(errors: &mut Vec<String>, inputs: &Value, field: &str, claim: &str) {
    let maybe_value = inputs.get(field).and_then(Value::as_str).map(str::trim);
    match maybe_value {
        Some(value) if !value.is_empty() => {}
        _ => errors.push(format!("{claim} requires constraints.{field}")),
    }
}

fn require_string_value(
    errors: &mut Vec<String>,
    inputs: &Value,
    field: &str,
    expected_value: &str,
    claim: &str,
) {
    let maybe_value = inputs.get(field).and_then(Value::as_str).map(str::trim);
    if maybe_value == Some(expected_value) {
        return;
    }

    errors.push(format!(
        "{claim} requires constraints.{field} to equal {expected_value}"
    ));
}

fn require_positive_integer(errors: &mut Vec<String>, inputs: &Value, field: &str, claim: &str) {
    let maybe_value = inputs.get(field).and_then(Value::as_i64);
    let Some(value) = maybe_value else {
        errors.push(format!(
            "{claim} requires constraints.{field} to be positive"
        ));
        return;
    };

    if value <= 0 {
        errors.push(format!(
            "{claim} requires constraints.{field} to be positive"
        ));
    }
}

fn expected_evidence_class(claim_tier: &str) -> &'static str {
    match claim_tier {
        "bounded-actuation"
        | "fault-stimulus"
        | "self-test-hardware"
        | "load-stress"
        | "runtime-display-input" => "hardware-regression",
        "unsupported-pending" => "deferred",
        "parity-redaction" => "workflow",
        "safe-baseline"
        | "read-only-observation"
        | "api-websocket-projection"
        | "safe-unavailable" => "hardware-smoke",
        _ => "unsupported",
    }
}

fn is_active_claim_tier(claim_tier: &str) -> bool {
    ACTIVE_CLAIM_TIERS.contains(&claim_tier)
}
