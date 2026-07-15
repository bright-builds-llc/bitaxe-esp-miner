use std::collections::BTreeMap;
use std::env;
use std::io::{self, Write};
use std::process::Command as ProcessCommand;

use anyhow::{bail, Context, Result};
use bitaxe_api::phase33_evidence::{
    classify_phase33_baseline, classify_phase33_delivery, classify_phase33_post_restart,
};
use camino::{Utf8Path, Utf8PathBuf};
use clap::{Parser, Subcommand, ValueEnum};
use operator_evidence::{
    complete_operator_evidence, consolidate_phase28_evidence, load_operator_evidence_documents,
    render_operator_evidence_report, validate_operator_evidence_documents, OperatorEvidenceFilters,
    OperatorEvidenceProfile, WorkflowStatus,
};
use release_evidence::{
    parse_flash_evidence_json, parse_release_evidence_manifest_json,
    render_release_evidence_report, validate_release_evidence, ReleaseEvidenceDocuments,
};
use release_gate::{
    render_release_gate_report, validate_release_gate, ReleaseGateDocuments,
    DEFAULT_CARGO_ABOUT_PATH, DEFAULT_LICENSE_INVENTORY_PATH, DEFAULT_PROVENANCE_PATH,
};
use serde::Serialize;
use std::io::ErrorKind;

const BAZEL_REFERENCE_GUARD_TARGET: &str = "//scripts:verify_reference_clean";
const DEFAULT_REFERENCE_GUARD_PATH: &str = "scripts/verify-reference-clean.sh";
const DEFAULT_REFERENCE_DIR: &str = "reference/esp-miner";
const DEFAULT_OPENAPI_PATH: &str = "reference/esp-miner/main/http_server/openapi.yaml";
const DEFAULT_API_COMPARE_MANIFEST: &str = "tools/parity/fixtures/api/phase05-required-routes.json";
const DEFAULT_AXEOS_ROUTE_USAGE: &str = "tools/parity/fixtures/api/axeos-route-usage.json";
const DEFAULT_PHASE30_PROMOTION_ARTIFACT_PATH: &str =
    "docs/parity/evidence/phase-30-live-share-outcome-and-verified-promotion/conclusion.md";

mod api_compare;
mod claim_ladder;
mod mining_allow;
mod operator_evidence;
#[cfg(test)]
mod phase32_source_guard;
#[cfg(test)]
mod phase33_source_guard;
#[cfg(test)]
mod phase34_source_guard;
mod release_evidence;
mod release_gate;
mod safety_allow;
mod v12_admission;

#[derive(Debug, Parser)]
#[command(name = "bitaxe-parity")]
#[command(about = "Report Bitaxe parity checklist status and evidence gaps.")]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Debug, Subcommand)]
enum CliCommand {
    Report(ReportArgs),
    ApiCompare(ApiCompareArgs),
    ReleaseGate(ReleaseGateArgs),
    ReleaseEvidence(ReleaseEvidenceArgs),
    SafetyAllow(SafetyAllowArgs),
    MiningAllow(MiningAllowArgs),
    OperatorEvidence(OperatorEvidenceArgs),
    CompleteOperatorEvidence(CompleteOperatorEvidenceArgs),
    ConsolidatePhase28Evidence(ConsolidatePhase28EvidenceArgs),
    Phase33Classify(Phase33ClassifyArgs),
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Phase33ClassifyMode {
    Baseline,
    Delivery,
    PostRestart,
}

#[derive(Debug, Parser)]
struct Phase33ClassifyArgs {
    #[arg(long, value_parser = parse_utf8_path)]
    trace: Utf8PathBuf,

    #[arg(long, value_enum)]
    mode: Phase33ClassifyMode,

    #[arg(long, default_value_t = 0)]
    start_byte: u64,

    #[arg(long)]
    expected_session: Option<String>,

    #[arg(long)]
    expected_ordinal: Option<u64>,
}

#[derive(Debug, Parser)]
struct ReportArgs {
    #[arg(long, default_value = "docs/parity/checklist.md", value_parser = parse_utf8_path)]
    checklist: Utf8PathBuf,

    #[arg(long, value_enum, default_value_t = ReportFormat::Text)]
    format: ReportFormat,

    #[arg(long = "fail-on-invalid-verified")]
    fail_on_invalid_verified: bool,
}

#[derive(Debug, Parser)]
struct ApiCompareArgs {
    #[arg(long, default_value = DEFAULT_OPENAPI_PATH, value_parser = parse_utf8_path)]
    openapi: Utf8PathBuf,

    #[arg(long, default_value = DEFAULT_API_COMPARE_MANIFEST, value_parser = parse_utf8_path)]
    route_manifest: Utf8PathBuf,

    #[arg(long, default_value = DEFAULT_AXEOS_ROUTE_USAGE, value_parser = parse_utf8_path)]
    static_usage: Utf8PathBuf,
}

#[derive(Debug, Parser)]
struct ReleaseGateArgs {
    #[arg(long, default_value = DEFAULT_LICENSE_INVENTORY_PATH, value_parser = parse_utf8_path)]
    license_inventory: Utf8PathBuf,

    #[arg(long, default_value = DEFAULT_PROVENANCE_PATH, value_parser = parse_utf8_path)]
    provenance: Utf8PathBuf,

    #[arg(long, default_value = DEFAULT_CARGO_ABOUT_PATH, value_parser = parse_utf8_path)]
    cargo_about: Utf8PathBuf,

    #[arg(long, value_name = "package-json", value_parser = parse_utf8_path)]
    manifest: Option<Utf8PathBuf>,
}

#[derive(Debug, Parser)]
struct ReleaseEvidenceArgs {
    #[arg(long, value_name = "package-json", value_parser = parse_utf8_path)]
    manifest: Utf8PathBuf,

    #[arg(long = "evidence-root", value_parser = parse_utf8_path)]
    evidence_root: Utf8PathBuf,

    #[arg(long = "flash-evidence-json", value_parser = parse_utf8_path)]
    maybe_flash_evidence_json: Option<Utf8PathBuf>,

    #[arg(long = "redaction-review", value_parser = parse_utf8_path)]
    maybe_redaction_review: Option<Utf8PathBuf>,

    #[arg(long = "require-redaction-passed")]
    require_redaction_passed: bool,

    #[arg(long = "allow-post-source-evidence-commits")]
    allow_post_source_evidence_commits: bool,
}

#[derive(Debug, Parser)]
struct SafetyAllowArgs {
    #[arg(long, value_parser = parse_utf8_path)]
    manifest: Utf8PathBuf,

    #[arg(long = "surface")]
    maybe_surface: Option<String>,

    #[arg(long = "allowed-command")]
    maybe_allowed_command: Option<String>,
}

#[derive(Debug, Parser)]
struct MiningAllowArgs {
    #[arg(long, value_parser = parse_utf8_path)]
    manifest: Utf8PathBuf,

    #[arg(long = "surface")]
    maybe_surface: Option<String>,

    #[arg(long = "allowed-command")]
    maybe_allowed_command: Option<String>,
}

#[derive(Debug, Parser)]
struct OperatorEvidenceArgs {
    #[arg(long, value_enum)]
    profile: OperatorEvidenceProfile,

    #[arg(long = "evidence-root", value_parser = parse_utf8_path)]
    evidence_root: Utf8PathBuf,

    #[arg(long = "require-redaction-passed")]
    require_redaction_passed: bool,
}

#[derive(Debug, Parser)]
struct CompleteOperatorEvidenceArgs {
    #[arg(long, value_enum)]
    profile: CompletionProfile,

    #[arg(long = "evidence-root", value_parser = parse_utf8_path)]
    evidence_root: Utf8PathBuf,

    #[arg(long = "workflow-status", value_enum)]
    workflow_status: WorkflowStatus,
}

#[derive(Debug, Parser)]
struct ConsolidatePhase28EvidenceArgs {
    #[arg(long = "phase27-root", value_parser = parse_utf8_path)]
    phase27_root: Utf8PathBuf,

