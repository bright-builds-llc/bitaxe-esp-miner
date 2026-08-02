use crate::*;

#[derive(Debug, Parser)]
#[command(name = "bitaxe-parity")]
#[command(about = "Report Bitaxe parity checklist status and evidence gaps.")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: CliCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum CliCommand {
    Report(ReportArgs),
    NextItem(NextItemArgs),
    Progress(ProgressArgs),
    SyncProgress(SyncProgressArgs),
    TransitionItem(TransitionItemArgs),
    ReviseChecklistDocumentation(ReviseChecklistDocumentationArgs),
    ApiCompare(ApiCompareArgs),
    ReleaseGate(ReleaseGateArgs),
    ReleaseEvidence(ReleaseEvidenceArgs),
    SafetyAllow(SafetyAllowArgs),
    MiningAllow(MiningAllowArgs),
    OperatorEvidence(OperatorEvidenceArgs),
    Phase33Classify(Phase33ClassifyArgs),
    ClassifyPhase35Flash(ClassifyPhase35FlashArgs),
    ClassifyPhase35Http(ClassifyPhase35HttpArgs),
    ProbePhase35Http(ProbePhase35HttpArgs),
    ValidatePhase35Evidence(ValidatePhase35EvidenceArgs),
    AdmitPhase35Evidence(AdmitPhase35EvidenceArgs),
    ClassifyPhase36Evidence(ClassifyPhase36EvidenceArgs),
    ClassifyPhase36Effects(ClassifyPhase36EffectsArgs),
    Phase36EvaluatorIdentity,
    Phase36AssembleHardwareCapture(Phase36AssembleHardwareCaptureArgs),
    Phase36HardwareCapture(Phase36HardwareCaptureArgs),
    Phase36SyntheticCapture(Phase36SyntheticCaptureArgs),
    InspectPhase36Candidate(InspectPhase36CandidateArgs),
    ClassifyPhase36Candidate(ClassifyPhase36CandidateArgs),
    ReevaluatePhase36Attempt31(ReevaluatePhase36Attempt31Args),
}

#[derive(Debug, Parser)]
pub(crate) struct NextItemArgs {
    #[arg(long, value_enum, default_value_t = ReportFormat::Text)]
    pub(crate) format: ReportFormat,
}

#[derive(Debug, Parser)]
pub(crate) struct ProgressArgs {
    #[arg(long, value_enum, default_value_t = ReportFormat::Text)]
    pub(crate) format: ReportFormat,
}

#[derive(Debug, Parser)]
pub(crate) struct SyncProgressArgs {
    #[arg(long)]
    pub(crate) source_commit: String,

    #[arg(long = "selected-row")]
    pub(crate) maybe_selected_row: Option<String>,

    #[arg(long = "plan", value_parser = parse_utf8_path)]
    pub(crate) maybe_plan: Option<Utf8PathBuf>,
}

#[derive(Debug, Parser)]
pub(crate) struct TransitionItemArgs {
    #[arg(long)]
    pub(crate) transition_id: String,

    #[arg(long)]
    pub(crate) row_id: String,

    #[arg(long)]
    pub(crate) to: String,

    #[arg(long)]
    pub(crate) evidence: String,

    #[arg(long = "rust-owned-target")]
    pub(crate) maybe_rust_owned_target: Option<String>,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) plan: Utf8PathBuf,

    #[arg(long = "result", value_parser = parse_utf8_path)]
    pub(crate) maybe_result: Option<Utf8PathBuf>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub(crate) enum Phase33ClassifyMode {
    Baseline,
    Delivery,
    PostRestart,
}

#[derive(Debug, Parser)]
pub(crate) struct Phase33ClassifyArgs {
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) trace: Utf8PathBuf,

    #[arg(long, value_enum)]
    pub(crate) mode: Phase33ClassifyMode,

    #[arg(long, default_value_t = 0)]
    pub(crate) start_byte: u64,

    #[arg(long)]
    pub(crate) expected_session: Option<String>,

    #[arg(long)]
    pub(crate) expected_ordinal: Option<u64>,
}

#[derive(Debug, Parser)]
pub(crate) struct ClassifyPhase35HttpArgs {
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) metrics_input: Utf8PathBuf,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) body_input: Utf8PathBuf,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) projection_output: Utf8PathBuf,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) hostname_output: Utf8PathBuf,
}

