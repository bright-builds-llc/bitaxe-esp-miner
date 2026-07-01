use anyhow::{Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Deserializer};
use serde_json::Value;

const DETECTOR_COMMAND: &str = "just detect-ultra205";
const REQUIRED_ABORT_CONDITIONS: &[&str] = &[
    "detector_mismatch",
    "board_info_failure",
    "missing_trusted_wrapper_markers",
    "redaction_uncertainty",
    "unsafe_temperature_or_power",
    "watchdog_unresponsive",
];
const REQUIRED_SAFE_STATE_MARKERS: &[&str] = &[
    "safe_state: mining=disabled",
    "hardware_control=disabled",
    "work_submission=disabled",
];
const ALLOWED_SURFACES: &[&str] = &[
    "bm1366-chip-detect",
    "bm1366-work-result",
    "mining-smoke",
    "bounded-soak",
    "parity-redaction",
];
const ALLOWED_CLAIM_TIERS: &[&str] = &[
    "diagnostic-chip-detect",
    "diagnostic-work-result",
    "controlled-no-share",
    "live-pool-smoke",
    "bounded-soak",
    "unsupported-pending",
    "parity-redaction",
];
const PROHIBITED_COMMAND_TOKENS: &[&str] = &[
    "erase-flash",
    "erase_flash",
    "write-flash",
    "write_flash",
    "--erase-all",
    "erase_region",
    "raw-bm1366",
    "voltage-control",
    "fan-control",
    "stratum",
];