    #[arg(long = "evidence-root", value_parser = parse_utf8_path)]
    evidence_root: Utf8PathBuf,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
#[value(rename_all = "lower")]
enum CompletionProfile {
    Phase25,
    Phase27,
}

impl From<CompletionProfile> for OperatorEvidenceProfile {
    fn from(profile: CompletionProfile) -> Self {
        match profile {
            CompletionProfile::Phase25 => Self::Phase25,
            CompletionProfile::Phase27 => Self::Phase27,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum ReportFormat {
    Text,
    Json,
}

#[derive(Debug)]
struct ReportRequest {
    checklist: Utf8PathBuf,
    format: ReportFormat,
    fail_on_invalid_verified: bool,
}

impl From<ReportArgs> for ReportRequest {
    fn from(args: ReportArgs) -> Self {
        Self {
            checklist: args.checklist,
            format: args.format,
            fail_on_invalid_verified: args.fail_on_invalid_verified,
        }
    }
}

#[derive(Debug, Serialize)]
struct ParityReport {
    reference_commit: String,
    rows: Vec<ChecklistRow>,
    validation_errors: Vec<ValidationError>,
}

impl ParityReport {
    #[cfg(test)]
    fn new(reference_commit: String, rows: Vec<ChecklistRow>) -> Self {
        Self::new_with_phase30_artifact(
            reference_commit,
            rows,
            &Phase30PromotionArtifactState::Unavailable(
                "structured Phase 30 evidence artifact was not loaded".to_owned(),
            ),
        )
    }

    fn new_with_phase30_artifact(
        reference_commit: String,
        rows: Vec<ChecklistRow>,
        phase30_artifact: &Phase30PromotionArtifactState,
    ) -> Self {
        let validation_errors = validate_rows_with_phase30_artifact(&rows, phase30_artifact);

        Self {
            reference_commit,
            rows,
            validation_errors,
        }
    }
}

#[derive(Debug, Eq, PartialEq, Serialize)]
struct ChecklistRow {
    id: String,
    surface: String,
    reference_breadcrumb: String,
    rust_owned_target: String,
    status: String,
    evidence: String,
    notes: String,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
struct ValidationError {
    id: String,
    message: String,
}

#[derive(Debug)]
struct Phase30PromotionArtifact {
    fields: BTreeMap<String, String>,
}

impl Phase30PromotionArtifact {
    fn field(&self, key: &str) -> Option<&str> {
        self.fields.get(key).map(String::as_str)
    }

    fn has_exact_field(&self, key: &str, value: &str) -> bool {
        self.field(key) == Some(value)
    }
}

#[derive(Debug)]
enum Phase30PromotionArtifactState {
    Available(Phase30PromotionArtifact),
    Unavailable(String),
    Malformed(String),
}

fn parse_phase30_promotion_artifact(
    document: &str,
) -> std::result::Result<Phase30PromotionArtifact, String> {
    let mut fields = BTreeMap::new();

    for line in document.lines() {
        let trimmed = line.trim();
        let Some((key, value)) = trimmed.split_once(": ") else {
            continue;
        };
        if key.is_empty()
            || !key
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "._-".contains(character))
        {
            continue;
        }
        if fields.insert(key.to_owned(), value.to_owned()).is_some() {
            return Err(format!("duplicate structured Phase 30 field {key}"));
        }
    }

    let artifact = Phase30PromotionArtifact { fields };
    validate_phase30_artifact_closed_fields(&artifact)?;
    Ok(artifact)
}

fn validate_phase30_artifact_closed_fields(
    artifact: &Phase30PromotionArtifact,
) -> std::result::Result<(), String> {
    require_phase30_field_value(
        artifact,
        "phase30_disposition",
        &["no_promotion_no_eligible_evidence", "promoted"],
    )?;
    require_phase30_field_value(artifact, "new_evidence_input", &["none", "explicit"])?;
    require_phase30_field_value(artifact, "archived_lineage_verification", &["gaps_found"])?;
    require_phase30_field_value(
        artifact,
        "eligible_share_outcome",
        &["none", "accepted", "rejected"],
    )?;
    require_phase30_field_value(artifact, "hardware_accessed", &["false", "true"])?;
    require_phase30_field_value(artifact, "credentials_accessed", &["false", "true"])?;
    require_phase30_field_value(artifact, "raw_artifacts_committed", &["no"])?;

    match artifact.field("phase30_disposition") {
        Some("no_promotion_no_eligible_evidence") => {
            require_phase30_field_value(artifact, "new_evidence_input", &["none"])?;
            require_phase30_field_value(artifact, "eligible_share_outcome", &["none"])?;
            require_phase30_field_value(artifact, "hardware_accessed", &["false"])?;
            require_phase30_field_value(artifact, "credentials_accessed", &["false"])?;
        }
        Some("promoted") => {
            require_phase30_field_value(artifact, "new_evidence_input", &["explicit"])?;
            require_phase30_field_value(
                artifact,
                "eligible_share_outcome",
                &["accepted", "rejected"],
            )?;
            require_phase30_field_value(artifact, "hardware_accessed", &["true"])?;
            for (key, value) in [
                ("current_source_gate", "passed"),
                ("detector_gate", "passed"),
                ("same_chain_gate", "passed"),
                ("provenance_gate", "passed"),
                ("redaction_status", "passed"),
            ] {
                require_phase30_field_value(artifact, key, &[value])?;
            }
        }
        _ => return Err("invalid phase30_disposition".to_owned()),
    }

    Ok(())
}

fn require_phase30_field_value(
    artifact: &Phase30PromotionArtifact,
    key: &str,
    allowed_values: &[&str],
) -> std::result::Result<(), String> {
    let Some(value) = artifact.field(key) else {
        return Err(format!("missing structured Phase 30 field {key}"));
    };
    if allowed_values.contains(&value) {
        return Ok(());
    }

    Err(format!(
        "invalid structured Phase 30 value for {key}: expected {}",
        allowed_values.join(" or ")
    ))
}

trait ReportEnvironment {
    fn run_reference_guard(&self) -> Result<()>;
    fn read_checklist(&self, path: &Utf8Path) -> Result<String>;
    fn read_phase30_promotion_artifact(&self, path: &Utf8Path) -> Result<String>;
    fn reference_commit(&self) -> Result<String>;
}

#[derive(Debug)]
struct LocalEnvironment {
    workspace_dir: Utf8PathBuf,
    reference_guard_path: Utf8PathBuf,
}

impl LocalEnvironment {
    fn detect() -> Result<Self> {
        let workspace_dir = detect_workspace_dir()?;
        let reference_guard_path = detect_reference_guard_path(&workspace_dir);

        Ok(Self {
            workspace_dir,
            reference_guard_path,
        })
    }
}

impl ReportEnvironment for LocalEnvironment {
    fn run_reference_guard(&self) -> Result<()> {
        let output = ProcessCommand::new("bash")
            .arg(self.reference_guard_path.as_std_path())
            .env("BUILD_WORKSPACE_DIRECTORY", self.workspace_dir.as_str())
            .output()
            .with_context(|| {
                format!(
                    "failed to run reference guard {BAZEL_REFERENCE_GUARD_TARGET} at {}",
                    self.reference_guard_path
                )
            })?;

        if output.status.success() {
            return Ok(());
        }

        bail!(
            "reference guard {BAZEL_REFERENCE_GUARD_TARGET} failed: {}",
            command_stderr_or_status(&output)
        );
    }

    fn read_checklist(&self, path: &Utf8Path) -> Result<String> {
        let checklist_path = self.workspace_path(path);
        std::fs::read_to_string(checklist_path.as_std_path())
            .with_context(|| format!("failed to read checklist {checklist_path}"))
    }

    fn read_phase30_promotion_artifact(&self, path: &Utf8Path) -> Result<String> {
        let artifact_path = self.workspace_path(path);
        std::fs::read_to_string(artifact_path.as_std_path())
            .with_context(|| format!("failed to read Phase 30 promotion artifact {artifact_path}"))
    }

    fn reference_commit(&self) -> Result<String> {
        let reference_dir = self.workspace_dir.join(DEFAULT_REFERENCE_DIR);
        let output = ProcessCommand::new("git")
            .args(["-C", reference_dir.as_str(), "rev-parse", "HEAD"])
            .output()
            .with_context(|| format!("failed to read reference commit from {reference_dir}"))?;

        if !output.status.success() {
            bail!(
                "failed to read reference commit from {reference_dir}: {}",
                command_stderr_or_status(&output)
            );
        }

        let commit = String::from_utf8(output.stdout)
            .context("reference commit output was not valid UTF-8")?;
        let trimmed = commit.trim();
        if trimmed.is_empty() {
            bail!("reference commit output was empty");
        }

        Ok(trimmed.to_owned())
    }
}

impl LocalEnvironment {
    fn workspace_path(&self, path: &Utf8Path) -> Utf8PathBuf {
        if path.is_absolute() {
            return path.to_owned();
        }

        self.workspace_dir.join(path)
    }

    fn current_git_head(&self) -> Result<String> {
        let output = ProcessCommand::new("git")
            .args(["-C", self.workspace_dir.as_str(), "rev-parse", "HEAD"])
            .output()
            .with_context(|| {
                format!(
                    "failed to read current git HEAD from {}",
                    self.workspace_dir
                )
            })?;

        if !output.status.success() {
            bail!(
                "failed to read current git HEAD from {}: {}",
                self.workspace_dir,
                command_stderr_or_status(&output)
            );
        }

        let commit = String::from_utf8(output.stdout)
            .context("current git HEAD output was not valid UTF-8")?;
        let trimmed = commit.trim();
        if trimmed.is_empty() {
            bail!("current git HEAD output was empty");
        }

        Ok(trimmed.to_owned())
    }

    fn source_commit_is_ancestor_of_head(&self, source_commit: &str) -> Result<bool> {
        let output = ProcessCommand::new("git")
            .args([
                "-C",
                self.workspace_dir.as_str(),
                "merge-base",
                "--is-ancestor",
                source_commit,
                "HEAD",
            ])
            .output()
            .with_context(|| {
                format!(
                    "failed to compare package source commit {source_commit} with HEAD in {}",
                    self.workspace_dir
                )
            })?;

        if output.status.success() {
            return Ok(true);
        }

        if output.status.code() == Some(1) {
            return Ok(false);
        }

        bail!(
            "failed to compare package source commit {source_commit} with HEAD in {}: {}",
            self.workspace_dir,
            command_stderr_or_status(&output)
        );
    }

    fn changed_paths_since(&self, source_commit: &str) -> Result<Vec<Utf8PathBuf>> {
        let output = ProcessCommand::new("git")
            .args([
                "-C",
                self.workspace_dir.as_str(),
                "diff",
                "--name-only",
                &format!("{source_commit}..HEAD"),
            ])
            .output()
            .with_context(|| {
                format!("failed to list paths changed since package source commit {source_commit}")
            })?;

        if !output.status.success() {
            bail!(
                "failed to list paths changed since package source commit {source_commit}: {}",
                command_stderr_or_status(&output)
            );
        }

        let stdout =
            String::from_utf8(output.stdout).context("git diff output was not valid UTF-8")?;
        Ok(stdout
            .lines()
            .filter(|line| !line.trim().is_empty())
            .map(Utf8PathBuf::from)
            .collect())
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let environment = LocalEnvironment::detect()?;

    let output = match cli.command {
        CliCommand::Report(args) => {
            let request = ReportRequest::from(args);
            run_report(&request, &environment)?
        }
        CliCommand::ApiCompare(args) => run_api_compare_command(args, &environment)?,
        CliCommand::ReleaseGate(args) => run_release_gate_command(args, &environment)?,
        CliCommand::ReleaseEvidence(args) => run_release_evidence_command(args, &environment)?,
        CliCommand::SafetyAllow(args) => run_safety_allow_command(args, &environment)?,
        CliCommand::MiningAllow(args) => run_mining_allow_command(args, &environment)?,
        CliCommand::OperatorEvidence(args) => run_operator_evidence_command(args, &environment)?,
        CliCommand::CompleteOperatorEvidence(args) => {
            run_complete_operator_evidence_command(args, &environment)?
        }
        CliCommand::ConsolidatePhase28Evidence(args) => {
            run_consolidate_phase28_evidence_command(args, &environment)?
        }
        CliCommand::Phase33Classify(args) => run_phase33_classify_command(args, &environment)?,
    };

    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{output}")?;

    Ok(())
}

#[derive(Serialize)]
struct Phase33ClassificationOutput {
    status: &'static str,
    category: &'static str,
    session: Option<String>,
    boot_ordinal: Option<u64>,
    device_url: Option<String>,
}

fn run_phase33_classify_command(
    args: Phase33ClassifyArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    let path = environment.workspace_path(&args.trace);
    let bytes = std::fs::read(path.as_std_path())
        .with_context(|| format!("failed to read Phase 33 trace {path}"))?;
    let start = usize::try_from(args.start_byte).context("start-byte does not fit usize")?;
    if start > bytes.len() {
        bail!("start-byte exceeds Phase 33 trace length");
    }
    let text = String::from_utf8_lossy(&bytes[start..]);
    let expected = || -> Result<(&str, u64)> {
        Ok((
            args.expected_session
                .as_deref()
                .context("expected-session is required for this mode")?,
            args.expected_ordinal
                .context("expected-ordinal is required for this mode")?,
        ))
    };

    let classified = match args.mode {
        Phase33ClassifyMode::Baseline => classify_phase33_baseline(&text),
        Phase33ClassifyMode::Delivery => {
            let (session, ordinal) = expected()?;
            classify_phase33_delivery(&text, session, ordinal).map(|()| {
                bitaxe_api::phase33_evidence::Phase33BootEvidence {
                    session: session.to_owned(),
                    boot_ordinal: ordinal,
                    device_url: String::new(),
                }
            })
        }
        Phase33ClassifyMode::PostRestart => {
            let (session, ordinal) = expected()?;
            classify_phase33_post_restart(&text, session, ordinal)
        }
    };

    let output = match classified {
        Ok(evidence) => Phase33ClassificationOutput {
            status: "passed",
            category: "none",
            session: Some(evidence.session),
            boot_ordinal: Some(evidence.boot_ordinal),
            device_url: (!evidence.device_url.is_empty()).then_some(evidence.device_url),
        },
        Err(error) => Phase33ClassificationOutput {
            status: "failed",
            category: error.category,
            session: None,
            boot_ordinal: None,
            device_url: None,
        },
    };
    serde_json::to_string(&output).context("failed to encode Phase 33 classification")
}

fn run_complete_operator_evidence_command(
    args: CompleteOperatorEvidenceArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    let created = complete_operator_evidence(
        &environment.workspace_dir,
        args.profile.into(),
        &args.evidence_root,
        args.workflow_status,
    )?;
    Ok(format!(
        "operator_evidence_completion_status: passed\ncreated_slots: {}",
        created.len()
    ))
}

fn run_consolidate_phase28_evidence_command(
    args: ConsolidatePhase28EvidenceArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    consolidate_phase28_evidence(
        &environment.workspace_dir,
        &args.phase27_root,
        &args.evidence_root,
    )?;
    Ok("phase28_consolidation_status: passed".to_owned())
}

fn run_release_evidence_command(
    args: ReleaseEvidenceArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    let manifest_path = environment.workspace_path(&args.manifest);
    let manifest_json = std::fs::read_to_string(manifest_path.as_std_path())
        .with_context(|| format!("failed to read package manifest {manifest_path}"))?;
    let manifest = parse_release_evidence_manifest_json(&manifest_json, &args.manifest)?;
    let current_git_head = environment.current_git_head()?;
    let source_commit_is_ancestor_of_head = args.allow_post_source_evidence_commits
        && current_git_head != manifest.source_commit
        && environment.source_commit_is_ancestor_of_head(&manifest.source_commit)?;
    let post_source_changed_paths =
        if args.allow_post_source_evidence_commits && current_git_head != manifest.source_commit {
            environment.changed_paths_since(&manifest.source_commit)?
        } else {
            Vec::new()
        };

    let maybe_flash_evidence = if let Some(flash_evidence_path) = &args.maybe_flash_evidence_json {
        let workspace_flash_evidence_path = environment.workspace_path(flash_evidence_path);
        let flash_evidence_json =
            std::fs::read_to_string(workspace_flash_evidence_path.as_std_path()).with_context(
                || format!("failed to read flash evidence {workspace_flash_evidence_path}"),
            )?;
        Some(parse_flash_evidence_json(
            &flash_evidence_json,
            &workspace_flash_evidence_path,
        )?)
    } else {
        None
    };

    let maybe_redaction_review = if let Some(redaction_review_path) = &args.maybe_redaction_review {
        let workspace_redaction_review_path = environment.workspace_path(redaction_review_path);
        Some(
            std::fs::read_to_string(workspace_redaction_review_path.as_std_path()).with_context(
                || format!("failed to read redaction review {workspace_redaction_review_path}"),
            )?,
        )
    } else {
        None
    };
    let (evidence_root, maybe_flash_evidence_json_path) =
        release_evidence_validation_paths(&args, environment);

    let documents = ReleaseEvidenceDocuments {
        manifest,
        current_git_head,
        allow_post_source_evidence_commits: args.allow_post_source_evidence_commits,
        source_commit_is_ancestor_of_head,
        post_source_changed_paths,
        evidence_root,
        maybe_flash_evidence_json_path,
        maybe_flash_evidence,
        maybe_redaction_review,
    };
    let report = validate_release_evidence(&documents, args.require_redaction_passed);
    let output = render_release_evidence_report(&documents, &report);

    if !report.passed() {
        bail!("release evidence failed:\n{output}");
    }

    Ok(output)
}

fn release_evidence_validation_paths(
    args: &ReleaseEvidenceArgs,
    environment: &LocalEnvironment,
) -> (Utf8PathBuf, Option<Utf8PathBuf>) {
    (
        environment.workspace_path(&args.evidence_root),
        args.maybe_flash_evidence_json
            .as_ref()
            .map(|path| environment.workspace_path(path)),
    )
}

fn run_mining_allow_command(
    args: MiningAllowArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    let manifest_path = environment.workspace_path(&args.manifest);
    let documents =
        mining_allow::load_mining_allow_documents(&environment.workspace_dir, &manifest_path)?;
    let filters = mining_allow::MiningAllowFilters {
        maybe_surface: args.maybe_surface,
        maybe_allowed_command: args.maybe_allowed_command,
    };
    let report = mining_allow::validate_mining_allow_documents(&documents, &filters);
    let output = mining_allow::render_mining_allow_report(&documents.manifest, &report);

    if !report.passed() {
        bail!("mining allow failed:\n{output}");
    }

    Ok(output)
}

fn run_operator_evidence_command(
    args: OperatorEvidenceArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    let evidence_root = environment.workspace_path(&args.evidence_root);
    let documents = load_operator_evidence_documents(&evidence_root)?;
    let filters = OperatorEvidenceFilters {
        require_redaction_passed: args.require_redaction_passed,
    };
    let report = validate_operator_evidence_documents(args.profile, &documents, &filters);
    let output = render_operator_evidence_report(&documents, &report);

    if !report.passed() {
        bail!("operator evidence failed:\n{output}");
    }

    Ok(output)
}

fn run_safety_allow_command(
    args: SafetyAllowArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    let manifest_path = environment.workspace_path(&args.manifest);
    let documents =
        safety_allow::load_safety_allow_documents(&environment.workspace_dir, &manifest_path)?;
    let filters = safety_allow::SafetyAllowFilters {
        maybe_surface: args.maybe_surface,
        maybe_allowed_command: args.maybe_allowed_command,
    };
    let report = safety_allow::validate_safety_allow_documents(&documents, &filters);
    let output = safety_allow::render_safety_allow_report(&documents.manifest, &report);

    if !report.passed() {
        bail!("safety allow failed:\n{output}");
    }

    Ok(output)
}

fn run_release_gate_command(
    args: ReleaseGateArgs,
    environment: &LocalEnvironment,
) -> Result<String> {
    let license_inventory_path = environment.workspace_path(&args.license_inventory);
    let provenance_path = environment.workspace_path(&args.provenance);
    let cargo_about_path = environment.workspace_path(&args.cargo_about);
    let maybe_manifest_path = args
        .manifest
        .as_ref()
        .map(|manifest| environment.workspace_path(manifest));

    let license_inventory_markdown = std::fs::read_to_string(license_inventory_path.as_std_path())
        .with_context(|| format!("failed to read license inventory {license_inventory_path}"))?;
    let provenance_markdown = std::fs::read_to_string(provenance_path.as_std_path())
        .with_context(|| format!("failed to read provenance manifest {provenance_path}"))?;
    let maybe_cargo_about_html = read_optional_text(&cargo_about_path)?;
    let maybe_manifest_json = if let Some(manifest_path) = &maybe_manifest_path {
        read_optional_text(manifest_path)?
    } else {
        None
    };

    let documents = ReleaseGateDocuments {
        license_inventory_path: args.license_inventory,
        license_inventory_markdown,
        provenance_path: args.provenance,
        provenance_markdown,
        cargo_about_path: args.cargo_about,
        maybe_cargo_about_html,
        maybe_manifest_path: args.manifest,
        maybe_manifest_json,
    };
    let report = validate_release_gate(&documents);
    let output = render_release_gate_report(&report);

    if !report.passed() {
        bail!("release gate failed:\n{output}");
    }

    Ok(output)
}

fn run_api_compare_command(args: ApiCompareArgs, environment: &LocalEnvironment) -> Result<String> {
    let openapi_path = environment.workspace_path(&args.openapi);
    let route_manifest_path = environment.workspace_path(&args.route_manifest);
    let static_usage_path = environment.workspace_path(&args.static_usage);

    let openapi_yaml = std::fs::read_to_string(openapi_path.as_std_path())
        .with_context(|| format!("failed to read OpenAPI contract {openapi_path}"))?;
    let route_manifest_json = std::fs::read_to_string(route_manifest_path.as_std_path())
        .with_context(|| format!("failed to read API compare manifest {route_manifest_path}"))?;
    let static_usage_json = std::fs::read_to_string(static_usage_path.as_std_path())
        .with_context(|| format!("failed to read AxeOS route usage fixture {static_usage_path}"))?;

    let request = api_compare::ApiCompareRequest {
        openapi_yaml: &openapi_yaml,
        route_manifest_json: &route_manifest_json,
        static_usage_json: &static_usage_json,
    };
    let loader = api_compare::WorkspaceFixtureLoader::new(environment.workspace_dir.clone());
    let report = api_compare::run_api_compare(&request, &loader)?;
    let output = api_compare::render_api_compare_report(&report);

    if report.has_validation_errors() {
        bail!("api compare failed:\n{output}");
    }

    Ok(output)
}

fn run_report(
    environment_request: &ReportRequest,
    environment: &impl ReportEnvironment,
) -> Result<String> {
    if let Err(error) = environment.run_reference_guard() {
        bail!("reference guard blocked parity report generation: {error:#}");
    }

    let checklist = environment
        .read_checklist(&environment_request.checklist)
        .with_context(|| format!("failed to load {}", environment_request.checklist))?;
    let rows = parse_checklist(&checklist)?;
    let phase30_artifact = load_phase30_promotion_artifact(&rows, environment);
    let reference_commit = environment.reference_commit()?;
    let report = ParityReport::new_with_phase30_artifact(reference_commit, rows, &phase30_artifact);

    if environment_request.fail_on_invalid_verified && !report.validation_errors.is_empty() {
        bail!(
            "invalid verified parity claims:\n{}",
            format_validation_errors(&report.validation_errors)
        );
    }

    render_report(&report, environment_request.format)
}

fn load_phase30_promotion_artifact(
    rows: &[ChecklistRow],
    environment: &impl ReportEnvironment,
) -> Phase30PromotionArtifactState {
    let has_verified_phase30_row = rows
        .iter()
        .any(|row| is_phase30_promotion_row(row) && normalize(&row.status) == "verified");
    if !has_verified_phase30_row {
        return Phase30PromotionArtifactState::Unavailable(
            "no verified Phase 30 row requested promotion admission".to_owned(),
        );
    }

    let document = match environment
        .read_phase30_promotion_artifact(Utf8Path::new(DEFAULT_PHASE30_PROMOTION_ARTIFACT_PATH))
    {
        Ok(document) => document,
        Err(error) => {
            return Phase30PromotionArtifactState::Unavailable(format!(
                "structured Phase 30 evidence artifact is missing or unreadable: {error:#}"
            ));
        }
    };

    match parse_phase30_promotion_artifact(&document) {
        Ok(artifact) => Phase30PromotionArtifactState::Available(artifact),
        Err(error) => Phase30PromotionArtifactState::Malformed(error),
    }
}

fn parse_checklist(checklist: &str) -> Result<Vec<ChecklistRow>> {
    let mut rows = Vec::new();

    for (line_index, line) in checklist.lines().enumerate() {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
            continue;
        }

        let cells: Vec<String> = trimmed
            .trim_matches('|')
            .split('|')
            .map(clean_markdown_cell)
            .collect();

        if is_header_or_separator(&cells) {
            continue;
        }

        if cells.len() != 7 {
            bail!(
                "invalid checklist row at line {}: expected 7 columns, found {}",
                line_index + 1,
                cells.len()
            );
        }

        rows.push(ChecklistRow {
            id: cells[0].clone(),
            surface: cells[1].clone(),
            reference_breadcrumb: cells[2].clone(),
            rust_owned_target: cells[3].clone(),
            status: cells[4].clone(),
            evidence: cells[5].clone(),
            notes: cells[6].clone(),
        });
    }

    Ok(rows)
}

#[cfg(test)]
fn validate_rows(rows: &[ChecklistRow]) -> Vec<ValidationError> {
    validate_rows_with_phase30_artifact(
        rows,
        &Phase30PromotionArtifactState::Unavailable(
            "structured Phase 30 evidence artifact was not loaded".to_owned(),
        ),
    )
}

fn validate_rows_with_phase30_artifact(
    rows: &[ChecklistRow],
    phase30_artifact: &Phase30PromotionArtifactState,
) -> Vec<ValidationError> {
    let mut errors = Vec::new();

    if let Err(message) = v12_admission::validate_closed_phase31_contract() {
        errors.push(ValidationError {
            id: "PHASE-31".to_owned(),
            message,
        });
    }

    for row in rows {
        if normalize(&row.status) != "verified" {
            continue;
        }

        if normalize(&row.evidence) == "pending" {
            errors.push(ValidationError {
                id: row.id.clone(),
                message: "verified rows require non-pending evidence".to_owned(),
            });
        }

        if is_safety_critical(row) && !has_hardware_evidence(row) {
            errors.push(ValidationError {
                id: row.id.clone(),
                message: "safety-critical verified rows require hardware-smoke or hardware-regression evidence".to_owned(),
            });
        }

        if is_active_safety_control(row) && !has_evidence_token(row, "hardware-regression") {
            errors.push(ValidationError {
                id: row.id.clone(),
                message: "active safety-control verified row requires hardware-regression evidence"
                    .to_owned(),
            });
        }

        errors.extend(validate_live_asic_mining_verified_row(row));
        errors.extend(validate_release_ota_verified_row(row));
        errors.extend(validate_deferred_scope_verified_row(row));
        errors.extend(validate_phase26_telemetry_verified_row(row));
        errors.extend(validate_phase28_hardware_promotion_row(
            row,
            phase30_artifact,
        ));
        errors.extend(validate_phase30_promotion_row(row, phase30_artifact));
    }

    errors
}

fn render_report(report: &ParityReport, format: ReportFormat) -> Result<String> {
    match format {
        ReportFormat::Json => {
            serde_json::to_string_pretty(report).context("failed to serialize parity report")
        }
        ReportFormat::Text => Ok(render_text_report(report)),
    }
}

fn render_text_report(report: &ParityReport) -> String {
    let mut output = String::new();
    output.push_str(&format!("reference_commit: {}\n", report.reference_commit));
    output.push_str("rows:\n");

    for row in &report.rows {
        output.push_str(&format!(
            "- {} | status={} | evidence={}\n  reference_breadcrumb: {}\n  rust_owned_target: {}\n  notes: {}\n",
            row.id,
            row.status,
            row.evidence,
            row.reference_breadcrumb,
            row.rust_owned_target,
            row.notes
        ));
    }

    if report.validation_errors.is_empty() {
        output.push_str("validation_errors: none\n");
    } else {
        output.push_str("validation_errors:\n");
        output.push_str(&format_validation_errors(&report.validation_errors));
    }

    output
}

fn format_validation_errors(errors: &[ValidationError]) -> String {
    let mut output = String::new();

    for error in errors {
        output.push_str(&format!("- {}: {}\n", error.id, error.message));
    }

    output
}

fn is_header_or_separator(cells: &[String]) -> bool {
    let Some(first_cell) = cells.first() else {
        return false;
    };

    first_cell == "ID" || cells.iter().all(|cell| cell.chars().all(is_separator_char))
}

fn is_separator_char(character: char) -> bool {
    matches!(character, '-' | ':' | ' ')
}

fn clean_markdown_cell(cell: &str) -> String {
    cell.trim().replace('`', "")
}

fn is_safety_critical(row: &ChecklistRow) -> bool {
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

fn has_hardware_evidence(row: &ChecklistRow) -> bool {
    has_evidence_token(row, "hardware-smoke") || has_evidence_token(row, "hardware-regression")
}

fn validate_live_asic_mining_verified_row(row: &ChecklistRow) -> Vec<ValidationError> {
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

fn is_live_asic_or_mining_row(row: &ChecklistRow) -> bool {
    matches!(
        row.id.as_str(),
        "ASIC-002" | "ASIC-003" | "ASIC-004" | "ASIC-005" | "ASIC-007" | "STR-006" | "STR-008"
    )
}

fn has_live_asic_mining_evidence(row: &ChecklistRow) -> bool {
    has_hardware_evidence(row) || has_evidence_token(row, "soak")
}

fn has_bounded_frequency_transition_regression(row: &ChecklistRow) -> bool {
    let haystack = row_haystack(row);

    has_evidence_token(row, "hardware-regression")
        && haystack.contains("bounded")
        && (haystack.contains("frequency-transition") || haystack.contains("frequency transition"))
        && haystack.contains("hardware")
}

fn has_mining_smoke_or_soak_details(row: &ChecklistRow) -> bool {
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

fn is_active_safety_control(row: &ChecklistRow) -> bool {
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

fn validate_release_ota_verified_row(row: &ChecklistRow) -> Vec<ValidationError> {
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

fn validate_filesystem_verified_row(row: &ChecklistRow) -> Vec<ValidationError> {
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

fn validate_firmware_ota_verified_row(row: &ChecklistRow) -> Vec<ValidationError> {
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

fn validate_otawww_verified_row(row: &ChecklistRow) -> Vec<ValidationError> {
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

fn validate_release_sensitive_verified_row(row: &ChecklistRow) -> Vec<ValidationError> {
    if has_hardware_evidence(row) || row_haystack(row).contains("release-gate") {
        return Vec::new();
    }

    vec![ValidationError {
        id: row.id.clone(),
        message: "release-sensitive verified rows require hardware-smoke, hardware-regression, or release-gate evidence beyond unit/workflow/api-compare/package-only evidence".to_owned(),
    }]
}

fn validate_release_image_verified_row(row: &ChecklistRow) -> Vec<ValidationError> {
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

fn validate_deferred_scope_verified_row(row: &ChecklistRow) -> Vec<ValidationError> {
    if !is_deferred_or_non_205_scope(row) || !uses_ultra_205_evidence(row) {
        return Vec::new();
    }

    vec![ValidationError {
        id: row.id.clone(),
        message: "deferred or non-205 verified rows cannot reuse Ultra 205 evidence".to_owned(),
    }]
}

fn validate_phase26_telemetry_verified_row(row: &ChecklistRow) -> Vec<ValidationError> {
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

fn is_phase26_telemetry_row(row: &ChecklistRow) -> bool {
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

fn validate_phase30_promotion_row(
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

    if let Some(forbidden_category) = phase30_forbidden_category(row) {
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

fn is_phase30_promotion_row(row: &ChecklistRow) -> bool {
    matches!(row.id.as_str(), "STR-09" | "CFG-07" | "ASIC-11")
}

fn phase30_forbidden_category(row: &ChecklistRow) -> Option<&'static str> {
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

fn phase30_missing_artifact_row_fields(
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

fn has_phase30_exact_promotion_proof(
    row: &ChecklistRow,
    artifact_state: &Phase30PromotionArtifactState,
) -> bool {
    let Phase30PromotionArtifactState::Available(artifact) = artifact_state else {
        return false;
    };

    is_phase30_promotion_row(row)
        && row_haystack(row).contains(DEFAULT_PHASE30_PROMOTION_ARTIFACT_PATH)
        && phase30_forbidden_category(row).is_none()
        && artifact.has_exact_field("phase30_disposition", "promoted")
        && phase30_missing_artifact_row_fields(row, artifact).is_empty()
}

fn validate_phase28_hardware_promotion_row(
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

fn is_phase28_hardware_promotion_row(row: &ChecklistRow) -> bool {
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

fn has_str09_accepted_rejected_hardware_share_proof(row: &ChecklistRow) -> bool {
    let haystack = row_haystack(row);
    has_hardware_evidence(row)
        && (haystack.contains("accepted share hardware")
            || haystack.contains("rejected share hardware")
            || haystack.contains("accepted share proof")
            || haystack.contains("rejected share proof"))
}

fn has_phase28_live_safety_hardware_proof(row: &ChecklistRow) -> bool {
    let haystack = row_haystack(row);
    has_hardware_evidence(row)
        && (haystack.contains("detector-gated live safety")
            || haystack.contains("live safety hardware proof")
            || haystack.contains("active voltage regression")
            || haystack.contains("thermal fault stimulus hardware"))
}

fn has_phase28_hardware_bridge_socket_proof(row: &ChecklistRow) -> bool {
    let haystack = row_haystack(row);
    has_hardware_evidence(row)
        && (haystack.contains("live socket success")
            || haystack.contains("asic bridge correlation")
            || haystack.contains("accepted share hardware")
            || haystack.contains("rejected share hardware"))
}

fn is_deferred_or_non_205_scope(row: &ChecklistRow) -> bool {
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

fn uses_ultra_205_evidence(row: &ChecklistRow) -> bool {
    let haystack = row_haystack(row);
    haystack.contains("ultra 205") || haystack.contains("ultra205")
}

fn has_evidence_token(row: &ChecklistRow, expected: &str) -> bool {
    row.evidence
        .split(',')
        .map(normalize)
        .any(|token| token == expected)
}

fn row_contains_live_evidence_blocker(row: &ChecklistRow) -> bool {
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

fn live_evidence_blocker_error(row: &ChecklistRow) -> ValidationError {
    ValidationError {
        id: row.id.clone(),
        message: "verified live release/OTA/filesystem rows must not contain blocker terms such as not run, blocked, pending, no reachable DEVICE_URL, or unverified".to_owned(),
    }
}

fn live_asic_mining_blocker_error(row: &ChecklistRow) -> ValidationError {
    ValidationError {
        id: row.id.clone(),
        message: "verified live ASIC/mining rows must not contain blocker terms such as missing live prerequisites, not run, blocked, pending, below verified, no reachable DEVICE_URL, or unverified".to_owned(),
    }
}

fn row_haystack(row: &ChecklistRow) -> String {
    format!(
        "{} {} {} {} {} {}",
        row.id, row.surface, row.rust_owned_target, row.status, row.evidence, row.notes
    )
    .to_ascii_lowercase()
}

struct RequiredTerm {
    label: &'static str,
    needle: &'static str,
}

impl RequiredTerm {
    const fn new(label: &'static str, needle: &'static str) -> Self {
        Self { label, needle }
    }
}

fn missing_required_terms(
    row: &ChecklistRow,
    required_terms: &[RequiredTerm],
) -> Vec<&'static str> {
    let haystack = row_haystack(row);

    required_terms
        .iter()
        .filter(|term| !haystack.contains(term.needle))
        .map(|term| term.label)
        .collect()
}

fn format_required_terms(missing_terms: &[&'static str]) -> String {
    if missing_terms.is_empty() {
        return "required release evidence terms".to_owned();
    }

    missing_terms.join(", ")
}

fn normalize(value: &str) -> String {
    value.trim().to_ascii_lowercase()
}

fn parse_utf8_path(value: &str) -> std::result::Result<Utf8PathBuf, String> {
    if value.trim().is_empty() {
        return Err("path must not be empty".to_owned());
    }

    Ok(Utf8PathBuf::from(value))
}

fn read_optional_text(path: &Utf8Path) -> Result<Option<String>> {
    match std::fs::read_to_string(path.as_std_path()) {
        Ok(contents) => Ok(Some(contents)),
        Err(error) if error.kind() == ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error).with_context(|| format!("failed to read {path}")),
    }
}

fn detect_workspace_dir() -> Result<Utf8PathBuf> {
    if let Ok(workspace_dir) = env::var("BUILD_WORKSPACE_DIRECTORY") {
        if !workspace_dir.trim().is_empty() {
            return Ok(Utf8PathBuf::from(workspace_dir));
        }
    }

    let output = ProcessCommand::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .context("failed to detect workspace root with git rev-parse --show-toplevel")?;

    if !output.status.success() {
        bail!(
            "failed to detect workspace root: {}",
            command_stderr_or_status(&output)
        );
    }

    let stdout = String::from_utf8(output.stdout).context("workspace path was not valid UTF-8")?;
    let workspace_dir = stdout.trim();
    if workspace_dir.is_empty() {
        bail!("workspace path output was empty");
    }

    Ok(Utf8PathBuf::from(workspace_dir))
}

fn detect_reference_guard_path(workspace_dir: &Utf8Path) -> Utf8PathBuf {
    let maybe_guard_path = env::var("BITAXE_REFERENCE_GUARD").ok();
    if let Some(guard_path) = maybe_guard_path {
        if !guard_path.trim().is_empty() {
            return Utf8PathBuf::from(guard_path);
        }
    }

    workspace_dir.join(DEFAULT_REFERENCE_GUARD_PATH)
}

fn command_stderr_or_status(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let trimmed = stderr.trim();
    if !trimmed.is_empty() {
        return trimmed.to_owned();
    }

    format!("process exited with status {}", output.status)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use anyhow::{anyhow, Result};
    use camino::{Utf8Path, Utf8PathBuf};

    use super::*;

    const CHECKLIST: &str = r#"
# Parity Checklist

## Foundation

| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| WF-001 | Read-only reference submodule | `reference/esp-miner` | `scripts/verify-reference-clean.sh` | implemented | pending | Guard exists. |
"#;

    #[test]
    fn parses_markdown_checklist_rows() {
        // Arrange
        let checklist = CHECKLIST;

        // Act
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Assert
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].id, "WF-001");
        assert_eq!(rows[0].status, "implemented");
        assert_eq!(rows[0].evidence, "pending");
        assert_eq!(rows[0].reference_breadcrumb, "reference/esp-miner");
        assert_eq!(
            rows[0].rust_owned_target,
            "scripts/verify-reference-clean.sh"
        );
    }

    #[test]
    fn json_output_includes_reference_commit() {
        // Arrange
        let rows = parse_checklist(CHECKLIST).expect("checklist should parse");
        let report = ParityReport::new("abc123".to_owned(), rows);

        // Act
        let output = render_report(&report, ReportFormat::Json).expect("json should render");
        let parsed: serde_json::Value =
            serde_json::from_str(&output).expect("output should be valid json");

        // Assert
        assert_eq!(parsed["reference_commit"], "abc123");
        assert_eq!(parsed["rows"][0]["id"], "WF-001");
    }

    #[test]
    fn release_evidence_validation_paths_resolve_relative_inputs_under_workspace() {
        // Arrange
        let environment = LocalEnvironment {
            workspace_dir: Utf8PathBuf::from("/tmp/bitaxe-workspace"),
            reference_guard_path: Utf8PathBuf::from("unused-reference-guard"),
        };
        let args = ReleaseEvidenceArgs {
            manifest: Utf8PathBuf::from("docs/evidence/package.json"),
            evidence_root: Utf8PathBuf::from("docs/evidence"),
            maybe_flash_evidence_json: Some(Utf8PathBuf::from("docs/evidence/flash.json")),
            maybe_redaction_review: None,
            require_redaction_passed: false,
            allow_post_source_evidence_commits: false,
        };

        // Act
        let (evidence_root, maybe_flash_evidence_json_path) =
            release_evidence_validation_paths(&args, &environment);

        // Assert
        assert_eq!(
            evidence_root,
            Utf8PathBuf::from("/tmp/bitaxe-workspace/docs/evidence")
        );
        assert_eq!(
            maybe_flash_evidence_json_path,
            Some(Utf8PathBuf::from(
                "/tmp/bitaxe-workspace/docs/evidence/flash.json"
            ))
        );
    }

    #[test]
    fn verified_rows_with_pending_evidence_are_invalid() {
        // Arrange
        let checklist = CHECKLIST.replace("implemented | pending", "verified | pending");
        let rows = parse_checklist(&checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("pending evidence"));
    }

    #[test]
    fn safety_critical_verified_rows_require_hardware_evidence() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| PWR-003 | Core voltage control | `reference/esp-miner/main/power/vcore.c` | `firmware/bitaxe` | verified | unit | Safety-critical. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(
            &errors,
            "PWR-003",
            "hardware-smoke or hardware-regression",
        );
    }

    #[test]
    fn safety_critical_notes_require_hardware_evidence() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| PWR-001 | ASIC reset behavior | `reference/esp-miner/main/power/asic_reset.c` | `firmware/bitaxe` | verified | unit | Safety-critical; requires hardware evidence. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(
            &errors,
            "PWR-001",
            "hardware-smoke or hardware-regression",
        );
    }

    #[test]
    fn safety_critical_self_test_verified_rows_require_hardware_evidence() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| SELF-001 | Self-test lifecycle | `reference/esp-miner/main/self_test/self_test.c` | `crates/bitaxe-safety`, `firmware/bitaxe` | verified | unit | Self-test hardware requires Ultra 205 hardware smoke before verification. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(
            &errors,
            "SELF-001",
            "hardware-smoke or hardware-regression",
        );
    }

    #[test]
    fn safety_critical_runtime_input_display_verified_rows_require_hardware_evidence() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| UI-003 | Input behavior | `reference/esp-miner/main/input.c` | `firmware/bitaxe` | verified | workflow | Runtime input and runtime display hardware-control rows require hardware-smoke evidence. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(
            &errors,
            "UI-003",
            "hardware-smoke or hardware-regression",
        );
    }

    #[test]
    fn safety_critical_implemented_rows_do_not_require_hardware_evidence() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| THR-003 | PID behavior | `reference/esp-miner/main/thermal/PID.c` | `crates/bitaxe-safety/src/thermal.rs` | implemented | unit | Pure PID behavior is covered by unit tests; hardware fan and thermal verification remains separate. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert!(errors.is_empty());
    }

    #[test]
    fn active_safety_control_verified_rows_require_hardware_regression() {
        // Arrange
        let active_ids = [
            "PWR-001", "PWR-002", "PWR-003", "PWR-005", "ASIC-007", "THR-001", "THR-002",
            "SELF-001", "UI-003",
        ];

        for active_id in active_ids {
            let checklist = format!(
                r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| {active_id} | Active safety-control row | `reference/esp-miner/main/safety.c` | `firmware/bitaxe` | verified | hardware-smoke | Active hardware-control behavior cannot be proven by broad smoke evidence. |
"#
            );
            let rows = parse_checklist(&checklist).expect("checklist should parse");

            // Act
            let errors = validate_rows(&rows);

            // Assert
            assert_validation_error_contains(
                &errors,
                active_id,
                "requires hardware-regression evidence",
            );
        }
    }

    #[test]
    fn active_safety_control_allows_hardware_regression_evidence() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| PWR-003 | Core voltage control | `reference/esp-miner/main/power/vcore.c` | `firmware/bitaxe` | verified | hardware-regression | Active voltage regression passed. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert!(errors.is_empty());
    }

    #[test]
    fn active_safety_control_allows_read_only_hardware_smoke_rows() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| PWR-006 | INA260 power telemetry freshness | `reference/esp-miner/main/power/INA260.c` | `firmware/bitaxe` | verified | hardware-smoke | Read-only INA260 current, bus voltage, and power telemetry freshness observed; no voltage writes claimed. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert!(errors.is_empty());
    }

    #[test]
    fn asic007_verified_requires_bounded_frequency_transition_hardware_regression() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| ASIC-007 | Frequency transition behavior | `reference/esp-miner/components/asic/frequency_transition_bmXX.c` | `crates/bitaxe-asic`, `firmware/bitaxe` | verified | hardware-smoke | Frequency transition smoke observed without a bounded frequency-transition hardware-regression artifact. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(&errors, "ASIC-007", "hardware-regression evidence");
        assert_validation_error_contains(&errors, "ASIC-007", "bounded frequency-transition");
    }

    #[test]
    fn asic007_verified_accepts_bounded_frequency_transition_hardware_regression() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| ASIC-007 | Frequency transition behavior | `reference/esp-miner/components/asic/frequency_transition_bmXX.c` | `crates/bitaxe-asic`, `firmware/bitaxe` | verified | hardware-regression | Bounded frequency-transition hardware artifact passed on Ultra 205. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert!(errors.is_empty());
    }

    #[test]
    fn asic_mining_verified_rows_require_hardware_or_soak_evidence() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| ASIC-002 | BM1366 initialization | `reference/esp-miner/components/asic/bm1366.c` | `crates/bitaxe-asic`, `firmware/bitaxe` | verified | unit,workflow | Pure init and workflow evidence only. |
| ASIC-003 | BM1366 work send | `reference/esp-miner/components/asic/bm1366.c` | `crates/bitaxe-asic` | verified | unit,golden | Diagnostic work fixture evidence only. |
| ASIC-004 | BM1366 result parsing | `reference/esp-miner/components/asic/bm1366.c` | `crates/bitaxe-asic` | verified | unit,golden | Result fixture evidence only. |
| ASIC-005 | ASIC serial transport | `reference/esp-miner/components/asic/serial.c` | `firmware/bitaxe` | verified | workflow | Firmware compile evidence only. |
| ASIC-007 | Frequency transition behavior | `reference/esp-miner/components/asic/frequency_transition_bmXX.c` | `crates/bitaxe-asic` | verified | unit | Frequency transition unit evidence only. |
| STR-006 | Protocol coordinator | `reference/esp-miner/main/tasks/protocol_coordinator.c` | `crates/bitaxe-stratum`, `firmware/bitaxe` | verified | unit,workflow | First live mining loop not observed. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        for row_id in [
            "ASIC-002", "ASIC-003", "ASIC-004", "ASIC-005", "ASIC-007", "STR-006",
        ] {
            assert_validation_error_contains(
                &errors,
                row_id,
                "requires hardware-smoke or soak evidence",
            );
        }
    }

    #[test]
    fn asic_mining_verified_str008_requires_mining_smoke_or_soak_details() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STR-008 | Live mining smoke and soak evidence | `reference/esp-miner/main/tasks/protocol_coordinator.c` | `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md` | verified | hardware-smoke | Board 205 port /dev/cu.usbmodem1101 firmware commit abc123 reference commit def456 redaction passed conclusion recorded, but no share or controlled no-share observation. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(
            &errors,
            "STR-008",
            "requires mining smoke or soak details",
        );
    }

    #[test]
    fn asic_mining_verified_str008_rejects_controlled_no_share_with_missing_live_prerequisites() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STR-008 | Live mining smoke and soak evidence | `reference/esp-miner/main/tasks/protocol_coordinator.c` | `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md` | verified | hardware-smoke | Board 205 port /dev/cu.usbmodem1101 firmware commit abc123 reference commit def456 controlled no-share condition redaction passed conclusion recorded; missing live prerequisites kept live smoke below verified. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(&errors, "STR-008", "blocker terms");
        assert_validation_error_contains(
            &errors,
            "STR-008",
            "requires mining smoke or soak details",
        );
    }

    #[test]
    fn asic_mining_verified_str008_accepts_live_share_metadata() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STR-008 | Live mining smoke and soak evidence | `reference/esp-miner/main/tasks/protocol_coordinator.c` | `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md` | verified | hardware-smoke | Board 205 port /dev/cu.usbmodem1101 firmware commit abc123 reference commit def456 accepted share observed redaction passed conclusion recorded. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert!(errors.is_empty());
    }

    #[test]
    fn asic_mining_verified_str008_accepts_approved_bounded_controlled_no_share_soak() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STR-008 | Live mining smoke and soak evidence | `reference/esp-miner/main/tasks/protocol_coordinator.c` | `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md` | verified | soak | Board 205 port /dev/cu.usbmodem1101 firmware commit abc123 reference commit def456 approved bounded controlled no-share soak redaction passed conclusion recorded. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert!(errors.is_empty());
    }

    #[test]
    fn asic_mining_verified_rows_reject_blocker_language() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STR-006 | Protocol coordinator | `reference/esp-miner/main/tasks/protocol_coordinator.c` | `crates/bitaxe-stratum`, `firmware/bitaxe` | verified | hardware-smoke | Board 205 coordination observed, but live prerequisites missing and pool lifecycle remains below verified. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(&errors, "STR-006", "blocker terms");
    }

    #[test]
    fn asic_mining_verified_str007_workflow_below_verified_remains_allowed() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STR-007 | Mining smoke and soak criteria | `reference/esp-miner/main/tasks/protocol_coordinator.c` | `docs/parity/evidence/phase-12-asic-and-mining-hardware-evidence.md` | implemented | workflow | Criteria documentation only; live smoke remains hardware evidence pending. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert!(errors.is_empty());
    }

    #[test]
    fn release_ota_verified_guard_rejects_filesystem_verified_without_live_static_recovery() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| FS-001 | SPIFFS/filesystem behavior | `reference/esp-miner/main/filesystem.c` | `firmware/bitaxe`, `tools/parity` | verified | workflow | Package evidence only. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(&errors, "FS-001", "live recovery/static smoke");
    }

    #[test]
    fn release_ota_verified_guard_rejects_firmware_ota_verified_without_hardware() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| OTA-001 | Firmware OTA route | `reference/esp-miner/main/http_server/http_server.c` | `firmware/bitaxe`, `tools/parity` | verified | workflow | Firmware OTA compile and package evidence only. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(
            &errors,
            "OTA-001",
            "hardware-smoke or hardware-regression",
        );
    }

