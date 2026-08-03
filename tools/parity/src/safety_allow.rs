use anyhow::{Context, Result};
use bitaxe_automation_contracts::WorkflowIdentity;
use camino::{Utf8Path, Utf8PathBuf};
use serde::{Deserialize, Deserializer};
use serde_json::Value;

mod policy;

use policy::{
    validate_active_claim_scope, validate_detector_gate, validate_failure_paths_scope,
    validate_filters, validate_live_api_websocket_scope, validate_package_identity,
    validate_required_procedure_scope, validate_surface_and_claim,
};

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
    pub(crate) workflow: WorkflowIdentity,
    pub(crate) constraints: Value,
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
    pub(crate) maybe_workflow: Option<String>,
    pub(crate) maybe_request_sha256: Option<String>,
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
mod tests;