#[derive(Debug, Deserialize)]
pub(crate) struct MiningAllowManifest {
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
    #[serde(deserialize_with = "deserialize_utf8_path_buf_vec")]
    pub(crate) prerequisite_artifacts: Vec<Utf8PathBuf>,
    #[serde(deserialize_with = "deserialize_utf8_path_buf")]
    pub(crate) evidence_dir: Utf8PathBuf,
    pub(crate) redaction_reviewer: String,
    pub(crate) checklist_rows: Vec<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct MiningAllowReport {
    pub(crate) validation_errors: Vec<String>,
}

impl MiningAllowReport {
    pub(crate) fn passed(&self) -> bool {
        self.validation_errors.is_empty()
    }
}

#[derive(Debug)]
pub(crate) struct MiningAllowDocuments {
    pub(crate) manifest: MiningAllowManifest,
    pub(crate) package_manifest: Value,
}

#[derive(Debug)]
pub(crate) struct MiningAllowFilters {
    pub(crate) maybe_surface: Option<String>,
    pub(crate) maybe_allowed_command: Option<String>,
}

pub(crate) fn load_mining_allow_documents(
    workspace_dir: &Utf8Path,
    manifest_path: &Utf8Path,
) -> Result<MiningAllowDocuments> {
    let manifest_json = std::fs::read_to_string(manifest_path.as_std_path())
        .with_context(|| format!("failed to read mining allow manifest {manifest_path}"))?;
    let manifest: MiningAllowManifest = serde_json::from_str(&manifest_json)
        .with_context(|| format!("mining allow manifest {manifest_path} is not valid JSON"))?;
    let package_manifest_path = resolve_workspace_path(workspace_dir, &manifest.package_manifest);
    let package_manifest_json = std::fs::read_to_string(package_manifest_path.as_std_path())
        .with_context(|| format!("failed to read package manifest {package_manifest_path}"))?;
    let package_manifest: Value = serde_json::from_str(&package_manifest_json)
        .with_context(|| format!("package manifest {package_manifest_path} is not valid JSON"))?;

    Ok(MiningAllowDocuments {
        manifest,
        package_manifest,
    })
}

pub(crate) fn validate_mining_allow_documents(
    documents: &MiningAllowDocuments,
    filters: &MiningAllowFilters,
) -> MiningAllowReport {
    let mut report =
        validate_mining_allow_manifest(&documents.manifest, &documents.package_manifest);
    validate_filters(&mut report.validation_errors, &documents.manifest, filters);
    report
}

pub(crate) fn validate_mining_allow_manifest(
    manifest: &MiningAllowManifest,
    package_manifest: &Value,
) -> MiningAllowReport {
    let mut validation_errors = Vec::new();

    validate_detector_gate(&mut validation_errors, manifest);
    validate_package_identity(&mut validation_errors, manifest, package_manifest);
    validate_surface_and_claim(&mut validation_errors, manifest);
    validate_required_procedure_scope(&mut validation_errors, manifest);
    validate_required_stop_contract(&mut validation_errors, manifest);
    validate_live_pool_smoke_scope(&mut validation_errors, manifest);
    validate_bounded_soak_scope(&mut validation_errors, manifest);

    MiningAllowReport { validation_errors }
}

pub(crate) fn render_mining_allow_report(
    manifest: &MiningAllowManifest,
    report: &MiningAllowReport,
) -> String {
    if report.passed() {
        return format!(
            "mining_allow_status: passed\nsurface: {}\nclaim_tier: {}\nevidence_class: {}\nchecklist_rows: {}\n",
            manifest.surface,
            manifest.claim_tier,
            manifest.evidence_class,
            manifest.checklist_rows.join(",")
        );
    }

    let mut output = String::from("mining_allow_status: failed\nvalidation_errors:\n");
    for error in &report.validation_errors {
        output.push_str("- ");
        output.push_str(error);
        output.push('\n');
    }
    output
}

fn resolve_workspace_path(workspace_dir: &Utf8Path, path: &Utf8Path) -> Utf8PathBuf {
    if path.is_absolute() {
        return path.to_owned();
    }

    workspace_dir.join(path)
}

fn validate_detector_gate(errors: &mut Vec<String>, manifest: &MiningAllowManifest) {
    if manifest.board != "205" {
        errors.push("board must be 205".to_owned());
    }

    if manifest.detector_command != DETECTOR_COMMAND {
        errors.push("detector command must be just detect-ultra205".to_owned());
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
    manifest: &MiningAllowManifest,
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

fn validate_surface_and_claim(errors: &mut Vec<String>, manifest: &MiningAllowManifest) {
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

fn validate_required_procedure_scope(errors: &mut Vec<String>, manifest: &MiningAllowManifest) {
    if manifest.allowed_command.trim().is_empty() {
        errors.push("allowed_command must not be empty".to_owned());
    } else {
        validate_allowed_command_scope(errors, manifest);
    }

    if manifest.allowed_inputs.is_null() {
        errors.push("allowed_inputs must not be null".to_owned());
    }

    if manifest.prerequisite_artifacts.is_empty() {
        errors.push("prerequisite_artifacts must not be empty".to_owned());
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

fn validate_required_stop_contract(errors: &mut Vec<String>, manifest: &MiningAllowManifest) {
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

fn validate_live_pool_smoke_scope(errors: &mut Vec<String>, manifest: &MiningAllowManifest) {
    if manifest.claim_tier != "live-pool-smoke" {
        return;
    }

    let maybe_pool_config = manifest
        .allowed_inputs
        .get("pool_config")
        .and_then(Value::as_str);
    if maybe_pool_config != Some("disposable-or-non-secret") {
        errors.push(
            "live-pool-smoke requires allowed_inputs.pool_config to equal disposable-or-non-secret"
                .to_owned(),
        );
    }

    let maybe_device_url = manifest
        .allowed_inputs
        .get("device_url")
        .and_then(Value::as_str);
    if maybe_device_url != Some("explicit") {
        errors.push(
            "live-pool-smoke requires allowed_inputs.device_url to equal explicit".to_owned(),
        );
    }

    if !manifest
        .allowed_command
        .split_whitespace()
        .any(|token| token == "--device-url")
    {
        errors.push("live-pool-smoke requires allowed_command to include --device-url".to_owned());
    }
}

fn validate_bounded_soak_scope(errors: &mut Vec<String>, manifest: &MiningAllowManifest) {
    if manifest.claim_tier != "bounded-soak" {
        return;
    }

    let maybe_duration_seconds = manifest
        .allowed_inputs
        .get("duration_seconds")
        .and_then(Value::as_i64);
    let Some(duration_seconds) = maybe_duration_seconds else {
        errors.push(
            "bounded-soak requires allowed_inputs.duration_seconds between 60 and 600".to_owned(),
        );
        return;
    };

    if !(60..=600).contains(&duration_seconds) {
        errors.push(
            "bounded-soak requires allowed_inputs.duration_seconds between 60 and 600".to_owned(),
        );
    }
}

fn validate_filters(
    errors: &mut Vec<String>,
    manifest: &MiningAllowManifest,
    filters: &MiningAllowFilters,
) {
    if let Some(expected_surface) = &filters.maybe_surface {
        if &manifest.surface != expected_surface {
            errors.push(format!(
                "surface filter mismatch: manifest `{}` != `{expected_surface}`",
                manifest.surface
            ));
        }
    }

    let Some(expected_allowed_command) = &filters.maybe_allowed_command else {
        errors.push("allowed command filter is required".to_owned());
        return;
    };

    if &manifest.allowed_command != expected_allowed_command {
        errors.push(format!(
            "allowed command filter mismatch: manifest `{}` != `{expected_allowed_command}`",
            manifest.allowed_command
        ));
    }
}

fn allowed_claim_tiers_for_surface(surface: &str) -> &'static [&'static str] {
    match surface {
        "bm1366-chip-detect" => &["diagnostic-chip-detect"],
        "bm1366-work-result" => &["diagnostic-work-result"],
        "mining-smoke" => &["controlled-no-share", "live-pool-smoke"],
        "bounded-soak" => &["bounded-soak", "unsupported-pending"],
        "parity-redaction" => &["parity-redaction"],
        _ => &[],
    }
}

fn validate_allowed_command_scope(errors: &mut Vec<String>, manifest: &MiningAllowManifest) {
    let tokens: Vec<&str> = manifest.allowed_command.split_whitespace().collect();

    for prohibited_token in PROHIBITED_COMMAND_TOKENS {
        if tokens.iter().any(|token| token == prohibited_token) {
            errors.push(format!(
                "allowed_command contains prohibited token `{prohibited_token}`"
            ));
        }
    }

    if is_expected_phase15_command(manifest, &tokens) {
        return;
    }

    errors.push(
        "allowed_command must route through an approved Phase 15 wrapper for its surface"
            .to_owned(),
    );
}

fn is_expected_phase15_command(manifest: &MiningAllowManifest, tokens: &[&str]) -> bool {
    match manifest.surface.as_str() {
        "bm1366-chip-detect" | "bm1366-work-result" => {
            starts_with_tokens(
                tokens,
                &["bazel", "run", "//tools/flash:flash", "--", "flash-monitor"],
            ) && option_equals(tokens, "--board", "205")
                && option_equals(tokens, "--port", &manifest.port)
                && has_option_with_value(tokens, "--manifest")
                && has_option_with_value(tokens, "--evidence-dir")
        }
        "mining-smoke" | "bounded-soak" => {
            starts_with_tokens(tokens, &["scripts/phase15-controlled-mining.sh"])
                && option_equals(tokens, "--surface", &manifest.surface)
                && has_option_with_value(tokens, "--manifest")
                && has_option_with_value(tokens, "--out-dir")
                && has_option_with_value(tokens, "--chip-detect-summary")
                && has_option_with_value(tokens, "--work-result-summary")
        }
        "parity-redaction" => {
            starts_with_tokens(tokens, &["rg", "-n", "-i"])
                && tokens.iter().any(|token| {
                    token.starts_with(
                        "docs/parity/evidence/phase-15-bm1366-mining-evidence-completion",
                    )
                })
        }
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

fn expected_evidence_class(claim_tier: &str) -> &'static str {
    match claim_tier {
        "diagnostic-chip-detect"
        | "diagnostic-work-result"
        | "controlled-no-share"
        | "live-pool-smoke" => "hardware-smoke",
        "bounded-soak" => "soak",
        "unsupported-pending" | "parity-redaction" => "workflow",
        _ => "unsupported",
    }
}

fn deserialize_utf8_path_buf<'de, D>(deserializer: D) -> Result<Utf8PathBuf, D::Error>
where
    D: Deserializer<'de>,
{
    let path = String::deserialize(deserializer)?;
    Ok(Utf8PathBuf::from(path))
}

fn deserialize_utf8_path_buf_vec<'de, D>(deserializer: D) -> Result<Vec<Utf8PathBuf>, D::Error>
where
    D: Deserializer<'de>,
{
    let paths = Vec::<String>::deserialize(deserializer)?;
    Ok(paths.into_iter().map(Utf8PathBuf::from).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mining_allow_rejects_non_205_board() {
        // Arrange
        let (manifest, package_manifest) = manifest_with_change(|json| {
            json["board"] = serde_json::json!("601");
        });

        // Act
        let report = validate_mining_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert_error_contains(&report, "board must be 205");
    }

    #[test]
    fn mining_allow_rejects_wrong_detector_command() {
        // Arrange
        let (manifest, package_manifest) = manifest_with_change(|json| {
            json["detector_command"] = serde_json::json!("just detect-any-board");
        });

        // Act
        let report = validate_mining_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert_error_contains(&report, "detector command must be just detect-ultra205");
    }

    #[test]
    fn mining_allow_rejects_detector_port_mismatch() {
        // Arrange
        let (manifest, package_manifest) = manifest_with_change(|json| {
            json["detector_port"] = serde_json::json!("/dev/cu.usbmodem9999");
        });

        // Act
        let report = validate_mining_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert_error_contains(&report, "detector port mismatch");
    }

    #[test]
    fn mining_allow_rejects_failed_board_info() {
        // Arrange
        let (manifest, package_manifest) = manifest_with_change(|json| {
            json["board_info_status"] = serde_json::json!("failed");
        });

        // Act
        let report = validate_mining_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert_error_contains(&report, "board-info must pass");
    }

    #[test]
    fn mining_allow_rejects_package_identity_mismatch() {
        // Arrange
        let (manifest, mut package_manifest) = manifest_with_change(|_json| {});
        package_manifest["source_commit"] = serde_json::json!("stale-source");

        // Act
        let report = validate_mining_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert_error_contains(&report, "package identity mismatch");
    }

    #[test]
    fn mining_allow_accepts_only_phase15_surfaces() {
        // Arrange
        let allowed_surface_claims = [
            (
                "bm1366-chip-detect",
                "diagnostic-chip-detect",
                "hardware-smoke",
                "bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest package.json --evidence-dir evidence/chip-detect",
            ),
            (
                "bm1366-work-result",
                "diagnostic-work-result",
                "hardware-smoke",
                "bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest package.json --evidence-dir evidence/work-result",
            ),
            (
                "mining-smoke",
                "controlled-no-share",
                "hardware-smoke",
                "scripts/phase15-controlled-mining.sh --manifest allow.json --surface mining-smoke --out-dir evidence/mining-smoke --chip-detect-summary chip.md --work-result-summary work.md",
            ),
            (
                "bounded-soak",
                "unsupported-pending",
                "workflow",
                "scripts/phase15-controlled-mining.sh --manifest allow.json --surface bounded-soak --duration-seconds 120 --out-dir evidence/bounded-soak --chip-detect-summary chip.md --work-result-summary work.md",
            ),
            (
                "parity-redaction",
                "parity-redaction",
                "workflow",
                "rg -n -i secret docs/parity/evidence/phase-15-bm1366-mining-evidence-completion",
            ),
        ];

        for (surface, claim_tier, evidence_class, allowed_command) in allowed_surface_claims {
            let (manifest, package_manifest) = manifest_with_change(|json| {
                json["surface"] = serde_json::json!(surface);
                json["claim_tier"] = serde_json::json!(claim_tier);
                json["evidence_class"] = serde_json::json!(evidence_class);
                json["allowed_command"] = serde_json::json!(allowed_command);
            });

            // Act
            let report = validate_mining_allow_manifest(&manifest, &package_manifest);

            // Assert
            assert!(report.passed(), "{surface} should pass: {report:#?}");
        }

        let (manifest, package_manifest) = manifest_with_change(|json| {
            json["surface"] = serde_json::json!("voltage-control");
        });

        // Act
        let report = validate_mining_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert_error_contains(&report, "surface `voltage-control` is not allowed");
    }

    #[test]
    fn mining_allow_rejects_surface_claim_tier_mismatch() {
        // Arrange
        let (manifest, package_manifest) = manifest_with_change(|json| {
            json["surface"] = serde_json::json!("bm1366-chip-detect");
            json["claim_tier"] = serde_json::json!("controlled-no-share");
            json["allowed_command"] = serde_json::json!(
                "bazel run //tools/flash:flash -- flash-monitor --board 205 --port /dev/cu.usbmodem1101 --manifest package.json --evidence-dir evidence/chip-detect"
            );
        });

        // Act
        let report = validate_mining_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert_error_contains(&report, "does not allow claim_tier `controlled-no-share`");
    }

    #[test]
    fn mining_allow_rejects_unapproved_or_unsafe_allowed_command() {
        // Arrange
        let (manifest, package_manifest) = manifest_with_change(|json| {
            json["allowed_command"] =
                serde_json::json!("espflash erase-flash --port /dev/cu.usbmodem1101");
        });

        // Act
        let report = validate_mining_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert_error_contains(&report, "prohibited token `erase-flash`");
        assert_error_contains(&report, "approved Phase 15 wrapper");
    }

    #[test]
    fn mining_allow_documents_require_allowed_command_filter() {
        // Arrange
        let (manifest, package_manifest) = manifest_with_change(|_json| {});
        let documents = MiningAllowDocuments {
            manifest,
            package_manifest,
        };
        let filters = MiningAllowFilters {
            maybe_surface: Some("mining-smoke".to_owned()),
            maybe_allowed_command: None,
        };

        // Act
        let report = validate_mining_allow_documents(&documents, &filters);

        // Assert
        assert_error_contains(&report, "allowed command filter is required");
    }

    #[test]
    fn mining_allow_live_pool_smoke_requires_disposable_inputs() {
        // Arrange
        let (manifest, package_manifest) = manifest_with_change(|json| {
            json["claim_tier"] = serde_json::json!("live-pool-smoke");
            json["allowed_inputs"]["pool_config"] = serde_json::json!("private");
            json["allowed_inputs"]["device_url"] = serde_json::json!("inferred");
        });

        // Act
        let report = validate_mining_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert_error_contains(&report, "pool_config");
        assert_error_contains(&report, "device_url");
        assert_error_contains(&report, "--device-url");
    }

    #[test]
    fn mining_allow_bounded_soak_requires_duration_and_safe_stop_contract() {
        // Arrange
        let (manifest, package_manifest) = manifest_with_change(|json| {
            json["claim_tier"] = serde_json::json!("bounded-soak");
            json["evidence_class"] = serde_json::json!("soak");
            json["allowed_inputs"]["duration_seconds"] = serde_json::json!(601);
            json["abort_conditions"] = serde_json::json!(["detector_mismatch"]);
            json["recovery_steps"] = serde_json::json!([]);
            json["post_action_safe_state_markers"] =
                serde_json::json!(["safe_state: mining=disabled"]);
        });

        // Act
        let report = validate_mining_allow_manifest(&manifest, &package_manifest);

        // Assert
        assert_error_contains(&report, "duration_seconds");
        assert_error_contains(
            &report,
            "abort_conditions must contain `board_info_failure`",
        );
        assert_error_contains(&report, "recovery_steps must not be empty");
        assert_error_contains(
            &report,
            "post_action_safe_state_markers must contain `hardware_control=disabled`",
        );
    }

    fn assert_error_contains(report: &MiningAllowReport, expected: &str) {
        assert!(
            report
                .validation_errors
                .iter()
                .any(|error| error.contains(expected)),
            "expected validation error containing {expected:?}, got {report:#?}"
        );
    }

    fn manifest_with_change(change: impl FnOnce(&mut Value)) -> (MiningAllowManifest, Value) {
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
            "surface": "mining-smoke",
            "claim_tier": "controlled-no-share",
            "evidence_class": "hardware-smoke",
            "allowed_command": "scripts/phase15-controlled-mining.sh --manifest allow.json --surface mining-smoke --out-dir evidence/mining-smoke --chip-detect-summary chip.md --work-result-summary work.md",
            "allowed_inputs": {
                "pool_config": "disposable-or-non-secret",
                "device_url": "explicit",
                "duration_seconds": 60
            },
            "abort_conditions": [
                "detector_mismatch",
                "board_info_failure",
                "missing_trusted_wrapper_markers",
                "redaction_uncertainty",
                "unsafe_temperature_or_power",
                "watchdog_unresponsive"
            ],
            "recovery_steps": [
                "just flash board=205 port=/dev/cu.usbmodem1101"
            ],
            "post_action_safe_state_markers": [
                "safe_state: mining=disabled",
                "hardware_control=disabled",
                "work_submission=disabled"
            ],
            "prerequisite_artifacts": [
                "docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/detector.json"
            ],
            "evidence_dir": "docs/parity/evidence/phase-15-bm1366-mining-evidence-completion/mining-smoke",
            "redaction_reviewer": "phase-15-reviewer",
            "checklist_rows": ["STR-007", "STR-008"]
        }"#
        .replace("__PACKAGE_MANIFEST__", package_manifest)
    }
}