#[derive(Debug, Parser)]
pub(crate) struct ClassifyPhase35FlashArgs {
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) metrics_input: Utf8PathBuf,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) private_log_input: Utf8PathBuf,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) projection_output: Utf8PathBuf,
}

#[derive(Debug, Parser)]
pub(crate) struct ProbePhase35HttpArgs {
    #[arg(long)]
    pub(crate) url: String,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) metrics_output: Utf8PathBuf,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) headers_output: Utf8PathBuf,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) body_output: Utf8PathBuf,
}

#[derive(Debug, Parser)]
pub(crate) struct ValidatePhase35EvidenceArgs {
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) root: Utf8PathBuf,
}

#[derive(Debug, Parser)]
pub(crate) struct AdmitPhase35EvidenceArgs {
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) root: Utf8PathBuf,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) staging: Utf8PathBuf,
}

#[derive(Debug, Parser)]
pub(crate) struct ClassifyPhase36EvidenceArgs {
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) root: Utf8PathBuf,
}

#[derive(Debug, Parser)]
pub(crate) struct ClassifyPhase36EffectsArgs {
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) root: Utf8PathBuf,
}

#[derive(Debug, Parser)]
pub(crate) struct Phase36SyntheticCaptureArgs {
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) private_output: Utf8PathBuf,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) candidate_output: Utf8PathBuf,

    #[arg(long)]
    pub(crate) capability_digest: String,
}

#[derive(Debug, Parser)]
pub(crate) struct Phase36HardwareCaptureArgs {
    #[arg(long)]
    pub(crate) board: u16,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) private_parent: Utf8PathBuf,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) attempt_handle_file: Utf8PathBuf,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) candidate_output: Utf8PathBuf,

    #[arg(long)]
    pub(crate) capture_timeout_seconds: u64,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) wifi_credentials: Utf8PathBuf,
}

#[derive(Debug, Parser)]
pub(crate) struct Phase36AssembleHardwareCaptureArgs {
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) attempt_child: Utf8PathBuf,
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) manifest: Utf8PathBuf,
    #[arg(long)]
    pub(crate) manifest_digest: String,
    #[arg(long)]
    pub(crate) firmware_elf_digest: String,
    #[arg(long)]
    pub(crate) executable_image_digest: String,
    #[arg(long)]
    pub(crate) factory_image_digest: String,
    #[arg(long)]
    pub(crate) package_identity_digest: String,
}

#[derive(Debug, Parser)]
pub(crate) struct InspectPhase36CandidateArgs {
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) candidate_input: Utf8PathBuf,
}

#[derive(Debug, Parser)]
pub(crate) struct ClassifyPhase36CandidateArgs {
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) private_input: Utf8PathBuf,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) candidate_input: Utf8PathBuf,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) classification_output: Utf8PathBuf,
}

#[derive(Debug, Parser)]
pub(crate) struct ReevaluatePhase36Attempt31Args {
    #[arg(long, default_value = ".", value_parser = parse_utf8_path)]
    pub(crate) workspace_root: Utf8PathBuf,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) maybe_protected_root: Option<Utf8PathBuf>,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) maybe_api_document: Option<Utf8PathBuf>,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) maybe_websocket_document: Option<Utf8PathBuf>,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) maybe_retained_document: Option<Utf8PathBuf>,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) maybe_exact_package_document: Option<Utf8PathBuf>,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) maybe_request_document: Option<Utf8PathBuf>,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) maybe_event_ledger_document: Option<Utf8PathBuf>,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) maybe_private_result_document: Option<Utf8PathBuf>,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) maybe_public_projection_document: Option<Utf8PathBuf>,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) maybe_independent_effect_document: Option<Utf8PathBuf>,
}

#[derive(Debug, Parser)]
pub(crate) struct ReportArgs {
    #[arg(long, default_value = "docs/parity/checklist.md", value_parser = parse_utf8_path)]
    pub(crate) checklist: Utf8PathBuf,

    #[arg(long, value_enum, default_value_t = ReportFormat::Text)]
    pub(crate) format: ReportFormat,