    #[test]
    fn release_ota_verified_guard_rejects_otawww_verified_without_interrupted_update_regression() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| OTA-002 | AxeOS OTAWWW route | `reference/esp-miner/main/http_server/http_server.c` | `firmware/bitaxe`, `tools/parity` | verified | hardware-smoke | Live static update smoke only. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(&errors, "OTA-002", "interrupted-update");
        assert_validation_error_contains(&errors, "OTA-002", "hardware-regression");
    }

    #[test]
    fn release_ota_verified_guard_rejects_partition_verified_from_package_only_evidence() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| REL-001 | Partition layout | `reference/esp-miner/partitions.csv` | `firmware/bitaxe` | verified | workflow | Package evidence only. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(&errors, "REL-001", "release-sensitive");
    }

    #[test]
    fn release_ota_verified_guard_rejects_sdk_config_verified_from_unit_evidence() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| REL-002 | SDK config parity | `reference/esp-miner/sdkconfig.defaults` | `firmware/bitaxe` | verified | unit | SDK config fixture evidence only. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(&errors, "REL-002", "release-sensitive");
    }

    #[test]
    fn release_ota_verified_guard_rejects_release_image_verified_without_gate_and_package() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| REL-003 | Release image behavior | `reference/esp-miner/.github/workflows/release.yml` | `MODULE.bazel`, `tools/flash` | verified | workflow | Package workflow evidence only. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(&errors, "REL-003", "release-gate");
        assert_validation_error_contains(&errors, "REL-003", "provenance");
        assert_validation_error_contains(&errors, "REL-003", "package workflow");
    }

    #[test]
    fn release_image_verified_requires_rel08_evidence() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| REL-003 | Release image behavior | `reference/esp-miner/.github/workflows/release.yml` | `MODULE.bazel`, `tools/flash` | verified | workflow | release-gate provenance package workflow evidence is present, but only package output was reviewed. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(&errors, "REL-003", "rollback");
        assert_validation_error_contains(&errors, "REL-003", "recovery");
        assert_validation_error_contains(&errors, "REL-003", "large erase");
        assert_validation_error_contains(&errors, "REL-003", "failed update");
        assert_validation_error_contains(&errors, "REL-003", "interrupted-update");
    }

    #[test]
    fn firmware_ota_verified_requires_valid_invalid_and_boot_validation() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| OTA-001 | Firmware OTA route | `reference/esp-miner/main/http_server/http_server.c` | `firmware/bitaxe`, `tools/parity` | verified | hardware-smoke | Ultra 205 route registration and OTA compile evidence only. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(&errors, "OTA-001", "valid OTA");
        assert_validation_error_contains(&errors, "OTA-001", "invalid image rejection");
        assert_validation_error_contains(&errors, "OTA-001", "boot-validation");
    }

    #[test]
    fn filesystem_verified_requires_live_static_recovery_surfaces() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| FS-001 | SPIFFS/filesystem behavior | `reference/esp-miner/main/filesystem.c` | `firmware/bitaxe`, `tools/parity` | verified | hardware-smoke | Live recovery and live static smoke passed on Ultra 205. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(&errors, "FS-001", "/assets/app.css.gz");
        assert_validation_error_contains(&errors, "FS-001", "missing static redirect");
        assert_validation_error_contains(&errors, "FS-001", "/recovery");
    }

    #[test]
    fn release_ota_verified_guard_rejects_blocker_language_that_contains_required_terms() {
        // Arrange
        let cases = [
            (
                "FS-001",
                "SPIFFS/filesystem behavior",
                "hardware-smoke",
                "live static not run; /assets/app.css.gz blocked; missing static redirect pending; /recovery no reachable DEVICE_URL; unverified smoke.",
            ),
            (
                "OTA-001",
                "Firmware OTA route",
                "hardware-smoke",
                "valid OTA not run; invalid image rejection blocked; boot-validation pending.",
            ),
            (
                "OTA-002",
                "AxeOS OTAWWW route",
                "hardware-regression",
                "interrupted-update not run because no reachable DEVICE_URL.",
            ),
            (
                "REL-003",
                "Release image behavior",
                "workflow",
                "release-gate provenance package workflow recorded; rollback not run; recovery blocked; large erase pending; failed update unverified; interrupted-update no reachable DEVICE_URL.",
            ),
        ];

        for (id, surface, evidence, notes) in cases {
            let checklist = format!(
                r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| {id} | {surface} | reference path | rust target | verified | {evidence} | {notes} |
"#
            );
            let rows = parse_checklist(&checklist).expect("checklist should parse");

            // Act
            let errors = validate_rows(&rows);

            // Assert
            assert_validation_error_contains(&errors, id, "blocker terms");
        }
    }

    #[test]
    fn deferred_scope_verified_rows_reject_ultra205_evidence() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| CFG-002 | Deferred Gamma 601 defaults | `reference/esp-miner/config-601.cvs` | `crates/bitaxe-config` | verified | hardware-smoke | Ultra 205 evidence was reused for a non-205 board. |
