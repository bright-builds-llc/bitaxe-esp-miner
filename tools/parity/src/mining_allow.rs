use anyhow::{Context, Result};
use bitaxe_automation_contracts::{AutomationCommand, WorkflowIdentity};
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
    pub(crate) workflow: WorkflowIdentity,
    pub(crate) constraints: Value,
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
    pub(crate) maybe_workflow: Option<String>,
    pub(crate) maybe_request_sha256: Option<String>,
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

    validate_required_schema_fields(&mut validation_errors, manifest);
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

fn validate_required_schema_fields(errors: &mut Vec<String>, manifest: &MiningAllowManifest) {
    let required_fields = [
        ("board", manifest.board.as_str()),
        ("port", manifest.port.as_str()),
        ("detector_command", manifest.detector_command.as_str()),
        ("detector_port", manifest.detector_port.as_str()),
        ("board_info_command", manifest.board_info_command.as_str()),
        ("board_info_status", manifest.board_info_status.as_str()),
        ("package_manifest", manifest.package_manifest.as_str()),
        ("source_commit", manifest.source_commit.as_str()),
        ("reference_commit", manifest.reference_commit.as_str()),
        ("surface", manifest.surface.as_str()),
        ("claim_tier", manifest.claim_tier.as_str()),
        ("evidence_class", manifest.evidence_class.as_str()),
        (
            "workflow.schema_version",
            manifest.workflow.schema_version.as_str(),
        ),
        (
            "workflow.request_sha256",
            manifest.workflow.request_sha256.as_str(),
        ),
        ("evidence_dir", manifest.evidence_dir.as_str()),
        ("redaction_reviewer", manifest.redaction_reviewer.as_str()),
    ];

    for (field_name, field_value) in required_fields {
        if field_value.trim().is_empty() {
            errors.push(format!("{field_name} must not be empty"));
        }
    }

    if !manifest.constraints.is_object() {
        errors.push("constraints must be a JSON object".to_owned());
    }
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

    if evidence_class_matches_claim(manifest) {
        return;
    }

    let expected_evidence_class = expected_evidence_class(&manifest.claim_tier);
    if manifest.claim_tier == "safe-prerequisite-blocked" {
        errors.push(
            "claim_tier `safe-prerequisite-blocked` requires evidence_class `workflow` or `hardware-smoke`"
                .to_owned(),
        );
    } else {
        errors.push(format!(
            "claim_tier `{}` requires evidence_class `{expected_evidence_class}`",
            manifest.claim_tier
        ));
    }
}

fn validate_required_procedure_scope(errors: &mut Vec<String>, manifest: &MiningAllowManifest) {
    validate_workflow_identity(errors, manifest);

    if manifest.constraints.is_null() {
        errors.push("constraints must not be null".to_owned());
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
        .constraints
        .get("pool_config")
        .and_then(Value::as_str);
    if !matches!(
        maybe_pool_config,
        Some("disposable-or-non-secret" | "local-owner-supplied")
    ) {
        errors.push(
            "live-pool-smoke requires constraints.pool_config to equal disposable-or-non-secret or local-owner-supplied"
                .to_owned(),
        );
    }

    let maybe_device_url = manifest
        .constraints
        .get("device_url")
        .and_then(Value::as_str);
    if maybe_device_url != Some("explicit") {
        errors.push("live-pool-smoke requires constraints.device_url to equal explicit".to_owned());
    }
}

fn validate_bounded_soak_scope(errors: &mut Vec<String>, manifest: &MiningAllowManifest) {
    if manifest.claim_tier != "bounded-soak" {
        return;
    }

    let maybe_duration_seconds = manifest
        .constraints
        .get("duration_seconds")
        .and_then(Value::as_i64);
    let Some(duration_seconds) = maybe_duration_seconds else {
        errors.push(
            "bounded-soak requires constraints.duration_seconds between 60 and 600".to_owned(),
        );
        return;
    };

    if !(60..=600).contains(&duration_seconds) {
        errors.push(
            "bounded-soak requires constraints.duration_seconds between 60 and 600".to_owned(),
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

    let Some(expected_workflow) = &filters.maybe_workflow else {
        errors.push("workflow filter is required".to_owned());
        return;
    };

    let actual_workflow = serde_json::to_value(manifest.workflow.command)
        .ok()
        .and_then(|value| value.as_str().map(str::to_owned))
        .unwrap_or_default();
    if &actual_workflow != expected_workflow {
        errors.push(format!(
            "workflow filter mismatch: manifest `{actual_workflow}` != `{expected_workflow}`"
        ));
    }

    let Some(expected_request_sha256) = &filters.maybe_request_sha256 else {
        errors.push("request digest filter is required".to_owned());
        return;
    };
    if &manifest.workflow.request_sha256 != expected_request_sha256 {
        errors.push("request digest filter mismatch".to_owned());
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

fn validate_workflow_identity(errors: &mut Vec<String>, manifest: &MiningAllowManifest) {
    if manifest.workflow.schema_version != "bitaxe-workflow-identity-v1" {
        errors.push("workflow schema must be bitaxe-workflow-identity-v1".to_owned());
    }
    if manifest.workflow.command != AutomationCommand::VerifyMining {
        errors.push("workflow command must be verify-mining".to_owned());
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

fn evidence_class_matches_claim(manifest: &MiningAllowManifest) -> bool {
    manifest.evidence_class == expected_evidence_class(&manifest.claim_tier)
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
mod tests;