    #[arg(long = "fail-on-invalid-verified")]
    pub(crate) fail_on_invalid_verified: bool,
}

#[derive(Debug, Parser)]
pub(crate) struct ReviseChecklistDocumentationArgs {
    #[arg(
        long,
        default_value = checklist_revision::CURRENT_REVISION_SPEC,
        value_parser = parse_utf8_path
    )]
    pub(crate) change_spec: Utf8PathBuf,
}

#[derive(Debug, Parser)]
pub(crate) struct ApiCompareArgs {
    #[arg(long, default_value = DEFAULT_OPENAPI_PATH, value_parser = parse_utf8_path)]
    pub(crate) openapi: Utf8PathBuf,

    #[arg(long, default_value = DEFAULT_API_COMPARE_MANIFEST, value_parser = parse_utf8_path)]
    pub(crate) route_manifest: Utf8PathBuf,

    #[arg(long, default_value = DEFAULT_AXEOS_ROUTE_USAGE, value_parser = parse_utf8_path)]
    pub(crate) static_usage: Utf8PathBuf,
}

#[derive(Debug, Parser)]
pub(crate) struct ReleaseGateArgs {
    #[arg(long, default_value = DEFAULT_LICENSE_INVENTORY_PATH, value_parser = parse_utf8_path)]
    pub(crate) license_inventory: Utf8PathBuf,

    #[arg(long, default_value = DEFAULT_PROVENANCE_PATH, value_parser = parse_utf8_path)]
    pub(crate) provenance: Utf8PathBuf,

    #[arg(long, default_value = DEFAULT_CARGO_ABOUT_PATH, value_parser = parse_utf8_path)]
    pub(crate) cargo_about: Utf8PathBuf,

    #[arg(long, value_name = "package-json", value_parser = parse_utf8_path)]
    pub(crate) manifest: Option<Utf8PathBuf>,
}

#[derive(Debug, Parser)]
pub(crate) struct ReleaseEvidenceArgs {
    #[arg(long, value_name = "package-json", value_parser = parse_utf8_path)]
    pub(crate) manifest: Utf8PathBuf,

    #[arg(long = "evidence-root", value_parser = parse_utf8_path)]
    pub(crate) evidence_root: Utf8PathBuf,

    #[arg(long = "flash-evidence-json", value_parser = parse_utf8_path)]
    pub(crate) maybe_flash_evidence_json: Option<Utf8PathBuf>,

    #[arg(long = "redaction-review", value_parser = parse_utf8_path)]
    pub(crate) maybe_redaction_review: Option<Utf8PathBuf>,

    #[arg(long = "require-redaction-passed")]
    pub(crate) require_redaction_passed: bool,

    #[arg(long = "allow-post-source-evidence-commits")]
    pub(crate) allow_post_source_evidence_commits: bool,
}

#[derive(Debug, Parser)]
pub(crate) struct SafetyAllowArgs {
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) manifest: Utf8PathBuf,

    #[arg(long = "surface")]
    pub(crate) maybe_surface: Option<String>,

    #[arg(long = "allowed-command")]
    pub(crate) maybe_allowed_command: Option<String>,
}

#[derive(Debug, Parser)]
pub(crate) struct MiningAllowArgs {
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) manifest: Utf8PathBuf,

    #[arg(long = "surface")]
    pub(crate) maybe_surface: Option<String>,

    #[arg(long = "allowed-command")]
    pub(crate) maybe_allowed_command: Option<String>,
}

#[derive(Debug, Parser)]
pub(crate) struct OperatorEvidenceArgs {
    #[arg(long, value_enum)]
    pub(crate) profile: OperatorEvidenceProfile,

    #[arg(long = "evidence-root", value_parser = parse_utf8_path)]
    pub(crate) evidence_root: Utf8PathBuf,

    #[arg(long = "require-redaction-passed")]
    pub(crate) require_redaction_passed: bool,

    #[arg(long = "require-operator-snapshot-coherence")]
    pub(crate) require_operator_snapshot_coherence: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum ReportFormat {
    Text,
    Json,
}

#[derive(Debug)]
pub(crate) struct ReportRequest {
    pub(crate) checklist: Utf8PathBuf,
    pub(crate) format: ReportFormat,
    pub(crate) fail_on_invalid_verified: bool,
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