| ASIC-008 | BM1370 parity | `reference/esp-miner/components/asic/bm1370.c` | `crates/bitaxe-asic` | verified | hardware-smoke | Ultra 205 evidence was reused for BM1370. |
| STR-005 | Stratum v2 protocol | `reference/esp-miner/components/stratum_v2/*.c` | `crates/bitaxe-stratum` | verified | hardware-smoke | Ultra 205 Stratum v1 evidence was reused. |
| BAP-001 | BAP interface initialization | `reference/esp-miner/main/bap/bap.c` | `firmware/bitaxe` | verified | hardware-smoke | Ultra 205 evidence was reused for BAP. |
| V2-FACTORY-001 | all-board factory image matrix | `reference/esp-miner` | `tools/xtask` | verified | hardware-smoke | Ultra 205 evidence was reused for an all-board release matrix. |
| V2-UI-001 | Angular UI rewrite | `reference/esp-miner/main/http_server/axe-os` | `firmware/bitaxe/static/www` | verified | hardware-smoke | Ultra 205 evidence was reused for an Angular rewrite. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(&errors, "CFG-002", "Ultra 205 evidence");
        assert_validation_error_contains(&errors, "ASIC-008", "Ultra 205 evidence");
        assert_validation_error_contains(&errors, "STR-005", "Ultra 205 evidence");
        assert_validation_error_contains(&errors, "BAP-001", "Ultra 205 evidence");
        assert_validation_error_contains(&errors, "V2-FACTORY-001", "Ultra 205 evidence");
        assert_validation_error_contains(&errors, "V2-UI-001", "Ultra 205 evidence");
    }

    #[test]
    fn release_ota_verified_guard_allows_implemented_package_evidence_below_verified() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| FS-001 | SPIFFS/filesystem behavior | `reference/esp-miner/main/filesystem.c` | `firmware/bitaxe`, `tools/parity` | implemented | unit,workflow | Package evidence only; live smoke pending. |
