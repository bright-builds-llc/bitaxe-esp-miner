use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Deserializer};
use serde_json::Value;

const DETECTOR_COMMAND: &str = "just detect-ultra205";
const REQUIRED_ABORT_CONDITIONS: &[&str] = &[
    "detector_mismatch",
    "board_info_failure",
    "missing_safe_state_marker",
];
const REQUIRED_SAFE_STATE_MARKERS: &[&str] =
    &["safe_state: mining=disabled", "hardware_control=disabled"];
const ACTIVE_CLAIM_TIERS: &[&str] = &[
    "bounded-actuation",
    "fault-stimulus",
    "self-test-hardware",
    "load-stress",
    "runtime-display-input",
];
const ALLOWED_SURFACES: &[&str] = &[
    "safe-baseline",
    "power-telemetry",
    "voltage-control",
    "thermal-fan",
    "self-test-watchdog-load",
    "display-input",
    "failure-paths",
    "live-api-websocket-telemetry",
    "parity-redaction",
];
const ALLOWED_CLAIM_TIERS: &[&str] = &[
    "safe-baseline",
    "read-only-observation",
    "bounded-actuation",
    "fault-stimulus",
    "self-test-hardware",
    "load-stress",
    "runtime-display-input",
    "api-websocket-projection",
    "safe-unavailable",
    "unsupported-pending",
    "parity-redaction",
];

