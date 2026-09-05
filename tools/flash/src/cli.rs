use crate::*;

mod rom_exit;
pub(crate) use rom_exit::{RomExitDiagnosticCommand, StartInstalledCommand};

#[derive(Debug, Parser)]
#[command(name = "bitaxe-flash")]
#[command(about = "Safe Bitaxe Ultra 205 flash and monitor workflow.")]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: CliCommand,
}

#[derive(Debug, Subcommand)]
pub(crate) enum CliCommand {
    Detect(DetectCommand),
    Flash(FlashCommand),
    Monitor(MonitorCommand),
    #[command(name = "flash-monitor")]
    FlashMonitor(FlashMonitorCommand),
    #[command(name = "finalize-evidence")]
    FinalizeEvidence(FinalizeEvidenceCommand),
    #[command(name = "mining-campaign")]
    MiningCampaign(MiningCampaignCommand),
    #[command(name = "input-uat")]
    InputUat(InputUatCommand),
    #[command(name = "signal-identify", alias = "confirm-identify")]
    SignalIdentify(SignalIdentifyCommand),
    #[command(name = "phase35-probe")]
    Phase35Probe(Phase35ProbeCommand),
    #[command(name = "rel003-large-erase")]
    ReleaseRecovery(ReleaseRecoveryCommand),
    #[command(name = "restore-installed")]
    RestoreInstalled(RestoreInstalledCommand),
    #[command(name = "noise-auth-diagnostic")]
    NoiseDiagnostic(NoiseDiagnosticCommand),
    #[command(name = "tcp-payload-diagnostic")]
    TcpPayloadDiagnostic(TcpPayloadDiagnosticCommand),
    #[command(name = "display-recovery-start")]
    DisplayRecoveryStart(DisplayRecoveryStartCommand),
    #[command(name = "nvs-readback")]
    NvsReadback(NvsReadbackCommand),
    #[command(name = "nvs-runtime-restore")]
    NvsRuntimeRestore(NvsRuntimeRestoreCommand),
    #[command(name = "rom-exit-diagnostic")]
    RomExitDiagnostic(RomExitDiagnosticCommand),
    #[command(name = "owner-recovery")]
    OwnerRecovery(OwnerRecoveryCommand),
    #[command(name = "boot-chain-readback")]
    BootChainReadback(BootChainReadbackCommand),
    #[command(name = "usb-stability-read")]
    UsbStabilityRead(UsbStabilityReadCommand),
    #[command(name = "native-usb-start-installed")]
    StartInstalled(StartInstalledCommand),
}

#[derive(Debug, Parser, Clone)]
pub(crate) struct UsbStabilityReadCommand {
    #[arg(long, default_value = "205", value_parser = parse_board)]
    pub(crate) board: BoardId,
    #[arg(long)]
    pub(crate) port: String,
    #[arg(long = "restore-bundle", value_parser = parse_utf8_path)]
    pub(crate) restore_bundle: Utf8PathBuf,
    #[arg(long = "private-root", value_parser = parse_utf8_path)]
    pub(crate) private_root: Utf8PathBuf,
    #[arg(long = "chunk-bytes", default_value_t = 65_536)]
    pub(crate) chunk_bytes: u32,
    #[arg(long, default_value_t = 4)]
    pub(crate) repetitions: u8,
    #[arg(long, value_enum, default_value = "repeated")]
    pub(crate) pattern: UsbStabilityPattern,
    #[arg(long = "redact-evidence")]
    pub(crate) redact_evidence: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum UsbStabilityPattern {
    Repeated,
    Sequential,
}

#[derive(Debug, Parser, Clone)]
pub(crate) struct BootChainReadbackCommand {
    #[arg(long, default_value = "205", value_parser = parse_board)]
    pub(crate) board: BoardId,
    #[arg(long)]
    pub(crate) port: String,
    #[arg(long = "package-manifest", value_parser = parse_utf8_path)]
    pub(crate) package_manifest: Utf8PathBuf,
    #[arg(long = "restore-bundle", value_parser = parse_utf8_path)]
    pub(crate) restore_bundle: Utf8PathBuf,
    #[arg(long = "private-root", value_parser = parse_utf8_path)]
    pub(crate) private_root: Utf8PathBuf,
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) plan: Utf8PathBuf,
    #[arg(long = "manual-checkpoint", value_parser = parse_utf8_path)]
    pub(crate) manual_checkpoint: Utf8PathBuf,
    #[arg(long = "redact-evidence")]
    pub(crate) redact_evidence: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub(crate) enum OwnerRecoveryAction {
    Observe,
    Recover,
}

#[derive(Debug, Parser, Clone)]
pub(crate) struct OwnerRecoveryCommand {
    #[arg(long, default_value = "205", value_parser = parse_board)]
    pub(crate) board: BoardId,
    #[arg(long)]
    pub(crate) port: String,
    #[arg(long, value_enum)]
    pub(crate) action: OwnerRecoveryAction,
    #[arg(long = "package-manifest", value_parser = parse_utf8_path)]
    pub(crate) package_manifest: Utf8PathBuf,
    #[arg(long = "restore-bundle", value_parser = parse_utf8_path)]
    pub(crate) restore_bundle: Utf8PathBuf,
    #[arg(long = "private-root", value_parser = parse_utf8_path)]
    pub(crate) private_root: Utf8PathBuf,
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) plan: Utf8PathBuf,
    #[arg(long = "manual-checkpoint", value_parser = parse_utf8_path)]
    pub(crate) manual_checkpoint: Option<Utf8PathBuf>,
    #[arg(long = "redact-evidence")]
    pub(crate) redact_evidence: bool,
}