| OTA-001 | Firmware OTA route | `reference/esp-miner/main/http_server/http_server.c` | `firmware/bitaxe`, `tools/parity` | implemented | workflow | Firmware OTA compile and package evidence only. |
| REL-003 | Release image behavior | `reference/esp-miner/.github/workflows/release.yml` | `MODULE.bazel`, `tools/flash` | implemented | workflow | Release-gate and package workflow evidence exist; hardware remains pending. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert!(errors.is_empty());
    }

    #[test]
    fn phase26_verified_telemetry_row_rejects_missing_summary_evidence() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| API-002 | System info response | `reference/esp-miner/main/http_server/system_api_json.c` | `crates/bitaxe-api`, `firmware/bitaxe` | verified | workflow | Phase 26 redaction-review.md redaction_status: passed exact_non_claims no_request_time_fabrication empty_without_parsed_share_outcome. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(
            &errors,
            "API-002",
            "phase26 verified row missing summary evidence",
        );
    }

    #[test]
    fn phase26_verified_row_rejects_blocked_or_pending_language() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| API-006 | WebSocket telemetry | `reference/esp-miner/main/http_server/websocket_api.c` | `crates/bitaxe-api`, `firmware/bitaxe` | verified | workflow | phase-26-telemetry-and-parity-closure/summary.md redaction-review.md redaction_status: passed but no reachable DEVICE_URL and blocked proof remain. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(&errors, "API-006", "phase26 blocked verified row");
    }

    #[test]
    fn phase26_verified_row_rejects_missing_redaction_evidence() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STAT-002 | Statistics task | `reference/esp-miner/main/tasks/statistics_task.c` | `crates/bitaxe-api`, `firmware/bitaxe` | verified | workflow | phase-26-telemetry-and-parity-closure/summary.md no_request_time_fabrication runtime_projection_marker_only. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(&errors, "STAT-002", "phase26 redaction evidence");
    }

    #[test]
    fn phase26_verified_row_rejects_missing_exact_non_claims() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| API-006 | WebSocket telemetry | `reference/esp-miner/main/http_server/websocket_api.c` | `crates/bitaxe-api`, `firmware/bitaxe` | verified | workflow | phase-26-telemetry-and-parity-closure/summary.md redaction-review.md redaction_status: passed projection-backed telemetry closure. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(&errors, "API-006", "exact_non_claims");
    }

    #[test]
    fn phase26_guard_accepts_conservative_rows_and_evd08_closure() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| API-002 | System info response | `reference/esp-miner/main/http_server/system_api_json.c` | `crates/bitaxe-api`, `firmware/bitaxe` | implemented | unit,api-compare,workflow | phase-26-telemetry-and-parity-closure/summary.md redaction-review.md redaction_status: passed exact_non_claims projection-backed. Accepted shares remain non-claims. |