#[derive(Debug, Deserialize)]
pub(crate) struct SafetyAllowManifest {
    pub(crate) board: String,
    pub(crate) port: String,
    pub(crate) detector_command: String,
    pub(crate) detector_port: String,
    pub(crate) board_info_command: String,
    pub(crate) board_info_status: String,
    #[serde(deserialize_with = "deserialize_utf8_path_buf")]
    pub(crate) package_manifest: Utf8PathBuf,
    pub(crate) source_commit: String,
    pub(crate) reference_commit: String,
    pub(crate) surface: String,
    pub(crate) claim_tier: String,
    pub(crate) evidence_class: String,
    pub(crate) allowed_command: String,
    pub(crate) allowed_inputs: Value,
    pub(crate) abort_conditions: Vec<String>,
    pub(crate) recovery_steps: Vec<String>,
    pub(crate) post_action_safe_state_markers: Vec<String>,
    #[serde(deserialize_with = "deserialize_utf8_path_buf")]
    pub(crate) evidence_dir: Utf8PathBuf,
    pub(crate) redaction_reviewer: String,
    pub(crate) checklist_rows: Vec<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct SafetyAllowReport {
    pub(crate) validation_errors: Vec<String>,
}

impl SafetyAllowReport {
    pub(crate) fn passed(&self) -> bool {
        self.validation_errors.is_empty()
    }
}

#[derive(Debug)]
pub(crate) struct SafetyAllowDocuments {
    pub(crate) manifest: SafetyAllowManifest,
    pub(crate) package_manifest: Value,
}

#[derive(Debug)]
pub(crate) struct SafetyAllowFilters {
    pub(crate) maybe_surface: Option<String>,
    pub(crate) maybe_allowed_command: Option<String>,
}

pub(crate) fn load_safety_allow_documents(
    workspace_dir: &Utf8Path,
    manifest_path: &Utf8Path,
) -> Result<SafetyAllowDocuments> {
    let manifest_json = std::fs::read_to_string(manifest_path.as_std_path())
        .with_context(|| format!("failed to read safety allow manifest {manifest_path}"))?;
    let manifest: SafetyAllowManifest = serde_json::from_str(&manifest_json)
        .with_context(|| format!("safety allow manifest {manifest_path} is not valid JSON"))?;
    let package_manifest_path = resolve_workspace_path(workspace_dir, &manifest.package_manifest);
    let package_manifest_json = std::fs::read_to_string(package_manifest_path.as_std_path())
        .with_context(|| format!("failed to read package manifest {package_manifest_path}"))?;
    let package_manifest: Value = serde_json::from_str(&package_manifest_json)
        .with_context(|| format!("package manifest {package_manifest_path} is not valid JSON"))?;

    Ok(SafetyAllowDocuments {
        manifest,
        package_manifest,
    })
}

pub(crate) fn validate_safety_allow_documents(
    documents: &SafetyAllowDocuments,
    filters: &SafetyAllowFilters,
) -> SafetyAllowReport {
    let mut report =
        validate_safety_allow_manifest(&documents.manifest, &documents.package_manifest);
    validate_filters(&mut report.validation_errors, &documents.manifest, filters);
    report
}

pub(crate) fn validate_safety_allow_manifest(
    manifest: &SafetyAllowManifest,
    package_manifest: &Value,
) -> SafetyAllowReport {
    let mut validation_errors = Vec::new();

    validate_detector_gate(&mut validation_errors, manifest);
    validate_package_identity(&mut validation_errors, manifest, package_manifest);
    validate_surface_and_claim(&mut validation_errors, manifest);
    validate_required_procedure_scope(&mut validation_errors, manifest);
    validate_failure_paths_scope(&mut validation_errors, manifest);
    validate_live_api_websocket_scope(&mut validation_errors, manifest);
    validate_active_claim_scope(&mut validation_errors, manifest);

    SafetyAllowReport { validation_errors }
}

pub(crate) fn render_safety_allow_report(
    manifest: &SafetyAllowManifest,
    report: &SafetyAllowReport,
) -> String {
    if report.passed() {
        return format!(
            "safety_allow_status: passed\nsurface: {}\nclaim_tier: {}\nevidence_class: {}\nchecklist_rows: {}\n",
            manifest.surface,
            manifest.claim_tier,
            manifest.evidence_class,
            manifest.checklist_rows.join(",")
        );
    }

    let mut output = String::from("safety_allow_status: failed\nvalidation_errors:\n");
    for error in &report.validation_errors {
        output.push_str("- ");
        output.push_str(error);
        output.push('\n');
    }
    output
}

fn validate_detector_gate(errors: &mut Vec<String>, manifest: &SafetyAllowManifest) {
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

fn validate_package_identity(
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

fn validate_surface_and_claim(errors: &mut Vec<String>, manifest: &SafetyAllowManifest) {
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

fn validate_required_procedure_scope(errors: &mut Vec<String>, manifest: &SafetyAllowManifest) {
    if manifest.allowed_command.trim().is_empty() {
        errors.push("allowed_command must not be empty".to_owned());
    } else {
        validate_allowed_command_scope(errors, manifest);
    }

    if manifest.allowed_inputs.is_null() {
        errors.push("allowed_inputs must not be null".to_owned());
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

fn validate_failure_paths_scope(errors: &mut Vec<String>, manifest: &SafetyAllowManifest) {
    if manifest.surface != "failure-paths" || manifest.claim_tier != "fault-stimulus" {
        return;
    }

    require_string(
        errors,
        &manifest.allowed_inputs,
        "stimulus",
        "fault-stimulus",
    );
    require_string(
        errors,
        &manifest.allowed_inputs,
        "expected_fault",
        "fault-stimulus",
    );
    require_string(
        errors,
        &manifest.allowed_inputs,
        "abort_condition",
        "fault-stimulus",
    );
    require_string(
        errors,
        &manifest.allowed_inputs,
        "restore_path",
        "fault-stimulus",
    );
    require_string(
        errors,
        &manifest.allowed_inputs,
        "projection_status",
        "fault-stimulus",
    );
    require_string(
        errors,
        &manifest.allowed_inputs,
        "final_safe_state_marker",
        "fault-stimulus",
    );
}

fn validate_live_api_websocket_scope(errors: &mut Vec<String>, manifest: &SafetyAllowManifest) {
    if manifest.surface != "live-api-websocket-telemetry" {
        return;
    }

    require_string_value(
        errors,
        &manifest.allowed_inputs,
        "network_scan",
        "disabled",
        "live-api-websocket-telemetry",
    );

    if manifest.claim_tier == "unsupported-pending" {
        require_string(
            errors,
            &manifest.allowed_inputs,
            "device_url_source",
            "unsupported-pending live-api-websocket-telemetry",
        );
        require_string(
            errors,
            &manifest.allowed_inputs,
            "reason",
            "unsupported-pending live-api-websocket-telemetry",
        );
        return;
    }

    if manifest.claim_tier != "api-websocket-projection" {
        return;
    }

    let maybe_device_url_source = manifest
        .allowed_inputs
        .get("device_url_source")
        .and_then(Value::as_str)
        .map(str::trim);
    let has_explicit_target = matches!(
        maybe_device_url_source,
        Some("explicit DEVICE_URL" | "trusted raw origin-only target lock")
    );
    if !has_explicit_target {
        errors.push(
            "api-websocket-projection requires allowed_inputs.device_url_source to name explicit DEVICE_URL or trusted raw origin-only target lock"
                .to_owned(),
        );
    }

    require_string(
        errors,
        &manifest.allowed_inputs,
        "route_path",
        "api-websocket-projection",
    );
    require_positive_integer(
        errors,
        &manifest.allowed_inputs,
        "duration_ms",
        "api-websocket-projection",
    );
    require_positive_integer(
        errors,
        &manifest.allowed_inputs,
        "max_frames",
        "api-websocket-projection",
    );

    if maybe_device_url_source == Some("explicit DEVICE_URL")
        && !has_option(&manifest.allowed_command, "--device-url")
    {
        errors.push(
            "api-websocket-projection with explicit DEVICE_URL requires allowed_command to include --device-url"
                .to_owned(),
        );
    }
}

fn validate_active_claim_scope(errors: &mut Vec<String>, manifest: &SafetyAllowManifest) {
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

fn validate_filters(
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

    if let Some(expected_allowed_command) = &filters.maybe_allowed_command {
        if &manifest.allowed_command != expected_allowed_command {
            errors.push(format!(
                "allowed command filter mismatch: manifest `{}` != `{expected_allowed_command}`",
                manifest.allowed_command
            ));
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

fn validate_allowed_command_scope(errors: &mut Vec<String>, manifest: &SafetyAllowManifest) {
    let tokens: Vec<&str> = manifest.allowed_command.split_whitespace().collect();

    if is_expected_safety_command(manifest, &tokens) {
        return;
    }

    errors.push(
        "allowed_command must route through an approved safety evidence wrapper for its surface"
            .to_owned(),
    );
}

fn is_expected_safety_command(manifest: &SafetyAllowManifest, tokens: &[&str]) -> bool {
    match manifest.surface.as_str() {
        "safe-baseline" => {
            starts_with_tokens(
                tokens,
                &["bazel", "run", "//tools/flash:flash", "--", "flash-monitor"],
            ) && option_equals(tokens, "--board", "205")
                && option_equals(tokens, "--port", &manifest.port)
                && has_option_with_value(tokens, "--manifest")
                && has_option_with_value(tokens, "--evidence-dir")
        }
        "power-telemetry" | "voltage-control" => {
            starts_with_tokens(tokens, &["scripts/phase14-power-voltage.sh"])
                && option_equals(tokens, "--surface", &manifest.surface)
                && has_option_with_value(tokens, "--manifest")
                && has_option_with_value(tokens, "--out-dir")
        }
        "thermal-fan" => {
            starts_with_tokens(tokens, &["scripts/phase14-thermal-fan.sh"])
                && option_equals(tokens, "--surface", "thermal-fan")
                && has_option_with_value(tokens, "--manifest")
                && has_option_with_value(tokens, "--out-dir")
        }
        "self-test-watchdog-load" => {
            starts_with_tokens(tokens, &["scripts/phase14-self-test-watchdog-load.sh"])
                && has_option_with_value(tokens, "--manifest")
                && has_option_with_value(tokens, "--out-dir")
        }
        "display-input" => {
            starts_with_tokens(tokens, &["scripts/phase14-display-input.sh"])
                && has_option_with_value(tokens, "--manifest")
                && has_option_with_value(tokens, "--out-dir")
        }
        "failure-paths" => {
            starts_with_tokens(tokens, &["scripts/phase20-failure-paths.sh"])
                && has_option_with_value(tokens, "--manifest")
                && has_option_with_value(tokens, "--out-dir")
                && !tokens.contains(&"--stimulus")
        }
        "live-api-websocket-telemetry" => {
            starts_with_tokens(tokens, &["scripts/phase14-live-telemetry.sh"])
                && has_option_with_value(tokens, "--manifest")
                && has_option_with_value(tokens, "--out-dir")
        }
        "parity-redaction" => starts_with_tokens(tokens, &["rg"]),
        _ => false,
    }
}

fn starts_with_tokens(tokens: &[&str], expected_prefix: &[&str]) -> bool {
    tokens.starts_with(expected_prefix)
}

fn option_equals(tokens: &[&str], option: &str, expected_value: &str) -> bool {
    tokens
        .windows(2)
        .any(|window| window[0] == option && window[1] == expected_value)
}

fn has_option_with_value(tokens: &[&str], option: &str) -> bool {
    tokens
        .windows(2)
        .any(|window| window[0] == option && !window[1].starts_with("--"))
}

fn has_option(command: &str, option: &str) -> bool {
    command.split_whitespace().any(|token| token == option)
}

fn require_string(errors: &mut Vec<String>, inputs: &Value, field: &str, claim: &str) {
    let maybe_value = inputs.get(field).and_then(Value::as_str).map(str::trim);
    match maybe_value {
        Some(value) if !value.is_empty() => {}
        _ => errors.push(format!("{claim} requires allowed_inputs.{field}")),
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
        "{claim} requires allowed_inputs.{field} to equal {expected_value}"
    ));
}

fn require_positive_integer(errors: &mut Vec<String>, inputs: &Value, field: &str, claim: &str) {
    let maybe_value = inputs.get(field).and_then(Value::as_i64);
    let Some(value) = maybe_value else {
        errors.push(format!(
            "{claim} requires allowed_inputs.{field} to be positive"
        ));
        return;
    };

    if value <= 0 {
        errors.push(format!(
            "{claim} requires allowed_inputs.{field} to be positive"
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

fn resolve_workspace_path(workspace_dir: &Utf8Path, path: &Utf8Path) -> Utf8PathBuf {
    if path.is_absolute() {
        return path.to_owned();
    }

    workspace_dir.join(path)
}

fn deserialize_utf8_path_buf<'de, D>(deserializer: D) -> Result<Utf8PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let path = String::deserialize(deserializer)?;
    Ok(Utf8PathBuf::from(path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn safety_allow_rejects_non_205_board() {
        // Arrange
        let (manifest, package_manifest) = manifest_with_change(|json| {
            json["board"] = serde_json::json!("601");
        });

        // Act
        let report = validate_safety_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert_error_contains(&report, "board must be 205");
    }

    #[test]
    fn safety_allow_rejects_detector_port_mismatch() {
        // Arrange
        let (manifest, package_manifest) = manifest_with_change(|json| {
            json["detector_port"] = serde_json::json!("/dev/cu.usbmodem9999");
        });

        // Act
        let report = validate_safety_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert_error_contains(&report, "detector port mismatch");
    }

    #[test]
    fn safety_allow_rejects_failed_board_info() {
        // Arrange
        let (manifest, package_manifest) = manifest_with_change(|json| {
            json["board_info_status"] = serde_json::json!("failed");
        });

        // Act
        let report = validate_safety_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert_error_contains(&report, "board-info must pass");
    }

    #[test]
    fn safety_allow_rejects_package_identity_mismatch() {
        // Arrange
        let (manifest, mut package_manifest) = manifest_with_change(|_json| {});
        package_manifest["source_commit"] = serde_json::json!("stale-source");

        // Act
        let report = validate_safety_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert_error_contains(&report, "package identity mismatch");
    }

    #[test]
    fn safety_allow_active_claim_tiers_require_hardware_regression() {
        // Arrange
        let active_tiers = [
            "bounded-actuation",
            "fault-stimulus",
            "self-test-hardware",
            "load-stress",
            "runtime-display-input",
        ];

        for claim_tier in active_tiers {
            let (manifest, package_manifest) = manifest_with_change(|json| {
                json["claim_tier"] = serde_json::json!(claim_tier);
                json["evidence_class"] = serde_json::json!("hardware-smoke");
            });

            // Act
            let report = validate_safety_allow_manifest(&manifest, &package_manifest);

            // Assert
            assert_error_contains(&report, "hardware-regression");
        }
    }

    #[test]
    fn safety_allow_allows_failure_paths_fault_stimulus_with_hardware_regression() {
        // Arrange
        let (claim_tier, evidence_class) = ("fault-stimulus", "hardware-regression");
        let (manifest, package_manifest) = manifest_with_change(|json| {
            json["surface"] = serde_json::json!("failure-paths");
            json["claim_tier"] = serde_json::json!(claim_tier);
            json["evidence_class"] = serde_json::json!(evidence_class);
            json["allowed_command"] =
                serde_json::json!("scripts/phase20-failure-paths.sh --manifest allow.json --out-dir evidence/failure-paths");
            json["allowed_inputs"] = serde_json::json!({
                "stimulus": "fan-rpm-unavailable",
                "expected_fault": "fan_fault",
                "abort_condition": "missing_safe_state_marker",
                "restore_path": "just flash board=205 port=/dev/cu.usbmodem1101",
                "projection_status": "api-and-websocket-observed",
                "final_safe_state_marker": "safe_state: mining=disabled"
            });
            json["evidence_dir"] = serde_json::json!(
                "docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/failure-paths"
            );
            json["checklist_rows"] = serde_json::json!(["SAFE-04"]);
        });

        // Act
        let report = validate_safety_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert!(report.passed(), "{report:#?}");
    }

    #[test]
    fn safety_allow_rejects_failure_paths_fault_stimulus_without_hardware_regression() {
        // Arrange
        let (manifest, package_manifest) = manifest_with_change(|json| {
            json["surface"] = serde_json::json!("failure-paths");
            json["claim_tier"] = serde_json::json!("fault-stimulus");
            json["evidence_class"] = serde_json::json!("hardware-smoke");
        });

        // Act
        let report = validate_safety_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert_error_contains(&report, "hardware-regression");
    }

    #[test]
    fn safety_allow_rejects_failure_paths_unrelated_claim_tier() {
        // Arrange
        let (manifest, package_manifest) = manifest_with_change(|json| {
            json["surface"] = serde_json::json!("failure-paths");
            json["claim_tier"] = serde_json::json!("read-only-observation");
            json["evidence_class"] = serde_json::json!("hardware-smoke");
            json["allowed_command"] = serde_json::json!(
                "scripts/phase20-failure-paths.sh --manifest allow.json --out-dir evidence/failure-paths"
            );
            json["abort_conditions"] = serde_json::json!([]);
            json["recovery_steps"] = serde_json::json!([]);
            json["post_action_safe_state_markers"] = serde_json::json!([]);
        });

        // Act
        let report = validate_safety_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert_error_contains(&report, "does not allow claim_tier `read-only-observation`");
    }

    #[test]
    fn safety_allow_rejects_failure_paths_fault_stimulus_missing_required_inputs() {
        // Arrange
        let (manifest, package_manifest) = manifest_with_change(|json| {
            json["surface"] = serde_json::json!("failure-paths");
            json["claim_tier"] = serde_json::json!("fault-stimulus");
            json["evidence_class"] = serde_json::json!("hardware-regression");
            json["allowed_command"] = serde_json::json!(
                "scripts/phase20-failure-paths.sh --manifest allow.json --out-dir evidence/failure-paths"
            );
            json["allowed_inputs"] = serde_json::json!({
                "restore_path": "just flash board=205 port=/dev/cu.usbmodem1101"
            });
        });

        // Act
        let report = validate_safety_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert_error_contains(&report, "allowed_inputs.stimulus");
        assert_error_contains(&report, "allowed_inputs.expected_fault");
        assert_error_contains(&report, "allowed_inputs.abort_condition");
        assert_error_contains(&report, "allowed_inputs.projection_status");
        assert_error_contains(&report, "allowed_inputs.final_safe_state_marker");
    }

    #[test]
    fn safety_allow_allows_failure_paths_unsupported_pending_deferred_without_recovery() {
        // Arrange
        let (manifest, package_manifest) = manifest_with_change(|json| {
            json["surface"] = serde_json::json!("failure-paths");
            json["claim_tier"] = serde_json::json!("unsupported-pending");
            json["evidence_class"] = serde_json::json!("deferred");
            json["allowed_command"] =
                serde_json::json!("scripts/phase20-failure-paths.sh --manifest allow.json --out-dir evidence/failure-paths");
            json["allowed_inputs"] = serde_json::json!({
                "blocked_by": "no compile-gated fault stimulus route"
            });
            json["abort_conditions"] = serde_json::json!([]);
            json["recovery_steps"] = serde_json::json!([]);
            json["post_action_safe_state_markers"] = serde_json::json!([]);
            json["evidence_dir"] = serde_json::json!(
                "docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence/failure-paths"
            );
            json["checklist_rows"] = serde_json::json!(["SAFE-04"]);
        });

        // Act
        let report = validate_safety_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert!(report.passed(), "{report:#?}");
    }

    #[test]
    fn safety_allow_rejects_live_api_projection_without_explicit_target() {
        // Arrange
        let (manifest, package_manifest) = manifest_with_change(|json| {
            json["surface"] = serde_json::json!("live-api-websocket-telemetry");
            json["claim_tier"] = serde_json::json!("api-websocket-projection");
            json["evidence_class"] = serde_json::json!("hardware-smoke");
            json["allowed_command"] = serde_json::json!(
                "scripts/phase14-live-telemetry.sh --manifest allow.json --out-dir evidence/live-api"
            );
            json["allowed_inputs"] = serde_json::json!({
                "device_url_source": "missing",
                "network_scan": "disabled",
                "route_path": "/api/system/info",
                "duration_ms": 10000,
                "max_frames": 5
            });
            json["abort_conditions"] = serde_json::json!([]);
            json["recovery_steps"] = serde_json::json!([]);
            json["post_action_safe_state_markers"] = serde_json::json!([]);
        });

        // Act
        let report = validate_safety_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert_error_contains(
            &report,
            "explicit DEVICE_URL or trusted raw origin-only target lock",
        );
    }

    #[test]
    fn safety_allow_rejects_missing_procedure_scope_fields() {
        // Arrange
        let cases = [
            ("recovery_steps", "recovery_steps must not be empty"),
            (
                "post_action_safe_state_markers",
                "post_action_safe_state_markers must not be empty",
            ),
            ("redaction_reviewer", "redaction_reviewer must not be empty"),
            ("checklist_rows", "checklist_rows must not be empty"),
        ];

        for (field, expected_error) in cases {
            let (manifest, package_manifest) = manifest_with_change(|json| match field {
                "recovery_steps" => json["recovery_steps"] = serde_json::json!([]),
                "post_action_safe_state_markers" => {
                    json["post_action_safe_state_markers"] = serde_json::json!([])
                }
                "redaction_reviewer" => json["redaction_reviewer"] = serde_json::json!(""),
                "checklist_rows" => json["checklist_rows"] = serde_json::json!([]),
                _ => unreachable!("test case field should be handled"),
            });

            // Act
            let report = validate_safety_allow_manifest(&manifest, &package_manifest);

            // Assert
            assert_error_contains(&report, expected_error);
        }
    }

    #[test]
    fn safety_allow_allows_surface_claim_tiers_with_matching_evidence_class() {
        // Arrange
        let cases = [
            (
                "safe-baseline",
                "safe-baseline",
                "hardware-smoke",
                "bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest package.json --evidence-dir evidence/safe-baseline",
                serde_json::json!({ "safe_state_marker": "safe_state: mining=disabled" }),
            ),
            (
                "safe-baseline",
                "read-only-observation",
                "hardware-smoke",
                "bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest package.json --evidence-dir evidence/safe-baseline",
                serde_json::json!({ "safe_state_marker": "safe_state: mining=disabled" }),
            ),
            (
                "live-api-websocket-telemetry",
                "api-websocket-projection",
                "hardware-smoke",
                "scripts/phase14-live-telemetry.sh --manifest allow.json --out-dir evidence/live-api --device-url http://[redacted]",
                serde_json::json!({
                    "device_url_source": "explicit DEVICE_URL",
                    "network_scan": "disabled",
                    "route_path": "/api/system/info",
                    "duration_ms": 10000,
                    "max_frames": 5
                }),
            ),
            (
                "failure-paths",
                "safe-unavailable",
                "hardware-smoke",
                "scripts/phase20-failure-paths.sh --manifest allow.json --out-dir evidence/failure-paths",
                serde_json::json!({ "reason": "fault route unavailable" }),
            ),
            (
                "failure-paths",
                "unsupported-pending",
                "deferred",
                "scripts/phase20-failure-paths.sh --manifest allow.json --out-dir evidence/failure-paths",
                serde_json::json!({ "reason": "no production-safe bounded fault-stimulus route" }),
            ),
            (
                "parity-redaction",
                "parity-redaction",
                "workflow",
                "rg -n -i secret docs/parity/evidence/phase-20-active-safety-hardware-telemetry-evidence",
                serde_json::json!({ "scan": "secret patterns" }),
            ),
        ];

        for (surface, claim_tier, evidence_class, allowed_command, allowed_inputs) in cases {
            let (manifest, package_manifest) = manifest_with_change(|json| {
                json["surface"] = serde_json::json!(surface);
                json["claim_tier"] = serde_json::json!(claim_tier);
                json["evidence_class"] = serde_json::json!(evidence_class);
                json["allowed_command"] = serde_json::json!(allowed_command);
                json["allowed_inputs"] = allowed_inputs;
                json["abort_conditions"] = serde_json::json!([]);
                json["recovery_steps"] = serde_json::json!([]);
                json["post_action_safe_state_markers"] = serde_json::json!([]);
            });

            // Act
            let report = validate_safety_allow_manifest(&manifest, &package_manifest);

            // Assert
            assert!(
                report.passed(),
                "{surface}/{claim_tier} should pass: {report:#?}"
            );
        }
    }

    #[test]
    fn safety_allow_renders_passed_cli_contract_for_valid_manifest() {
        // Arrange
        let cli_command =
            "bazel run //tools/parity:report -- safety-allow --manifest <valid-test-manifest>";
        let (manifest, package_manifest) = manifest_with_change(|_json| {});

        // Act
        let report = validate_safety_allow_manifest(&manifest, &package_manifest);
        let output = render_safety_allow_report(&manifest, &report);

        // Assert
        assert!(cli_command.contains("safety-allow --manifest"));
        assert!(report.passed(), "{output}");
        assert!(output.contains("safety_allow_status: passed"));
        assert!(output.contains("surface: voltage-control"));
        assert!(output.contains("claim_tier: bounded-actuation"));
        assert!(output.contains("evidence_class: hardware-regression"));
        assert!(output.contains("checklist_rows: PWR-003,PWR-005"));
    }

    #[test]
    fn safety_allow_filters_reject_mismatched_surface_and_command() {
        // Arrange
        let (manifest, package_manifest) = manifest_with_change(|_json| {});
        let documents = SafetyAllowDocuments {
            manifest,
            package_manifest,
        };
        let filters = SafetyAllowFilters {
            maybe_surface: Some("thermal-fan".to_owned()),
            maybe_allowed_command: Some("scripts/other.sh --manifest allow.json".to_owned()),
        };

        // Act
        let report = validate_safety_allow_documents(&documents, &filters);

        // Assert
        assert_error_contains(&report, "surface filter mismatch");
        assert_error_contains(&report, "allowed command filter mismatch");
    }

    #[test]
    fn safety_allow_loads_checked_in_style_json_from_temporary_directory() {
        // Arrange
        let fixture = SafetyAllowFixture::new(valid_manifest_json_string(
            "package/bitaxe-ultra205-package.json",
        ));

        // Act
        let documents = load_safety_allow_documents(&fixture.workspace_dir, &fixture.manifest_path)
            .expect("fixture documents should load");
        let report =
            validate_safety_allow_manifest(&documents.manifest, &documents.package_manifest);

        // Assert
        assert!(report.passed(), "{report:#?}");
        assert_eq!(documents.manifest.board, "205");
        std::fs::remove_dir_all(fixture.workspace_dir.as_std_path()).expect("fixture cleanup");
    }

    fn assert_error_contains(report: &SafetyAllowReport, expected: &str) {
        assert!(
            report
                .validation_errors
                .iter()
                .any(|error| error.contains(expected)),
            "expected validation error containing {expected:?}, got {report:#?}"
        );
    }

    fn manifest_with_change(change: impl FnOnce(&mut Value)) -> (SafetyAllowManifest, Value) {
        let package_manifest =
            serde_json::from_str(package_manifest_json_string()).expect("package json");
        let mut manifest_json: Value = serde_json::from_str(&valid_manifest_json_string(
            "bazel-bin/firmware/bitaxe/bitaxe-ultra205-package.json",
        ))
        .expect("manifest json");
        change(&mut manifest_json);
        let manifest = serde_json::from_value(manifest_json).expect("valid manifest shape");

        (manifest, package_manifest)
    }

    fn package_manifest_json_string() -> &'static str {
        r#"{
  "source_commit": "source-abc",
  "reference_commit": "reference-def"
}"#
    }

    fn valid_manifest_json_string(package_manifest: &str) -> String {
        r#"{
            "board": "205",
            "port": "/dev/cu.usbmodem1101",
            "detector_command": "just detect-ultra205",
            "detector_port": "/dev/cu.usbmodem1101",
            "board_info_command": "espflash board-info --chip esp32s3 --port /dev/cu.usbmodem1101 --non-interactive",
            "board_info_status": "passed",
            "package_manifest": "__PACKAGE_MANIFEST__",
            "source_commit": "source-abc",
            "reference_commit": "reference-def",
            "surface": "voltage-control",
            "claim_tier": "bounded-actuation",
            "evidence_class": "hardware-regression",
            "allowed_command": "scripts/phase14-power-voltage.sh --manifest allow.json --surface voltage-control --out-dir evidence/voltage-control",
            "allowed_inputs": {
                "setpoint_mv": [1200]
            },
            "abort_conditions": [
                "detector_mismatch",
                "board_info_failure",
                "missing_safe_state_marker"
            ],
            "recovery_steps": [
                "just flash board=205 port=/dev/cu.usbmodem1101"
            ],
            "post_action_safe_state_markers": [
                "safe_state: mining=disabled",
                "hardware_control=disabled"
            ],
            "evidence_dir": "docs/parity/evidence/phase-14-safety-hardware-evidence-completion/voltage-control",
            "redaction_reviewer": "phase-14-reviewer",
            "checklist_rows": ["PWR-003", "PWR-005"]
        }"#
        .replace("__PACKAGE_MANIFEST__", package_manifest)
    }

    struct SafetyAllowFixture {
        workspace_dir: Utf8PathBuf,
        manifest_path: Utf8PathBuf,
    }

    impl SafetyAllowFixture {
        fn new(manifest_json: String) -> Self {
            let workspace_dir = unique_temp_dir();
            let package_dir = workspace_dir.join("package");
            std::fs::create_dir_all(package_dir.as_std_path()).expect("package dir");
            std::fs::write(
                package_dir
                    .join("bitaxe-ultra205-package.json")
                    .as_std_path(),
                package_manifest_json_string(),
            )
            .expect("package manifest");

            let manifest_path = workspace_dir.join("allow.json");
            std::fs::write(manifest_path.as_std_path(), manifest_json).expect("allow manifest");

            Self {
                workspace_dir,
                manifest_path,
            }
        }
    }

    fn unique_temp_dir() -> Utf8PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock after epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("bitaxe-safety-allow-{nanos}"));
        let utf8_path = Utf8PathBuf::from_path_buf(path).expect("temp path should be UTF-8");
        std::fs::create_dir_all(utf8_path.as_std_path()).expect("temp dir");
        utf8_path
    }
}