#[derive(Debug, Parser, Clone)]
pub(crate) struct TcpPayloadDiagnosticCommand {
    #[arg(long, default_value = "205", value_parser = parse_board)]
    pub(crate) board: BoardId,
    #[arg(long)]
    pub(crate) port: String,
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) manifest: Utf8PathBuf,
    #[arg(long = "wifi-credentials", value_parser = parse_utf8_path)]
    pub(crate) wifi_credentials: Utf8PathBuf,
    #[arg(long = "pool-credentials", value_parser = parse_utf8_path)]
    pub(crate) pool_credentials: Utf8PathBuf,
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) intent: Utf8PathBuf,
    #[arg(long = "capture-timeout-seconds", default_value_t = 360)]
    pub(crate) capture_timeout_seconds: u64,
    #[arg(long = "redact-evidence")]
    pub(crate) redact_evidence: bool,
}

#[derive(Debug, Parser, Clone)]
pub(crate) struct NoiseDiagnosticCommand {
    #[arg(long, default_value = "205", value_parser = parse_board)]
    pub(crate) board: BoardId,

    #[arg(long)]
    pub(crate) port: String,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) manifest: Utf8PathBuf,

    #[arg(long = "wifi-credentials", value_parser = parse_utf8_path)]
    pub(crate) wifi_credentials: Utf8PathBuf,

    #[arg(long = "pool-credentials", value_parser = parse_utf8_path)]
    pub(crate) pool_credentials: Utf8PathBuf,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) intent: Utf8PathBuf,

    #[arg(long = "capture-timeout-seconds", default_value_t = 120)]
    pub(crate) capture_timeout_seconds: u64,

    #[arg(long = "redact-evidence")]
    pub(crate) redact_evidence: bool,
}

#[derive(Debug, Parser, Clone)]
pub(crate) struct RestoreInstalledCommand {
    #[arg(long, default_value = "205", value_parser = parse_board)]
    pub(crate) board: BoardId,

    #[arg(long)]
    pub(crate) port: String,

    #[arg(long = "restore-bundle", value_parser = parse_utf8_path)]
    pub(crate) restore_bundle: Utf8PathBuf,

    #[arg(long = "restore-authorization", value_parser = parse_utf8_path)]
    pub(crate) restore_authorization: Utf8PathBuf,

    #[arg(long = "remediation-plan", value_parser = parse_utf8_path)]
    pub(crate) remediation_plan: Utf8PathBuf,

    #[arg(long = "private-root", value_parser = parse_utf8_path)]
    pub(crate) private_root: Utf8PathBuf,

    #[arg(long = "wifi-credentials", value_parser = parse_utf8_path)]
    pub(crate) wifi_credentials: Utf8PathBuf,

    #[arg(long = "redact-evidence")]
    pub(crate) redact_evidence: bool,