| STAT-002 | Statistics task | `reference/esp-miner/main/tasks/statistics_task.c` | `crates/bitaxe-api`, `firmware/bitaxe` | implemented | unit,workflow | phase-26-telemetry-and-parity-closure/summary.md redaction-review.md redaction_status: passed no_request_time_fabrication runtime_projection_marker_only. |
| STAT-003 | Scoreboard | `reference/esp-miner/main/tasks/scoreboard.c` | `crates/bitaxe-api` | implemented | unit,workflow | phase-26-telemetry-and-parity-closure/summary.md redaction-review.md redaction_status: passed empty_without_parsed_share_outcome exact_non_claims. |
| EVD-08 | Phase 26 exact telemetry closure | `docs/parity/evidence/phase-26-telemetry-and-parity-closure/summary.md` | `docs/parity/checklist.md`, `tools/parity/src/main.rs` | verified | workflow | API-11 API-12 API-13 EVD-08 phase-26-telemetry-and-parity-closure/summary.md redaction-review.md redaction_status: passed exact_non_claims just parity guard passed. Full active voltage and unbounded stress remain non-claims. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert!(errors.is_empty());
    }

    fn phase30_verified_row(requirement_id: &str, promotion_terms: &str) -> ChecklistRow {
        ChecklistRow {
            id: requirement_id.to_owned(),
            surface: "Phase 30 exact promotion claim".to_owned(),
            reference_breadcrumb: "reference/esp-miner/main/system.c".to_owned(),
            rust_owned_target: "tools/parity/src/main.rs".to_owned(),
            status: "verified".to_owned(),
            evidence: "workflow,hardware-smoke,hardware-regression".to_owned(),
            notes: format!(
                "phase-28-hardware-evidence-and-checklist-promotion/summary.md \
                 redaction-review.md exact_non_claims \
                 accepted share hardware proof asic bridge correlation {promotion_terms}"
            ),
        }
    }

    fn phase30_complete_promotion_artifact(requirement_id: &str) -> String {
        let row_proof = match requirement_id {
            "STR-09" => {
                "STR-09.live_submit_response_classified: true\n\
                 STR-09.asic_correlation: passed\n\
                 STR-09.safe_stop_status: complete"
            }
            "CFG-07" => {
                "CFG-07.runtime_credentials_input: local-owner-supplied\n\
                 CFG-07.live_mining_credentials_consumed: true\n\
                 CFG-07.committed_credential_values: none\n\
                 CFG-07.safe_stop_status: complete"
            }
            "ASIC-11" => {
                "ASIC-11.asic_result_to_active_work: correlated\n\
                 ASIC-11.submit_intent_from_correlated_result: true\n\
                 ASIC-11.safe_stop_status: complete"
            }
            _ => panic!("unsupported Phase 30 requirement fixture: {requirement_id}"),
        };

        format!(
            "phase30_disposition: promoted\n\
             new_evidence_input: explicit\n\
             archived_lineage_verification: gaps_found\n\
             eligible_share_outcome: accepted\n\
             hardware_accessed: true\n\
             credentials_accessed: false\n\
             raw_artifacts_committed: no\n\
             current_source_gate: passed\n\
             detector_gate: passed\n\
             same_chain_gate: passed\n\
             provenance_gate: passed\n\
             redaction_status: passed\n\
             {row_proof}\n"
        )
    }

    #[test]
    fn phase30_verified_row_rejects_current_no_promotion_artifact() {
        // Arrange
        let row = phase30_verified_row("STR-09", DEFAULT_PHASE30_PROMOTION_ARTIFACT_PATH);
        let artifact = parse_phase30_promotion_artifact(include_str!(
            "../../../docs/parity/evidence/phase-30-live-share-outcome-and-verified-promotion/conclusion.md"
        ))
        .expect("committed Phase 30 conclusion should parse");

        // Act
        let errors = validate_rows_with_phase30_artifact(
            &[row],
            &Phase30PromotionArtifactState::Available(artifact),
        );

        // Assert
        assert_validation_error_contains(&errors, "STR-09", "no_promotion_no_eligible_evidence");
    }

    #[test]
    fn phase30_report_rejects_verified_row_against_committed_no_promotion_artifact() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STR-09 | Phase 30 exact promotion claim | `reference/esp-miner/main/system.c` | `tools/parity/src/main.rs` | verified | workflow,hardware-smoke,hardware-regression | phase-28-hardware-evidence-and-checklist-promotion/summary.md redaction-review.md exact_non_claims accepted share hardware proof asic bridge correlation docs/parity/evidence/phase-30-live-share-outcome-and-verified-promotion/conclusion.md |
"#;
        let environment = FakeEnvironment::with_documents(
            checklist,
            include_str!(
                "../../../docs/parity/evidence/phase-30-live-share-outcome-and-verified-promotion/conclusion.md"
            ),
        );
        let request = ReportRequest {
            checklist: Utf8PathBuf::from("docs/parity/checklist.md"),
            format: ReportFormat::Text,
            fail_on_invalid_verified: true,
        };

        // Act
        let result = run_report(&request, &environment);

        // Assert
        assert!(result.is_err());
        assert!(result
            .expect_err("current no-promotion artifact must reject verified row")
            .to_string()
            .contains("no_promotion_no_eligible_evidence"));
    }

    #[test]
    fn phase30_verified_rows_accept_matching_structured_artifacts() {
        // Arrange
        let cases = ["STR-09", "CFG-07", "ASIC-11"];

        // Act
        let results = cases.map(|requirement_id| {
            let artifact = parse_phase30_promotion_artifact(&phase30_complete_promotion_artifact(
                requirement_id,
            ))
            .expect("complete Phase 30 promotion artifact should parse");
            let errors = validate_rows_with_phase30_artifact(
                &[phase30_verified_row(
                    requirement_id,
                    DEFAULT_PHASE30_PROMOTION_ARTIFACT_PATH,
                )],
                &Phase30PromotionArtifactState::Available(artifact),
            );
            (requirement_id, errors)
        });

        // Assert
        for (requirement_id, errors) in results {
            assert!(
                errors.is_empty(),
                "expected structured artifact for {requirement_id} to pass, got {errors:#?}"
            );
        }
    }

    #[test]
    fn phase30_verified_row_rejects_missing_artifact() {
        // Arrange
        let row = phase30_verified_row("STR-09", DEFAULT_PHASE30_PROMOTION_ARTIFACT_PATH);
        let artifact = Phase30PromotionArtifactState::Unavailable(
            "structured Phase 30 evidence artifact is missing".to_owned(),
        );

        // Act
        let errors = validate_rows_with_phase30_artifact(&[row], &artifact);

        // Assert
        assert_validation_error_contains(&errors, "STR-09", "artifact is missing");
    }

    #[test]
    fn phase30_verified_row_rejects_malformed_artifact_value() {
        // Arrange
        let row = phase30_verified_row("STR-09", DEFAULT_PHASE30_PROMOTION_ARTIFACT_PATH);
        let malformed = phase30_complete_promotion_artifact("STR-09")
            .replace("detector_gate: passed", "detector_gate: maybe");
        let artifact = parse_phase30_promotion_artifact(&malformed)
            .expect_err("invalid closed value must fail parsing");

        // Act
        let errors = validate_rows_with_phase30_artifact(
            &[row],
            &Phase30PromotionArtifactState::Malformed(artifact),
        );

        // Assert
        assert_validation_error_contains(&errors, "STR-09", "detector_gate");
    }

    #[test]
    fn phase30_verified_row_rejects_mismatched_artifact_bundle() {
        // Arrange
        let row = phase30_verified_row("STR-09", DEFAULT_PHASE30_PROMOTION_ARTIFACT_PATH);
        let artifact =
            parse_phase30_promotion_artifact(&phase30_complete_promotion_artifact("CFG-07"))
                .expect("complete CFG-07 fixture should parse");

        // Act
        let errors = validate_rows_with_phase30_artifact(
            &[row],
            &Phase30PromotionArtifactState::Available(artifact),
        );

        // Assert
        assert_validation_error_contains(
            &errors,
            "STR-09",
            "STR-09.live_submit_response_classified",
        );
    }

    #[test]
    fn phase30_committed_conservative_rows_remain_valid() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STR-09 | Live submit response classification or blocker | `reference/esp-miner/main/system.c` | `crates/bitaxe-stratum` | implemented | unit,workflow | phase-30-live-share-outcome-and-verified-promotion/disposition.md phase30_disposition: no_promotion_no_eligible_evidence below verified. |