    #[arg(long = "admission-only")]
    pub(crate) admission_only: bool,
}

#[derive(Debug, Parser, Clone)]
pub(crate) struct ReleaseRecoveryCommand {
    #[arg(long, default_value = "205", value_parser = parse_board)]
    pub(crate) board: BoardId,

    #[arg(long = "private-root", value_parser = parse_utf8_path)]
    pub(crate) private_root: Utf8PathBuf,

    #[arg(long = "package-manifest", value_parser = parse_utf8_path)]
    pub(crate) package_manifest: Utf8PathBuf,

    #[arg(long = "wifi-credentials", value_parser = parse_utf8_path)]
    pub(crate) wifi_credentials: Utf8PathBuf,

    #[arg(long = "detector-output", value_parser = parse_utf8_path)]
    pub(crate) detector_output: Utf8PathBuf,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) plan: Utf8PathBuf,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) projection: Utf8PathBuf,

    #[arg(long = "capture-timeout-seconds", default_value_t = DEFAULT_MONITOR_CAPTURE_TIMEOUT_SECONDS)]
    pub(crate) capture_timeout_seconds: u64,
}

#[derive(Debug, Parser, Clone)]
pub(crate) struct InputUatCommand {
    #[arg(long, default_value = "205", value_parser = parse_board)]
    pub(crate) board: BoardId,

    #[arg(long)]
    pub(crate) port: String,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) manifest: Utf8PathBuf,

    #[arg(long = "private-root", value_parser = parse_utf8_path)]
    pub(crate) private_root: Utf8PathBuf,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) plan: Utf8PathBuf,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) projection: Utf8PathBuf,
}

#[derive(Debug, Parser, Clone)]
pub(crate) struct SignalIdentifyCommand {
    #[arg(long = "evidence-dir", value_parser = parse_utf8_path)]
    pub(crate) evidence_dir: Utf8PathBuf,

    #[arg(long, value_enum)]
    pub(crate) checkpoint: network::IdentifyCheckpointKind,

    #[arg(long, value_enum, default_value = "confirmed")]
    pub(crate) outcome: network::IdentifyCheckpointOutcome,
}

#[derive(Debug, Parser, Clone)]
pub(crate) struct DetectCommand {
    #[arg(long, default_value = "205", value_parser = parse_board)]
    pub(crate) board: BoardId,

    #[arg(long)]
    pub(crate) port: Option<String>,

    #[arg(long = "retain-rom")]
    pub(crate) retain_rom: bool,
}

#[derive(Debug, Parser, Clone)]
pub(crate) struct DisplayRecoveryStartCommand {
    #[arg(long, default_value = "205", value_parser = parse_board)]
    pub(crate) board: BoardId,
    #[arg(long)]
    pub(crate) port: String,
    #[arg(long = "package-manifest", value_parser = parse_utf8_path)]
    pub(crate) package_manifest: Utf8PathBuf,
    #[arg(long = "restore-bundle", value_parser = parse_utf8_path)]
    pub(crate) restore_bundle: Utf8PathBuf,
    #[arg(long = "settings-backup", value_parser = parse_utf8_path)]
    pub(crate) settings_backup: Utf8PathBuf,
    #[arg(long = "wifi-credentials", value_parser = parse_utf8_path)]
    pub(crate) wifi_credentials: Utf8PathBuf,
    #[arg(long = "pool-credentials", value_parser = parse_utf8_path)]
    pub(crate) pool_credentials: Utf8PathBuf,
    #[arg(long = "capture-input", value_parser = parse_utf8_path)]
    pub(crate) capture_input: Utf8PathBuf,
    #[arg(long = "private-root", value_parser = parse_utf8_path)]
    pub(crate) private_root: Utf8PathBuf,
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) plan: Utf8PathBuf,
    #[arg(long = "redact-evidence")]
    pub(crate) redact_evidence: bool,
}

#[derive(Debug, Parser, Clone)]
pub(crate) struct NvsReadbackCommand {
    #[arg(long, default_value = "205", value_parser = parse_board)]
    pub(crate) board: BoardId,
    #[arg(long)]
    pub(crate) port: String,
    #[arg(long = "wifi-credentials", value_parser = parse_utf8_path)]
    pub(crate) wifi_credentials: Utf8PathBuf,
    #[arg(long = "private-root", value_parser = parse_utf8_path)]
    pub(crate) private_root: Utf8PathBuf,
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) plan: Utf8PathBuf,
    #[arg(long = "redact-evidence")]
    pub(crate) redact_evidence: bool,
    #[arg(long = "admission-only")]
    pub(crate) admission_only: bool,
}

#[derive(Debug, Parser, Clone)]
pub(crate) struct NvsRuntimeRestoreCommand {
    #[arg(long, default_value = "205", value_parser = parse_board)]
    pub(crate) board: BoardId,
    #[arg(long)]
    pub(crate) port: String,
    #[arg(long = "private-root", value_parser = parse_utf8_path)]
    pub(crate) private_root: Utf8PathBuf,
    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) plan: Utf8PathBuf,
    #[arg(long = "redact-evidence")]
    pub(crate) redact_evidence: bool,
}

#[derive(Debug, Args, Clone)]
pub(crate) struct CommonArgs {
    #[arg(long, default_value = "205", value_parser = parse_board)]
    pub(crate) board: BoardId,

    #[arg(long)]
    pub(crate) port: Option<String>,

    #[arg(long)]
    pub(crate) dry_run: bool,

    #[arg(long = "redact-evidence")]
    pub(crate) redact_evidence: bool,

    #[arg(long = "evidence-mode", value_enum, conflicts_with = "redact_evidence")]
    pub(crate) evidence_mode: Option<EvidenceMode>,

    #[arg(long = "evidence-dir", value_parser = parse_utf8_path)]
    pub(crate) evidence_dir: Option<Utf8PathBuf>,
}

#[derive(Debug, Parser, Clone)]
pub(crate) struct FlashCommand {
    /// Explicit factory installation erases NVS, including Device Identity and replay state.
    #[arg(long = "factory-reset")]
    pub(crate) factory_reset: bool,

    #[command(flatten)]
    pub(crate) common: CommonArgs,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) image: Option<Utf8PathBuf>,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) manifest: Option<Utf8PathBuf>,

    #[arg(long = "wifi-credentials", value_parser = parse_utf8_path)]
    pub(crate) wifi_credentials: Option<Utf8PathBuf>,
}

#[derive(Debug, Parser, Clone)]
pub(crate) struct MonitorCommand {
    #[command(flatten)]
    pub(crate) common: CommonArgs,

    #[arg(long = "capture-timeout-seconds", default_value_t = DEFAULT_MONITOR_CAPTURE_TIMEOUT_SECONDS)]
    pub(crate) capture_timeout_seconds: u64,
}

#[derive(Debug, Parser, Clone)]
pub(crate) struct FlashMonitorCommand {
    /// Explicit factory installation erases NVS, including Device Identity and replay state.
    #[arg(long = "factory-reset")]
    pub(crate) factory_reset: bool,

    #[command(flatten)]
    pub(crate) common: CommonArgs,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) image: Option<Utf8PathBuf>,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) manifest: Option<Utf8PathBuf>,

    #[arg(long = "wifi-credentials", value_parser = parse_utf8_path)]
    pub(crate) wifi_credentials: Option<Utf8PathBuf>,

    #[arg(long = "network-reconnect-probe", requires = "wifi_credentials")]
    pub(crate) network_reconnect_probe: bool,

    #[arg(
        long = "thermal-fault-stimulus-intent",
        value_parser = parse_utf8_path,
        requires = "wifi_credentials",
        conflicts_with = "network_reconnect_probe"
    )]
    pub(crate) thermal_fault_stimulus_intent: Option<Utf8PathBuf>,

    #[arg(
        long = "self-test-intent",
        value_parser = parse_utf8_path,
        requires = "wifi_credentials",
        conflicts_with_all = ["network_reconnect_probe", "thermal_fault_stimulus_intent"]
    )]
    pub(crate) self_test_intent: Option<Utf8PathBuf>,

    #[arg(long = "capture-timeout-seconds", default_value_t = DEFAULT_MONITOR_CAPTURE_TIMEOUT_SECONDS)]
    pub(crate) capture_timeout_seconds: u64,
}