| CFG-07 | Runtime-only credential labels | `reference/esp-miner/main/nvs_config.c` | `scripts/phase23-redacted-operator-evidence.sh` | implemented | workflow | phase-30-live-share-outcome-and-verified-promotion/disposition.md phase30_disposition: no_promotion_no_eligible_evidence below verified. |
| ASIC-11 | BM1366 result correlation before submit intent | `reference/esp-miner/components/asic/bm1366.c` | `crates/bitaxe-stratum` | implemented | unit,workflow | phase-30-live-share-outcome-and-verified-promotion/disposition.md phase30_disposition: no_promotion_no_eligible_evidence below verified. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert!(errors.is_empty());
    }

    #[test]
    fn phase28_verified_str09_rejects_missing_summary_evidence() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STR-09 | Live submit response classification or blocker | `reference/esp-miner/main/system.c` | `crates/bitaxe-stratum` | verified | unit,workflow | redaction-review.md redaction_status: passed exact_non_claims accepted share hardware proof. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(
            &errors,
            "STR-09",
            "phase28 verified row missing summary evidence",
        );
    }

    #[test]
    fn phase28_verified_str09_rejects_blocked_safe_prerequisite() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STR-09 | Live submit response classification or blocker | `reference/esp-miner/main/system.c` | `crates/bitaxe-stratum` | verified | unit,workflow | phase-28-hardware-evidence-and-checklist-promotion/summary.md redaction-review.md redaction_status: passed exact_non_claims share_outcome: blocked_safe_prerequisite. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(&errors, "STR-09", "blocked_safe_prerequisite");
    }

    #[test]
    fn phase28_verified_cfg07_rejects_verified_status() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| CFG-07 | Runtime-only credential labels | `reference/esp-miner/main/nvs_config.c` | `scripts/phase23-redacted-operator-evidence.sh` | verified | workflow | phase-28-hardware-evidence-and-checklist-promotion/summary.md redaction-review.md redaction_status: passed exact_non_claims pool_config: local-owner-supplied. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(&errors, "CFG-07", "CFG-07 must remain below verified");
    }

    #[test]
    fn phase28_verified_safe10_rejects_without_live_safety_hardware_proof() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| SAFE-10 | Production mining prerequisite readiness | `reference/esp-miner/main/tasks/protocol_coordinator.c` | `crates/bitaxe-safety` | verified | unit,workflow | phase-28-hardware-evidence-and-checklist-promotion/summary.md redaction-review.md redaction_status: passed exact_non_claims consolidation only. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(
            &errors,
            "SAFE-10",
            "detector-gated live safety hardware proof",
        );
    }

    #[test]
    fn phase28_verified_row_rejects_missing_redaction_evidence() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| ASIC-09 | BM1366 diagnostic and production mode separation | `reference/esp-miner/components/asic/bm1366.c` | `crates/bitaxe-asic` | verified | unit,workflow | phase-28-hardware-evidence-and-checklist-promotion/summary.md exact_non_claims live socket success hardware-regression asic bridge correlation. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(&errors, "ASIC-09", "phase28 redaction evidence");
    }

    #[test]
    fn phase28_verified_row_rejects_missing_exact_non_claims() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| ASIC-10 | Pool-derived BM1366 production work registry | `reference/esp-miner/components/stratum/mining.c` | `crates/bitaxe-stratum` | verified | unit,workflow | phase-28-hardware-evidence-and-checklist-promotion/summary.md redaction-review.md redaction_status: passed live socket success hardware-regression asic bridge correlation. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert_validation_error_contains(&errors, "ASIC-10", "exact_non_claims");
    }

    #[test]
    fn phase28_guard_accepts_conservative_rows() {
        // Arrange
        let checklist = r#"
| ID | Surface | Reference Breadcrumb | Rust-Owned Target | Status | Evidence | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| STR-09 | Live submit response classification or blocker | `reference/esp-miner/main/system.c` | `crates/bitaxe-stratum` | implemented | unit,workflow | phase-28-hardware-evidence-and-checklist-promotion/summary.md phase-27-live-hardware-asic-and-stratum-bridge/share-outcome.md redaction-review.md redaction_status: passed exact_non_claims share_outcome: blocked_safe_prerequisite below verified. |
| CFG-07 | Runtime-only credential labels | `reference/esp-miner/main/nvs_config.c` | `scripts/phase23-redacted-operator-evidence.sh` | implemented | workflow | phase-28-hardware-evidence-and-checklist-promotion/summary.md redaction-review.md redaction_status: passed exact_non_claims below verified category labels only. |
| SAFE-10 | Production mining prerequisite readiness | `reference/esp-miner/main/tasks/protocol_coordinator.c` | `crates/bitaxe-safety` | implemented | unit,workflow | phase-28-hardware-evidence-and-checklist-promotion/summary.md phase-22-claim-ladder-and-safety-preconditions/safety-preconditions.md exact_non_claims below verified. |
"#;
        let rows = parse_checklist(checklist).expect("checklist should parse");

        // Act
        let errors = validate_rows(&rows);

        // Assert
        assert!(errors.is_empty());
    }

    #[test]
    fn missing_reference_guard_failure_blocks_report_output() {
        // Arrange
        let env = FakeEnvironment::failing_guard("reference missing or not initialized");
        let request = ReportRequest {
            checklist: Utf8PathBuf::from("docs/parity/checklist.md"),
            format: ReportFormat::Text,
            fail_on_invalid_verified: true,
        };

        // Act
        let result = run_report(&request, &env);

        // Assert
        assert!(result.is_err());
        assert!(result
            .expect_err("report should fail")
            .to_string()
            .contains("reference missing"));
        assert!(!env.read_called.get());
    }

    #[test]
    fn dirty_reference_guard_failure_blocks_report_output() {
        // Arrange
        let env = FakeEnvironment::failing_guard("reference dirty");
        let request = ReportRequest {
            checklist: Utf8PathBuf::from("docs/parity/checklist.md"),
            format: ReportFormat::Text,
            fail_on_invalid_verified: true,
        };

        // Act
        let result = run_report(&request, &env);

        // Assert
        assert!(result.is_err());
        assert!(result
            .expect_err("report should fail")
            .to_string()
            .contains("reference dirty"));
        assert!(!env.read_called.get());
    }

    fn assert_validation_error_contains(
        errors: &[ValidationError],
        expected_id: &str,
        expected_message: &str,
    ) {
        assert!(
            errors.iter().any(|error| {
                error.id == expected_id && error.message.contains(expected_message)
            }),
            "expected {expected_id} validation error containing {expected_message:?}, got {errors:#?}"
        );
    }

    struct FakeEnvironment {
        maybe_guard_error: Option<&'static str>,
        maybe_checklist: Option<&'static str>,
        maybe_phase30_artifact: Option<&'static str>,
        read_called: Cell<bool>,
    }

    impl FakeEnvironment {
        fn failing_guard(message: &'static str) -> Self {
            Self {
                maybe_guard_error: Some(message),
                maybe_checklist: None,
                maybe_phase30_artifact: None,
                read_called: Cell::new(false),
            }
        }

        fn with_documents(checklist: &'static str, phase30_artifact: &'static str) -> Self {
            Self {
                maybe_guard_error: None,
                maybe_checklist: Some(checklist),
                maybe_phase30_artifact: Some(phase30_artifact),
                read_called: Cell::new(false),
            }
        }
    }

    impl ReportEnvironment for FakeEnvironment {
        fn run_reference_guard(&self) -> Result<()> {
            if let Some(message) = self.maybe_guard_error {
                return Err(anyhow!(message));
            }

            Ok(())
        }

        fn read_checklist(&self, _path: &Utf8Path) -> Result<String> {
            self.read_called.set(true);
            Ok(self.maybe_checklist.unwrap_or(CHECKLIST).to_owned())
        }

        fn read_phase30_promotion_artifact(&self, _path: &Utf8Path) -> Result<String> {
            let Some(document) = self.maybe_phase30_artifact else {
                bail!("structured Phase 30 evidence artifact is missing");
            };
            Ok(document.to_owned())
        }

        fn reference_commit(&self) -> Result<String> {
            Ok("abc123".to_owned())
        }
    }
}