#[derive(Debug, Parser, Clone)]
pub(crate) struct FinalizeEvidenceCommand {
    #[arg(long = "evidence-dir", value_parser = parse_utf8_path)]
    pub(crate) evidence_dir: Utf8PathBuf,

    #[arg(long = "expected-private-sha256", value_parser = parse_sha256)]
    pub(crate) expected_private_sha256: String,
}

#[derive(Debug, Parser, Clone)]
pub(crate) struct MiningCampaignCommand {
    #[arg(long, value_enum)]
    pub(crate) stage: MiningCampaignStage,

    #[arg(long, value_enum)]
    pub(crate) profile: Option<MiningCampaignProfile>,

    #[arg(long, default_value = "205", value_parser = parse_board)]
    pub(crate) board: BoardId,

    #[arg(long)]
    pub(crate) port: Option<String>,

    #[arg(long, value_parser = parse_utf8_path)]
    pub(crate) manifest: Option<Utf8PathBuf>,

    #[arg(long = "wifi-credentials", value_parser = parse_utf8_path)]
    pub(crate) wifi_credentials: Utf8PathBuf,

    #[arg(long = "pool-credentials", value_parser = parse_utf8_path)]
    pub(crate) pool_credentials: Option<Utf8PathBuf>,

    #[arg(long = "evidence-dir", value_parser = parse_utf8_path)]
    pub(crate) evidence_dir: Utf8PathBuf,

    #[arg(long = "duration-seconds")]
    pub(crate) duration_seconds: u64,

    #[arg(long = "redact-evidence")]
    pub(crate) redact_evidence: bool,
}

#[derive(Debug, Parser, Clone)]
pub(crate) struct Phase35ProbeCommand {
    #[arg(long, default_value = "205", value_parser = parse_board)]
    pub(crate) board: BoardId,

    #[arg(long)]
    pub(crate) port: String,

    #[arg(long = "stage-root", value_parser = parse_utf8_path)]
    pub(crate) stage_root: Utf8PathBuf,

    #[arg(long = "timeout-seconds", default_value_t = 30)]
    pub(crate) timeout_seconds: u64,
}

#[derive(Debug, Serialize)]
pub(crate) struct Phase36EffectResult<'a> {
    pub(crate) schema_version: &'static str,
    pub(crate) operation: &'a str,
    pub(crate) status: &'static str,
    pub(crate) failure: Option<&'static str>,
    pub(crate) package_identity_digest: &'a str,
    pub(crate) factory_image_digest: &'a str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BoardId {
    Ultra205,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvidenceRedactionMode {
    DeveloperRaw,
    CommitRedacted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
pub(crate) enum EvidenceMode {
    Dual,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum MiningCampaignStage {
    Observation,
    LiveShare,
    Soak,
    JobTransition,
    CommandEffects,
    StratumV2,
}

impl MiningCampaignStage {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Observation => "observation",
            Self::LiveShare => "live-share",
            Self::Soak => "soak",
            Self::JobTransition => "job-transition",
            Self::CommandEffects => "command-effects",
            Self::StratumV2 => "stratum-v2",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum MiningCampaignProfile {
    Conservative,
    UpstreamDefault,
}

impl MiningCampaignProfile {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Conservative => "conservative",
            Self::UpstreamDefault => "upstream-default",
        }
    }
}

impl EvidenceRedactionMode {
    pub(crate) fn from_common(common: &CommonArgs) -> Self {
        if common.redact_evidence {
            return Self::CommitRedacted;
        }

        Self::DeveloperRaw
    }

    pub(crate) fn as_str(self) -> &'static str {
        match self {
            Self::DeveloperRaw => "developer-raw",
            Self::CommitRedacted => "commit-redacted",
        }
    }

    pub(crate) fn commit_ready(self) -> bool {
        matches!(self, Self::CommitRedacted)
    }
}

impl FromStr for BoardId {
    type Err = String;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        match value {
            "205" => Ok(Self::Ultra205),
            "601" => Err(
                "board 601 is deferred after the Ultra 205 pivot; Phase 1 supports board=205 only"
                    .to_owned(),
            ),
            other => Err(format!(
                "unsupported board {other}; Phase 1 supports board=205 only"
            )),
        }
    }
}

impl fmt::Display for BoardId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Ultra205 => formatter.write_str("205"),
        }
    }
}
