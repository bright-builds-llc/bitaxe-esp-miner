use std::collections::BTreeSet;
use std::env;
use std::fmt;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::process::Command;
use std::str::FromStr;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use bitaxe_api::BuildProvenance;
use bitaxe_config::{
    apply_settings_patch, ConfigValidationError, NvsWrite, RawSettingValue, SettingsPatch,
    SettingsUpdateDecision, NVS_NAMESPACE,
};
use camino::{Utf8Path, Utf8PathBuf};
use clap::{Args, Parser, Subcommand, ValueEnum};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

mod esp32s3_image;
mod evidence;
mod package_admission;

const PACKAGE_BUILD_DISPLAY: &str = "bazel build //firmware/bitaxe:firmware_image";
const PACKAGE_BUILD_TARGET: &str = "//firmware/bitaxe:firmware_image";
const PACKAGE_MANIFEST_RELATIVE_PATH: &str = "firmware/bitaxe/bitaxe-ultra205-package.json";
const DEFAULT_ELF_NAME: &str = "bitaxe-ultra205.elf";
const FACTORY_IMAGE_NAME: &str = "bitaxe-ultra205-factory.bin";
const DEFAULT_MONITOR_CAPTURE_TIMEOUT_SECONDS: u64 = 25;
const MIN_COMMIT_PREFIX_LEN: usize = 12;
const NVS_PARTITION_OFFSET: &str = "0x9000";
const NVS_PARTITION_SIZE: &str = "0x6000";
const NVS_GENERATOR_PYTHON_RELATIVE_PATH: &str =
    ".embuild/espressif/python_env/idf5.5_py3.9_env/bin/python";
const BUILD_IDENTITY_STATUS_RELATIVE_PATH: &str = "scripts/build-identity-status.sh";
const UNAVAILABLE: &str = "Unavailable";
const PROTECTED_OPERATIONAL: &str = "protected-operational";
const ESPFLASH_EXPECTED_VERSION: &str = "4.5.0";
const PHASE35_FLASH_SCHEMA: &str = "phase35-flash-boundary-v1";

#[derive(Debug, Parser)]
#[command(name = "bitaxe-flash")]
#[command(about = "Safe Bitaxe Ultra 205 flash and monitor workflow.")]
struct Cli {
    #[command(subcommand)]
    command: CliCommand,
}

#[derive(Debug, Subcommand)]
enum CliCommand {
    Flash(FlashCommand),
    Monitor(MonitorCommand),
    #[command(name = "flash-monitor")]
    FlashMonitor(FlashMonitorCommand),
    #[command(name = "finalize-evidence")]
    FinalizeEvidence(FinalizeEvidenceCommand),
    #[command(name = "phase35-probe")]
    Phase35Probe(Phase35ProbeCommand),
}

#[derive(Debug, Args, Clone)]
struct CommonArgs {
    #[arg(long, default_value = "205", value_parser = parse_board)]
    board: BoardId,

    #[arg(long)]
    port: Option<String>,

    #[arg(long)]
    dry_run: bool,

    #[arg(long = "redact-evidence")]
    redact_evidence: bool,

    #[arg(long = "evidence-mode", value_enum, conflicts_with = "redact_evidence")]
    evidence_mode: Option<EvidenceMode>,

    #[arg(long = "evidence-dir", value_parser = parse_utf8_path)]
    evidence_dir: Option<Utf8PathBuf>,
}

#[derive(Debug, Parser, Clone)]
struct FlashCommand {
    #[command(flatten)]
    common: CommonArgs,

    #[arg(long, value_parser = parse_utf8_path)]
    image: Option<Utf8PathBuf>,

    #[arg(long, value_parser = parse_utf8_path)]
    manifest: Option<Utf8PathBuf>,

    #[arg(long = "wifi-credentials", value_parser = parse_utf8_path)]
    wifi_credentials: Option<Utf8PathBuf>,
}

#[derive(Debug, Parser, Clone)]
struct MonitorCommand {
    #[command(flatten)]
    common: CommonArgs,
}

#[derive(Debug, Parser, Clone)]
struct FlashMonitorCommand {
    #[command(flatten)]
    common: CommonArgs,

    #[arg(long, value_parser = parse_utf8_path)]
    image: Option<Utf8PathBuf>,

    #[arg(long, value_parser = parse_utf8_path)]
    manifest: Option<Utf8PathBuf>,

    #[arg(long = "wifi-credentials", value_parser = parse_utf8_path)]
    wifi_credentials: Option<Utf8PathBuf>,

    #[arg(long = "capture-timeout-seconds", default_value_t = DEFAULT_MONITOR_CAPTURE_TIMEOUT_SECONDS)]
    capture_timeout_seconds: u64,
}

#[derive(Debug, Parser, Clone)]
struct FinalizeEvidenceCommand {
    #[arg(long = "evidence-dir", value_parser = parse_utf8_path)]
    evidence_dir: Utf8PathBuf,

    #[arg(long = "expected-private-sha256", value_parser = parse_sha256)]
    expected_private_sha256: String,
}

#[derive(Debug, Parser, Clone)]
struct Phase35ProbeCommand {
    #[arg(long, default_value = "205", value_parser = parse_board)]
    board: BoardId,

    #[arg(long)]
    port: String,

    #[arg(long = "stage-root", value_parser = parse_utf8_path)]
    stage_root: Utf8PathBuf,

    #[arg(long = "timeout-seconds", default_value_t = 30)]
    timeout_seconds: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BoardId {
    Ultra205,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EvidenceRedactionMode {
    DeveloperRaw,
    CommitRedacted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum EvidenceMode {
    Dual,
}

impl EvidenceRedactionMode {
    fn from_common(common: &CommonArgs) -> Self {
        if common.redact_evidence {
            return Self::CommitRedacted;
        }

        Self::DeveloperRaw
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::DeveloperRaw => "developer-raw",
            Self::CommitRedacted => "commit-redacted",
        }
    }

    fn commit_ready(self) -> bool {
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

#[derive(Debug, Clone, Eq, PartialEq)]
struct CommandSpec {
    program: String,
    args: Vec<String>,
}

impl CommandSpec {
    fn new<I, S>(program: &str, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        Self {
            program: program.to_owned(),
            args: args
                .into_iter()
                .map(|arg| arg.as_ref().to_owned())
                .collect(),
        }
    }

    fn display(&self) -> String {
        let mut parts = Vec::with_capacity(self.args.len() + 1);
        parts.push(self.program.clone());
        parts.extend(self.args.iter().cloned());
        parts.join(" ")
    }
}

#[derive(Debug)]
struct FlashOutcome {
    manifest: Option<Utf8PathBuf>,
    flash_image: Utf8PathBuf,
    command: CommandSpec,
    nvs_seed: Option<NvsSeedOutcome>,
}

struct PreparedFlash {
    outcome: FlashOutcome,
    execution_command: CommandSpec,
    _execution_snapshot: Option<AdmittedExecutionSnapshot>,
}

struct AdmittedFactoryImage {
    manifest: Utf8PathBuf,
    display_path: Utf8PathBuf,
    bytes: Vec<u8>,
}

enum AdmittedFlashImage {
    DeveloperDryRun { display_path: Utf8PathBuf },
    Factory(AdmittedFactoryImage),
}

impl AdmittedFlashImage {
    fn maybe_manifest(&self) -> Option<&Utf8Path> {
        match self {
            Self::DeveloperDryRun { .. } => None,
            Self::Factory(factory) => Some(&factory.manifest),
        }
    }

    fn display_path(&self) -> &Utf8Path {
        match self {
            Self::DeveloperDryRun { display_path } => display_path,
            Self::Factory(factory) => &factory.display_path,
        }
    }

    fn maybe_factory_bytes(&self) -> Option<&[u8]> {
        match self {
            Self::DeveloperDryRun { .. } => None,
            Self::Factory(factory) => Some(&factory.bytes),
        }
    }
}

struct AdmittedExecutionSnapshot {
    _file: tempfile::NamedTempFile,
    path: Utf8PathBuf,
}

impl AdmittedExecutionSnapshot {
    fn materialize(bytes: &[u8]) -> Result<Self> {
        let mut file = tempfile::NamedTempFile::new().map_err(|_| {
            anyhow::anyhow!("identity_admission=blocked reason=execution_snapshot_create_failed")
        })?;
        file.as_file_mut().write_all(bytes).map_err(|_| {
            anyhow::anyhow!("identity_admission=blocked reason=execution_snapshot_write_failed")
        })?;
        file.as_file_mut().flush().map_err(|_| {
            anyhow::anyhow!("identity_admission=blocked reason=execution_snapshot_write_failed")
        })?;
        file.as_file().sync_all().map_err(|_| {
            anyhow::anyhow!("identity_admission=blocked reason=execution_snapshot_sync_failed")
        })?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            let mut permissions = file
                .as_file()
                .metadata()
                .map_err(|_| {
                    anyhow::anyhow!(
                        "identity_admission=blocked reason=execution_snapshot_permissions_failed"
                    )
                })?
                .permissions();
            permissions.set_mode(0o600);
            file.as_file().set_permissions(permissions).map_err(|_| {
                anyhow::anyhow!(
                    "identity_admission=blocked reason=execution_snapshot_permissions_failed"
                )
            })?;
        }
        let path = Utf8PathBuf::from_path_buf(file.path().to_path_buf()).map_err(|_| {
            anyhow::anyhow!("identity_admission=blocked reason=execution_snapshot_path_invalid")
        })?;

        Ok(Self { _file: file, path })
    }

    fn path(&self) -> &Utf8Path {
        &self.path
    }
}

#[derive(Debug)]
struct NvsSeedOutcome {
    image: Utf8PathBuf,
    command: CommandSpec,
    _temp_dir: tempfile::TempDir,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct CaptureProcessResult {
    status: CaptureProcessStatus,
}

#[derive(Debug, Clone, Eq, PartialEq)]
enum CaptureProcessStatus {
    SpawnFailed,
    ExitedSuccess,
    ExitedFailure(String),
    TimedOut,
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
enum CaptureStatus {
    Completed,
    TimedOutAfterTrustedOutput,
    TimedOutWithoutTrustedOutput,
    Failed,
    DryRun,
}

#[derive(Debug, Clone, Eq, PartialEq, Serialize)]
struct MonitorCaptureOutcome {
    capture_mode: String,
    capture_status: CaptureStatus,
    capture_timeout_seconds: u64,
    trusted_output: bool,
    observed_firmware_commit: String,
    observed_reference_commit: String,
    conclusion: String,
}

impl MonitorCaptureOutcome {
    fn accepted(&self) -> bool {
        self.trusted_output
            && matches!(
                self.capture_status,
                CaptureStatus::Completed | CaptureStatus::TimedOutAfterTrustedOutput
            )
    }
}

struct EvidenceRecordInput<'a> {
    command_kind: &'a str,
    command: &'a str,
    flash_command: &'a str,
    monitor_command: &'a str,
    log_path: &'a Utf8Path,
    private_log_path: Option<&'a Utf8Path>,
    private_log_sha256: Option<&'a str>,
    admitted_log_sha256: Option<&'a str>,
    capture_outcome: &'a MonitorCaptureOutcome,
}

struct MonitorEvidenceArtifacts<'a> {
    admitted_log: &'a Utf8Path,
    dual_paths: Option<&'a evidence::DualEvidencePaths>,
    private_log_sha256: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
struct PackageManifest {
    schema_version: u32,
    semantic_version: String,
    source_commit: String,
    reference_commit: String,
    app_elf_sha256: String,
    build_identity: PackageBuildIdentity,
    default_flash_image: String,
    artifacts: Vec<PackageArtifact>,
}

#[derive(Debug, Deserialize)]
struct PackageBuildIdentity {
    label: String,
    channel: String,
    source_dirty: bool,
    release_tag: Option<String>,
}

#[derive(Debug, Deserialize)]
struct PackageArtifact {
    kind: String,
    path: String,
    sha256: String,
}

trait FlashEnvironment {
    fn build_package(&self) -> Result<()>;
    fn bazel_bin(&self) -> Result<Utf8PathBuf>;
    fn workspace_path(&self, path: &Utf8Path) -> Utf8PathBuf {
        path.to_owned()
    }
    fn read_to_string(&self, path: &Utf8Path) -> Result<String>;
    fn read_bytes(&self, path: &Utf8Path) -> Result<Vec<u8>>;
    fn create_admitted_execution_snapshot(
        &self,
        bytes: &[u8],
    ) -> Result<AdmittedExecutionSnapshot> {
        AdmittedExecutionSnapshot::materialize(bytes)
    }
    fn approve_private_evidence_root(&self, path: &Utf8Path) -> Result<()>;
    fn current_provenance(&self) -> Result<BuildProvenance>;
    fn list_ports(&self) -> Result<String>;
    fn write_file(&self, path: &Utf8Path, contents: &str) -> Result<()>;
    fn generate_nvs_partition(
        &self,
        csv_path: &Utf8Path,
        bin_path: &Utf8Path,
        size: &str,
    ) -> Result<()>;
    fn execute(&self, command_spec: &CommandSpec) -> Result<()>;
    fn phase35_stage_readiness_gate(&self, _stage: &str, _port: &str) -> Result<()> {
        Ok(())
    }
    fn execute_capturing(
        &self,
        command_spec: &CommandSpec,
        log_path: &Utf8Path,
        timeout_seconds: u64,
        redaction_mode: EvidenceRedactionMode,
        create_new: bool,
    ) -> Result<CaptureProcessResult>;
    fn firmware_commit(&self) -> String;
    fn reference_commit(&self) -> String;
    fn write_evidence(&self, path: &Utf8Path, contents: &str) -> Result<()>;
}

#[derive(Debug)]
struct LocalFlashEnvironment {
    workspace_dir: Utf8PathBuf,
    espflash_bin: Utf8PathBuf,
    espflash_version: String,
    espflash_sha256: String,
}

impl LocalFlashEnvironment {
    fn detect() -> Result<Self> {
        let espflash_bin = resolve_espflash_executable()?;
        let output = Command::new(espflash_bin.as_std_path())
            .arg("--version")
            .output()
            .context("failed to query espflash version")?;
        let espflash_version = command_output_to_string(output, "espflash --version")?;
        if espflash_version != format!("espflash {ESPFLASH_EXPECTED_VERSION}") {
            bail!("espflash_version_mismatch expected={ESPFLASH_EXPECTED_VERSION}");
        }
        let espflash_sha256 = sha256_bytes(
            &fs::read(espflash_bin.as_std_path())
                .context("failed to digest espflash executable")?,
        );
        Ok(Self {
            workspace_dir: detect_workspace_dir()?,
            espflash_bin,
            espflash_version,
            espflash_sha256,
        })
    }
}

fn approve_local_private_evidence_root(
    workspace_dir: &Utf8Path,
    requested_root: &Utf8Path,
) -> Result<()> {
    let canonical_workspace = fs::canonicalize(workspace_dir.as_std_path())
        .context("failed to resolve workspace for private evidence admission")?;
    let canonical_workspace = Utf8PathBuf::from_path_buf(canonical_workspace)
        .map_err(|_| anyhow::anyhow!("private_evidence_root=blocked reason=non_utf8_workspace"))?;
    let relative_root = if requested_root.is_absolute() {
        requested_root
            .strip_prefix(&canonical_workspace)
            .or_else(|_| requested_root.strip_prefix(workspace_dir))
            .map(Utf8Path::to_owned)
            .map_err(|_| {
                anyhow::anyhow!("private_evidence_root=blocked reason=outside_workspace")
            })?
    } else {
        requested_root.to_owned()
    };
    if relative_root.as_str().is_empty()
        || relative_root.as_std_path().components().any(|component| {
            !matches!(
                component,
                std::path::Component::Normal(_) | std::path::Component::CurDir
            )
        })
    {
        bail!("private_evidence_root=blocked reason=invalid_workspace_path");
    }

    let canonical_candidate = canonical_workspace.join(&relative_root);
    let mut maybe_existing = Some(canonical_candidate.as_path());
    let existing_ancestor = loop {
        let Some(candidate) = maybe_existing else {
            bail!("private_evidence_root=blocked reason=missing_workspace_ancestor");
        };
        if candidate.exists() {
            break candidate;
        }
        maybe_existing = candidate.parent();
    };
    let canonical_ancestor = fs::canonicalize(existing_ancestor.as_std_path())
        .context("failed to resolve private evidence ancestor")?;
    if !canonical_ancestor.starts_with(canonical_workspace.as_std_path()) {
        bail!("private_evidence_root=blocked reason=symlink_escape");
    }

    let status = Command::new("git")
        .current_dir(canonical_workspace.as_std_path())
        .args(["check-ignore", "--quiet", "--"])
        .arg(relative_root.as_std_path())
        .status()
        .context("failed to verify private evidence ignore admission")?;
    if !status.success() {
        bail!("private_evidence_root=blocked reason=not_repo_ignored");
    }
    Ok(())
}

impl FlashEnvironment for LocalFlashEnvironment {
    fn build_package(&self) -> Result<()> {
        let status = Command::new("bazel")
            .current_dir(self.workspace_dir.as_std_path())
            .arg("build")
            .arg(PACKAGE_BUILD_TARGET)
            .status()
            .context("failed to run bazel build for firmware package")?;
        if !status.success() {
            bail!("{PACKAGE_BUILD_DISPLAY} failed with {status}");
        }

        Ok(())
    }

    fn execute_capturing(
        &self,
        command_spec: &CommandSpec,
        log_path: &Utf8Path,
        timeout_seconds: u64,
        redaction_mode: EvidenceRedactionMode,
        create_new: bool,
    ) -> Result<CaptureProcessResult> {
        let mut resolved_command = command_spec.clone();
        resolved_command.program = self.espflash_bin.to_string();
        let result = evidence::capture_command(
            &resolved_command,
            &self.espflash_bin,
            log_path,
            timeout_seconds,
            redaction_mode,
            create_new,
        )?;
        if let Ok(stage_root) = env::var("PHASE35_FLASH_STAGE_ROOT") {
            if !stage_root.is_empty() {
                let private_log = fs::read(log_path.as_std_path())?;
                let observed_bytes = !private_log.is_empty();
                let launched = !matches!(result.status, CaptureProcessStatus::SpawnFailed);
                let connected = launched && observed_bytes;
                let completed = connected
                    && !matches!(
                        result.status,
                        CaptureProcessStatus::SpawnFailed | CaptureProcessStatus::ExitedFailure(_)
                    );
                let metrics = serde_json::json!({
                    "schema_version": PHASE35_FLASH_SCHEMA,
                    "stage": "monitor",
                    "tool_version_valid": true,
                    "launched": launched,
                    "connected": connected,
                    "device_info_complete": connected,
                    "transfer_started": connected,
                    "completed": completed,
                    "duration_millis": timeout_seconds.saturating_mul(1_000),
                });
                let stage_root = Utf8Path::new(&stage_root);
                fs::create_dir_all(stage_root.as_std_path())?;
                set_private_directory_mode(stage_root)?;
                let monitor_log = stage_root.join("monitor.private.log");
                let monitor_metrics = stage_root.join("monitor.metrics.json");
                write_private_new_bytes(&monitor_log, &private_log)?;
                let mut encoded = serde_json::to_vec_pretty(&metrics)?;
                encoded.push(b'\n');
                write_private_new_bytes(&monitor_metrics, &encoded)?;
            }
        }
        Ok(result)
    }

    fn approve_private_evidence_root(&self, path: &Utf8Path) -> Result<()> {
        approve_local_private_evidence_root(&self.workspace_dir, path)
    }

    fn bazel_bin(&self) -> Result<Utf8PathBuf> {
        let output = Command::new("bazel")
            .current_dir(self.workspace_dir.as_std_path())
            .arg("info")
            .arg("bazel-bin")
            .output()
            .context("failed to run bazel info bazel-bin")?;
        command_output_to_string(output, "bazel info bazel-bin").map(Utf8PathBuf::from)
    }

    fn workspace_path(&self, path: &Utf8Path) -> Utf8PathBuf {
        if path.is_absolute() {
            return path.to_owned();
        }

        self.workspace_dir.join(path)
    }

    fn read_to_string(&self, path: &Utf8Path) -> Result<String> {
        fs::read_to_string(path.as_std_path()).with_context(|| format!("failed to read {path}"))
    }

    fn read_bytes(&self, path: &Utf8Path) -> Result<Vec<u8>> {
        fs::read(path.as_std_path()).with_context(|| format!("failed to read {path}"))
    }

    fn current_provenance(&self) -> Result<BuildProvenance> {
        let status_command = self.workspace_dir.join(BUILD_IDENTITY_STATUS_RELATIVE_PATH);
        let output = Command::new(status_command.as_std_path())
            .current_dir(self.workspace_dir.as_std_path())
            .output()
            .context("failed to run canonical build identity status command")?;
        let status = command_output_to_string(output, "build identity status command")?;
        BuildProvenance::parse_workspace_status(&status)
            .context("current workspace build identity is invalid")
    }

    fn list_ports(&self) -> Result<String> {
        let output = Command::new(self.espflash_bin.as_std_path())
            .arg("list-ports")
            .output()
            .context("failed to run espflash list-ports")?;
        command_output_to_string(output, "espflash list-ports")
    }

    fn write_file(&self, path: &Utf8Path, contents: &str) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent.as_std_path())
                .with_context(|| format!("failed to create directory {parent}"))?;
        }

        fs::write(path.as_std_path(), contents).with_context(|| format!("failed to write {path}"))
    }

    fn generate_nvs_partition(
        &self,
        csv_path: &Utf8Path,
        bin_path: &Utf8Path,
        size: &str,
    ) -> Result<()> {
        let python = self.nvs_generator_python()?;
        let output = Command::new(python.as_std_path())
            .arg("-m")
            .arg("esp_idf_nvs_partition_gen")
            .arg("generate")
            .arg(csv_path.as_str())
            .arg(bin_path.as_str())
            .arg(size)
            .output()
            .context("failed to run ESP-IDF NVS partition generator")?;
        if !output.status.success() {
            bail!(
                "ESP-IDF NVS partition generator failed: {}",
                command_stderr_or_status(&output)
            );
        }

        Ok(())
    }

    fn execute(&self, command_spec: &CommandSpec) -> Result<()> {
        if command_spec.program != "espflash" {
            bail!("unsupported command program: {}", command_spec.program);
        }

        if let Some(stage) = phase35_stage_for_command(command_spec) {
            if let Ok(stage_root) = env::var("PHASE35_FLASH_STAGE_ROOT") {
                if !stage_root.is_empty() {
                    return self.execute_phase35_stage(
                        command_spec,
                        stage,
                        Utf8Path::new(&stage_root),
                    );
                }
            }
        }

        let mut command = Command::new(self.espflash_bin.as_std_path());
        for arg in &command_spec.args {
            command.arg(arg);
        }

        let status = command
            .status()
            .with_context(|| format!("failed to run {}", command_spec.display()))?;
        if !status.success() {
            bail!("{} failed with {status}", command_spec.display());
        }

        Ok(())
    }

    fn phase35_stage_readiness_gate(&self, stage: &str, port: &str) -> Result<()> {
        let Ok(stage_root) = env::var("PHASE35_FLASH_STAGE_ROOT") else {
            return Ok(());
        };
        if stage_root.is_empty() {
            return Ok(());
        }
        if !matches!(stage, "after-factory" | "after-nvs") {
            bail!("phase35_stage_readiness=blocked reason=invalid_stage");
        }

        let expected_gate = self
            .workspace_dir
            .join("scripts/phase35-stage-readiness.sh");
        let expected_gate = fs::canonicalize(expected_gate.as_std_path())
            .context("phase35_stage_readiness=blocked reason=gate_unavailable")?;
        let requested_gate = env::var("PHASE35_STAGE_READINESS_BIN")
            .context("phase35_stage_readiness=blocked reason=gate_unavailable")?;
        let requested_gate = fs::canonicalize(&requested_gate)
            .context("phase35_stage_readiness=blocked reason=gate_unavailable")?;
        if requested_gate != expected_gate {
            bail!("phase35_stage_readiness=blocked reason=untrusted_gate");
        }
        let expected_physical_identity = env::var("PHASE35_EXPECTED_PHYSICAL_IDENTITY")
            .context("phase35_stage_readiness=blocked reason=identity_unavailable")?;
        if !is_lower_hex_digest(&expected_physical_identity) {
            bail!("phase35_stage_readiness=blocked reason=identity_invalid");
        }
        let trace_root = Utf8Path::new(&stage_root).join("readiness");
        fs::create_dir_all(trace_root.as_std_path())
            .context("phase35_stage_readiness=blocked reason=trace_root_invalid")?;
        set_private_directory_mode(&trace_root)?;

        let output = Command::new(expected_gate)
            .args([
                "--stage",
                stage,
                "--port",
                port,
                "--expected-physical-identity",
                expected_physical_identity.as_str(),
                "--trace-root",
                trace_root.as_str(),
            ])
            .output()
            .context("phase35_stage_readiness=failed reason=spawn_failure")?;
        if !output.status.success() {
            bail!("phase35_stage_readiness=failed reason=gate_rejected");
        }
        let stdout = std::str::from_utf8(&output.stdout)
            .context("phase35_stage_readiness=failed reason=output_invalid")?;
        validate_phase35_readiness_output(stdout)?;
        Ok(())
    }

    fn firmware_commit(&self) -> String {
        git_output(&self.workspace_dir, ["rev-parse", "HEAD"])
            .unwrap_or_else(|| UNAVAILABLE.to_owned())
    }

    fn reference_commit(&self) -> String {
        git_output(
            &self.workspace_dir,
            ["-C", "reference/esp-miner", "rev-parse", "HEAD"],
        )
        .unwrap_or_else(|| UNAVAILABLE.to_owned())
    }

    fn write_evidence(&self, path: &Utf8Path, contents: &str) -> Result<()> {
        let maybe_parent = path.parent();
        if let Some(parent) = maybe_parent {
            fs::create_dir_all(parent.as_std_path())
                .with_context(|| format!("failed to create evidence directory {parent}"))?;
        }

        fs::write(path.as_std_path(), contents)
            .with_context(|| format!("failed to write evidence {path}"))
    }
}

impl LocalFlashEnvironment {
    fn execute_phase35_stage(
        &self,
        command_spec: &CommandSpec,
        stage: &str,
        stage_root: &Utf8Path,
    ) -> Result<()> {
        fs::create_dir_all(stage_root.as_std_path())
            .context("failed to create private Phase 35 stage root")?;
        set_private_directory_mode(stage_root)?;
        let log_path = stage_root.join(format!("{stage}.private.log"));
        let metrics_path = stage_root.join(format!("{stage}.metrics.json"));
        if log_path.exists() || metrics_path.exists() {
            bail!("phase35_stage_capture=blocked reason=destination_exists");
        }

        let mut resolved_command = command_spec.clone();
        resolved_command.program = self.espflash_bin.to_string();
        let started = Instant::now();
        let capture = evidence::capture_command(
            &resolved_command,
            &self.espflash_bin,
            &log_path,
            360,
            EvidenceRedactionMode::DeveloperRaw,
            true,
        )?;
        let duration_millis = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        let launched = !matches!(capture.status, CaptureProcessStatus::SpawnFailed);
        let success = matches!(capture.status, CaptureProcessStatus::ExitedSuccess);
        let log = fs::read_to_string(log_path.as_std_path())
            .context("failed to read sanitized Phase 35 stage log")?;
        let connected = launched && flash_log_connected(&log);
        let device_info_complete = connected && flash_log_device_info_complete(&log);
        let transfer_started =
            device_info_complete && (success || flash_log_transfer_started(&log));
        let completed = transfer_started && success;
        let metrics = serde_json::json!({
            "schema_version": PHASE35_FLASH_SCHEMA,
            "stage": stage,
            "tool_version_valid": self.espflash_version == format!("espflash {ESPFLASH_EXPECTED_VERSION}"),
            "launched": launched,
            "connected": connected,
            "device_info_complete": device_info_complete,
            "transfer_started": transfer_started,
            "completed": completed,
            "duration_millis": if launched { duration_millis } else { 0 },
        });
        let mut encoded = serde_json::to_vec_pretty(&metrics)?;
        encoded.push(b'\n');
        write_private_new_bytes(&metrics_path, &encoded)?;
        if !launched {
            bail!("phase35_stage_capture=failed reason=spawn_failure");
        }
        if !success {
            bail!("phase35_stage_capture=failed reason=child_failure");
        }
        Ok(())
    }
}

impl LocalFlashEnvironment {
    fn nvs_generator_python(&self) -> Result<Utf8PathBuf> {
        if let Ok(path) = env::var("ESP_IDF_NVS_PYTHON") {
            if !path.is_empty() {
                return Ok(Utf8PathBuf::from(path));
            }
        }

        let candidate = self.workspace_dir.join(NVS_GENERATOR_PYTHON_RELATIVE_PATH);
        if !candidate.is_file() {
            bail!(
                "ESP-IDF NVS generator python not found at {candidate}; run just bootstrap-esp or build firmware once"
            );
        }

        Ok(candidate)
    }
}

fn main() -> Result<()> {
    let cli = parse_cli(env::args())?;
    let environment = LocalFlashEnvironment::detect()?;
    emit_line("espflash_version", ESPFLASH_EXPECTED_VERSION)?;
    emit_line("espflash_executable_sha256", &environment.espflash_sha256)?;

    match cli.command {
        CliCommand::Flash(command) => {
            run_flash(&command, &environment)?;
        }
        CliCommand::Monitor(command) => {
            run_monitor(&command, &environment)?;
        }
        CliCommand::FlashMonitor(command) => {
            run_flash_monitor(&command, &environment)?;
        }
        CliCommand::FinalizeEvidence(command) => {
            run_finalize_evidence(&command, &environment)?;
        }
        CliCommand::Phase35Probe(command) => {
            run_phase35_probe(&command, &environment)?;
        }
    }

    Ok(())
}

fn parse_cli<I, S>(args: I) -> Result<Cli>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let normalized = normalize_args(args);
    let cli = Cli::try_parse_from(normalized).map_err(anyhow::Error::new)?;
    match &cli.command {
        CliCommand::Flash(command) if command.common.evidence_mode.is_some() => {
            bail!("--evidence-mode dual is supported only by flash-monitor");
        }
        CliCommand::Monitor(command) if command.common.evidence_mode.is_some() => {
            bail!("--evidence-mode dual is supported only by flash-monitor");
        }
        _ => {}
    }
    Ok(cli)
}

fn normalize_args<I, S>(args: I) -> Vec<String>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let mut normalized = Vec::new();
    for arg in args {
        let arg = arg.into();
        if arg.starts_with("--") {
            normalized.push(arg);
            continue;
        }

        let Some((key, value)) = arg.split_once('=') else {
            normalized.push(arg);
            continue;
        };

        match key {
            "board" => push_flag_value(&mut normalized, "--board", value),
            "port" => push_flag_value(&mut normalized, "--port", value),
            "image" => push_flag_value(&mut normalized, "--image", value),
            "manifest" => push_flag_value(&mut normalized, "--manifest", value),
            "wifi-credentials" | "wifi_credentials" => {
                push_flag_value(&mut normalized, "--wifi-credentials", value)
            }
            "evidence-dir" | "evidence_dir" => {
                push_flag_value(&mut normalized, "--evidence-dir", value)
            }
            "evidence-mode" | "evidence_mode" => {
                push_flag_value(&mut normalized, "--evidence-mode", value)
            }
            "expected-private-sha256" | "expected_private_sha256" => {
                push_flag_value(&mut normalized, "--expected-private-sha256", value)
            }
            "capture-timeout-seconds" | "capture_timeout_seconds" => {
                push_flag_value(&mut normalized, "--capture-timeout-seconds", value)
            }
            "stage-root" | "stage_root" => push_flag_value(&mut normalized, "--stage-root", value),
            "timeout-seconds" | "timeout_seconds" => {
                push_flag_value(&mut normalized, "--timeout-seconds", value)
            }
            "redact-evidence" | "redact_evidence" => {
                if parse_bool_alias(value) {
                    normalized.push("--redact-evidence".to_owned());
                }
            }
            "dry-run" | "dry_run" => {
                if parse_bool_alias(value) {
                    normalized.push("--dry-run".to_owned());
                }
            }
            _ => normalized.push(arg),
        }
    }

    normalized
}

fn push_flag_value(args: &mut Vec<String>, flag: &str, value: &str) {
    args.push(flag.to_owned());
    args.push(value.to_owned());
}

fn parse_bool_alias(value: &str) -> bool {
    matches!(value, "true" | "1" | "yes" | "on")
}

fn run_flash(command: &FlashCommand, environment: &impl FlashEnvironment) -> Result<FlashOutcome> {
    let PreparedFlash {
        outcome,
        execution_command,
        _execution_snapshot,
    } = prepare_flash(command, environment)?;
    emit_flash_outcome(
        &outcome,
        command.common.evidence_mode != Some(EvidenceMode::Dual),
    )?;

    if !command.common.dry_run {
        environment.execute(&execution_command).map_err(|_| {
            anyhow::anyhow!("flash_execution=failed reason=admitted_image_child_failed")
        })?;
        let port = command_port(&execution_command)
            .context("phase35_stage_readiness=blocked reason=port_unavailable")?;
        environment.phase35_stage_readiness_gate("after-factory", &port)?;
        if let Some(nvs_seed) = &outcome.nvs_seed {
            environment.execute(&nvs_seed.command)?;
            environment.phase35_stage_readiness_gate("after-nvs", &port)?;
        }
    }

    write_evidence_if_requested(&command.common, &outcome, "flash", environment)?;
    Ok(outcome)
}

fn run_monitor(command: &MonitorCommand, environment: &impl FlashEnvironment) -> Result<()> {
    let command_spec = prepare_monitor_command(&command.common, environment)?;
    emit_command("monitor_command", &command_spec)?;

    if !command.common.dry_run {
        environment.execute(&command_spec)?;
    }

    Ok(())
}

fn run_flash_monitor(
    command: &FlashMonitorCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let resolved_dir = resolved_evidence_dir(&command.common, environment);
    if command.common.evidence_mode.is_some() && resolved_dir.is_none() {
        bail!("--evidence-mode dual requires --evidence-dir");
    }
    let dual_paths = if command.common.evidence_mode == Some(EvidenceMode::Dual) {
        let evidence_dir = resolved_dir
            .as_deref()
            .context("dual evidence mode requires an evidence directory")?;
        environment
            .approve_private_evidence_root(evidence_dir)
            .map_err(|_| anyhow::anyhow!("dual_evidence=failed reason=root_admission_failed"))?;
        Some(
            evidence::preflight_dual_paths(evidence_dir).map_err(|_| {
                anyhow::anyhow!("dual_evidence=failed reason=path_preflight_failed")
            })?,
        )
    } else {
        None
    };

    let mut flash_common = command.common.clone();
    flash_common.evidence_dir = None;
    let flash_command = FlashCommand {
        common: flash_common,
        image: command.image.clone(),
        manifest: command.manifest.clone(),
        wifi_credentials: command.wifi_credentials.clone(),
    };
    let flash_outcome = run_flash(&flash_command, environment).map_err(|error| {
        if command.common.evidence_mode == Some(EvidenceMode::Dual) {
            return anyhow::anyhow!("dual_evidence=failed reason=flash_workflow_failed");
        }
        error
    })?;

    if let Some(evidence_dir) = resolved_dir {
        let monitor_command = prepare_evidence_monitor_command(&command.common, environment)
            .map_err(|error| {
                if command.common.evidence_mode == Some(EvidenceMode::Dual) {
                    return anyhow::anyhow!(
                        "dual_evidence=failed reason=monitor_preparation_failed"
                    );
                }
                error
            })?;
        emit_operational_command(
            "monitor_command",
            &monitor_command,
            command.common.evidence_mode != Some(EvidenceMode::Dual),
        )?;
        let log_path = evidence_dir.join("flash-monitor.log");
        let capture_log_path = dual_paths
            .as_ref()
            .map(|paths| paths.private_log.as_path())
            .unwrap_or(log_path.as_path());
        let capture_outcome = if command.common.dry_run {
            let dry_run_text =
                "dry-run: espflash monitor was not executed; no hardware log captured\n";
            if let Some(paths) = &dual_paths {
                evidence::write_dual_private_text(&paths.private_log, dry_run_text).map_err(
                    |_| anyhow::anyhow!("dual_evidence=failed reason=private_capture_failed"),
                )?;
            } else {
                environment.write_evidence(&log_path, dry_run_text)?;
            }
            dry_run_monitor_capture_outcome(command.capture_timeout_seconds)
        } else {
            let capture_result = environment
                .execute_capturing(
                    &monitor_command,
                    capture_log_path,
                    command.capture_timeout_seconds,
                    if dual_paths.is_some() {
                        EvidenceRedactionMode::DeveloperRaw
                    } else {
                        EvidenceRedactionMode::from_common(&command.common)
                    },
                    dual_paths.is_some(),
                )
                .map_err(|error| {
                    if command.common.evidence_mode != Some(EvidenceMode::Dual) {
                        return error;
                    }
                    if format!("{error:#}").contains("evidence_sanitization_invalid") {
                        return anyhow::anyhow!("evidence_sanitization_invalid");
                    }
                    anyhow::anyhow!("dual_evidence=failed reason=capture_failed")
                })?;
            let monitor_log = environment
                .read_to_string(capture_log_path)
                .map_err(|error| {
                    if command.common.evidence_mode == Some(EvidenceMode::Dual) {
                        return anyhow::anyhow!(
                            "dual_evidence=failed reason=private_capture_unreadable"
                        );
                    }
                    error.context(format!("failed to read monitor log {capture_log_path}"))
                })?;
            monitor_capture_outcome(
                &capture_result.status,
                &monitor_log,
                command.capture_timeout_seconds,
                &environment.firmware_commit(),
                &environment.reference_commit(),
            )
        };
        let maybe_private_sha256 = dual_paths
            .as_ref()
            .map(|paths| evidence::private_log_sha256(&paths.private_log))
            .transpose()
            .map_err(|error| {
                if command.common.evidence_mode == Some(EvidenceMode::Dual) {
                    return anyhow::anyhow!("dual_evidence=failed reason=private_digest_failed");
                }
                error
            })?;
        write_flash_monitor_evidence_if_requested(
            &command.common,
            &flash_outcome,
            &monitor_command,
            &evidence_dir,
            MonitorEvidenceArtifacts {
                admitted_log: &log_path,
                dual_paths: dual_paths.as_ref(),
                private_log_sha256: maybe_private_sha256.as_deref(),
            },
            &capture_outcome,
            environment,
        )
        .map_err(|error| {
            if command.common.evidence_mode == Some(EvidenceMode::Dual) {
                return anyhow::anyhow!("dual_evidence=failed reason=evidence_record_failed");
            }
            error
        })?;
        if !command.common.dry_run && !capture_outcome.accepted() {
            if command.common.evidence_mode == Some(EvidenceMode::Dual) {
                bail!("dual_evidence=failed reason=capture_not_accepted");
            }
            let port = command_port(&monitor_command).unwrap_or_else(|| UNAVAILABLE.to_owned());
            let user_evidence_dir = command
                .common
                .evidence_dir
                .as_deref()
                .unwrap_or(evidence_dir.as_path());
            bail!(
                "{}\n{}",
                capture_outcome.conclusion,
                evidence_capture_failure_guidance(&port, user_evidence_dir)
            );
        }
        return Ok(());
    }

    let monitor_command = prepare_monitor_command(&command.common, environment)?;
    emit_command("monitor_command", &monitor_command)?;

    if !command.common.dry_run {
        environment.execute(&monitor_command)?;
    }

    Ok(())
}

fn run_finalize_evidence(
    command: &FinalizeEvidenceCommand,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let evidence_dir = environment.workspace_path(&command.evidence_dir);
    environment
        .approve_private_evidence_root(&evidence_dir)
        .map_err(|_| anyhow::anyhow!("dual_evidence=failed reason=root_admission_failed"))?;
    let paths = evidence::preflight_dual_finalization_paths(&evidence_dir)
        .map_err(|_| anyhow::anyhow!("dual_evidence=failed reason=finalize_preflight_failed"))?;
    let private_sha256 = evidence::private_log_sha256(&paths.private_log)
        .map_err(|_| anyhow::anyhow!("dual_evidence=failed reason=private_digest_failed"))?;
    if private_sha256 != command.expected_private_sha256 {
        bail!("dual_evidence=failed reason=classified_digest_mismatch");
    }

    let private_json = environment
        .read_to_string(&paths.private_record)
        .map_err(|_| anyhow::anyhow!("dual_evidence=failed reason=private_record_unreadable"))?;
    let mut record: EvidenceRecord = serde_json::from_str(&private_json)
        .map_err(|_| anyhow::anyhow!("dual_evidence=failed reason=private_record_invalid"))?;
    if record.redaction_mode != "dual"
        || record.commit_ready
        || record.private_monitor_log_path.as_deref() != Some(paths.private_log.as_str())
        || record.private_monitor_log_sha256.as_deref()
            != Some(command.expected_private_sha256.as_str())
        || record.monitor_log_sha256.is_some()
    {
        bail!("dual_evidence=failed reason=private_record_mismatch");
    }

    let finalize_result = (|| -> Result<()> {
        let digests =
            evidence::derive_admitted_log(&paths, command.expected_private_sha256.as_str())?;
        record.command = PROTECTED_OPERATIONAL.to_owned();
        record.flash_command = PROTECTED_OPERATIONAL.to_owned();
        record.monitor_command = PROTECTED_OPERATIONAL.to_owned();
        record.port = "[redacted]".to_owned();
        record.manifest_path = PROTECTED_OPERATIONAL.to_owned();
        record.flash_image_path = PROTECTED_OPERATIONAL.to_owned();
        record.log_path = "flash-monitor.log".to_owned();
        record.monitor_log_path = "flash-monitor.log".to_owned();
        record.private_log_role = None;
        record.private_monitor_log_path = None;
        record.private_monitor_log_sha256 = None;
        record.monitor_log_sha256 = Some(digests.admitted_sha256);
        record.commit_ready = true;
        let admitted_json = serde_json::to_string_pretty(&record)
            .context("failed to serialize admitted evidence")?;
        evidence::write_dual_admitted_text(&paths.admitted_record, &admitted_json)
    })();
    if let Err(error) = finalize_result {
        for path in [&paths.admitted_log, &paths.admitted_record] {
            if let Err(remove_error) = fs::remove_file(path.as_std_path()) {
                if remove_error.kind() != std::io::ErrorKind::NotFound {
                    return Err(remove_error).context("failed to roll back admitted evidence");
                }
            }
        }
        return Err(error).context("dual_evidence=failed reason=finalization_failed");
    }
    emit_line("dual_evidence", "finalized")
}

fn run_phase35_probe(
    command: &Phase35ProbeCommand,
    environment: &LocalFlashEnvironment,
) -> Result<()> {
    ensure_ultra_205(command.board)?;
    if command.timeout_seconds == 0 || command.timeout_seconds > 420 {
        bail!("phase35_probe=blocked reason=invalid_timeout");
    }

    let stage_root = environment.workspace_path(&command.stage_root);
    environment
        .approve_private_evidence_root(&stage_root)
        .map_err(|_| anyhow::anyhow!("phase35_probe=blocked reason=root_admission_failed"))?;
    fs::create_dir_all(stage_root.as_std_path())
        .context("failed to create private Phase 35 probe root")?;
    set_private_directory_mode(&stage_root)?;

    let log_path = stage_root.join("probe.private.log");
    let metrics_path = stage_root.join("probe.metrics.json");
    if log_path.exists() || metrics_path.exists() {
        bail!("phase35_probe=blocked reason=destination_exists");
    }

    let command_spec = phase35_probe_command(&environment.espflash_bin, &command.port);
    let started = Instant::now();
    let capture = evidence::capture_command(
        &command_spec,
        &environment.espflash_bin,
        &log_path,
        command.timeout_seconds,
        EvidenceRedactionMode::DeveloperRaw,
        true,
    )?;
    let duration_millis = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let launched = !matches!(capture.status, CaptureProcessStatus::SpawnFailed);
    let success = matches!(capture.status, CaptureProcessStatus::ExitedSuccess);
    let log = fs::read_to_string(log_path.as_std_path())
        .context("failed to read sanitized Phase 35 probe log")?;
    let checksum_observed = phase35_probe_checksum_observed(&log);
    let connected = launched && (success || flash_log_connected(&log));
    let device_info_complete = connected && (success || flash_log_device_info_complete(&log));
    let transfer_started = device_info_complete && checksum_observed;
    let completed = transfer_started && success;
    let metrics = serde_json::json!({
        "schema_version": PHASE35_FLASH_SCHEMA,
        "stage": "probe",
        "tool_version_valid": environment.espflash_version == format!("espflash {ESPFLASH_EXPECTED_VERSION}"),
        "launched": launched,
        "connected": connected,
        "device_info_complete": device_info_complete,
        "transfer_started": transfer_started,
        "completed": completed,
        "duration_millis": if launched { duration_millis } else { 0 },
    });
    let mut encoded = serde_json::to_vec_pretty(&metrics)?;
    encoded.push(b'\n');
    write_private_new_bytes(&metrics_path, &encoded)?;

    if !completed {
        bail!("phase35_probe=failed reason=child_boundary");
    }
    emit_line("phase35_probe", "ready")
}

fn prepare_flash(
    command: &FlashCommand,
    environment: &impl FlashEnvironment,
) -> Result<PreparedFlash> {
    ensure_ultra_205(command.common.board)?;
    let admitted_image = resolve_flash_image(command, environment)?;
    let maybe_execution_snapshot = if command.common.dry_run {
        None
    } else {
        let Some(factory_bytes) = admitted_image.maybe_factory_bytes() else {
            bail!("identity_admission=blocked reason=developer_image_requires_dry_run");
        };
        Some(environment.create_admitted_execution_snapshot(factory_bytes)?)
    };
    let execution_path = maybe_execution_snapshot
        .as_ref()
        .map(AdmittedExecutionSnapshot::path)
        .unwrap_or_else(|| admitted_image.display_path());
    let port = resolve_port(command.common.port.as_deref(), environment)?;
    let execution_command = flash_command_for_admitted_image(
        &port,
        &admitted_image,
        execution_path,
        command.common.dry_run,
    )?;
    let display_command = if maybe_execution_snapshot.is_some() {
        flash_command_for_admitted_image(
            &port,
            &admitted_image,
            Utf8Path::new("<admitted-factory-snapshot>"),
            command.common.dry_run,
        )?
    } else {
        execution_command.clone()
    };
    let nvs_seed = match &command.wifi_credentials {
        Some(path) => Some(prepare_wifi_nvs_seed(&port, path, environment)?),
        None => None,
    };

    Ok(PreparedFlash {
        outcome: FlashOutcome {
            manifest: admitted_image.maybe_manifest().map(Utf8Path::to_owned),
            flash_image: admitted_image.display_path().to_owned(),
            command: display_command,
            nvs_seed,
        },
        execution_command,
        _execution_snapshot: maybe_execution_snapshot,
    })
}

fn flash_command_for_admitted_image(
    port: &str,
    admitted_image: &AdmittedFlashImage,
    execution_path: &Utf8Path,
    dry_run: bool,
) -> Result<CommandSpec> {
    match admitted_image {
        AdmittedFlashImage::Factory(_) => Ok(CommandSpec::new(
            "espflash",
            [
                "write-bin",
                "--chip",
                "esp32s3",
                "--port",
                port,
                "--non-interactive",
                "--before",
                "usb-reset",
                "--after",
                "hard-reset",
                "--skip-update-check",
                "0x0",
                execution_path.as_str(),
            ],
        )),
        AdmittedFlashImage::DeveloperDryRun { .. } if dry_run => Ok(CommandSpec::new(
            "espflash",
            [
                "flash",
                "--chip",
                "esp32s3",
                "--port",
                port,
                execution_path.as_str(),
            ],
        )),
        AdmittedFlashImage::DeveloperDryRun { .. } => {
            bail!("identity_admission=blocked reason=developer_image_requires_dry_run")
        }
    }
}

fn prepare_wifi_nvs_seed(
    port: &str,
    credentials_path: &Utf8Path,
    environment: &impl FlashEnvironment,
) -> Result<NvsSeedOutcome> {
    let credentials_path = environment.workspace_path(credentials_path);
    let credentials = read_wifi_credentials(&credentials_path, environment)?;
    let temp_dir = tempfile::Builder::new()
        .prefix("bitaxe-wifi-nvs-")
        .tempdir()
        .context("failed to create temporary Wi-Fi NVS directory")?;
    let temp_dir_path =
        Utf8PathBuf::from_path_buf(temp_dir.path().to_path_buf()).map_err(|path| {
            anyhow::anyhow!("temporary Wi-Fi NVS directory is not valid UTF-8: {path:?}")
        })?;
    let csv_path = temp_dir_path.join("wifi-nvs.csv");
    let image_path = temp_dir_path.join("wifi-nvs.bin");
    environment.write_file(&csv_path, &wifi_nvs_csv(&credentials))?;
    environment.generate_nvs_partition(&csv_path, &image_path, NVS_PARTITION_SIZE)?;

    Ok(NvsSeedOutcome {
        command: nvs_seed_command_for_image(port, &image_path),
        image: image_path,
        _temp_dir: temp_dir,
    })
}

fn nvs_seed_command_for_image(port: &str, nvs_image: &Utf8Path) -> CommandSpec {
    CommandSpec::new(
        "espflash",
        [
            "write-bin",
            "--chip",
            "esp32s3",
            "--port",
            port,
            "--non-interactive",
            "--before",
            "usb-reset",
            "--after",
            "hard-reset",
            "--skip-update-check",
            NVS_PARTITION_OFFSET,
            nvs_image.as_str(),
        ],
    )
}

#[derive(Debug, Deserialize)]
struct WifiCredentialsFile {
    ssid: String,
    #[serde(rename = "wifiPass")]
    wifi_pass: String,
}

#[derive(Debug, Clone, Eq, PartialEq)]
struct WifiCredentials {
    ssid: String,
    wifi_pass: String,
}

fn read_wifi_credentials(
    path: &Utf8Path,
    environment: &impl FlashEnvironment,
) -> Result<WifiCredentials> {
    let contents = environment
        .read_to_string(path)
        .with_context(|| format!("failed to read Wi-Fi credential file {path}"))?;
    let file: WifiCredentialsFile = serde_json::from_str(&contents)
        .with_context(|| format!("failed to parse Wi-Fi credential file JSON {path}"))?;
    validate_wifi_credentials(file)
}

fn validate_wifi_credentials(file: WifiCredentialsFile) -> Result<WifiCredentials> {
    let patch = SettingsPatch::from_pairs([
        ("ssid", RawSettingValue::String(file.ssid)),
        ("wifiPass", RawSettingValue::String(file.wifi_pass)),
    ]);

    match apply_settings_patch(&patch) {
        SettingsUpdateDecision::Accepted { writes } => Ok(WifiCredentials {
            ssid: string_write_value(&writes, "wifissid")?,
            wifi_pass: string_write_value(&writes, "wifipass")?,
        }),
        SettingsUpdateDecision::Rejected { errors } => {
            bail!(
                "invalid Wi-Fi credentials: {}",
                validation_error_summaries(&errors)
            );
        }
    }
}

fn string_write_value(writes: &[NvsWrite], key_name: &str) -> Result<String> {
    writes
        .iter()
        .find_map(|write| match write {
            NvsWrite::String { key, value } if key.as_str() == key_name => Some(value.clone()),
            _ => None,
        })
        .with_context(|| format!("validated Wi-Fi patch did not produce {key_name} NVS write"))
}

fn validation_error_summaries(errors: &[ConfigValidationError]) -> String {
    errors
        .iter()
        .map(validation_error_summary)
        .collect::<Vec<_>>()
        .join("; ")
}

fn validation_error_summary(error: &ConfigValidationError) -> String {
    match error {
        ConfigValidationError::InvalidLength {
            field,
            min,
            max,
            actual,
        } => format!("{field} length {actual} is outside {min}..={max}"),
        ConfigValidationError::OutOfRange {
            field,
            min,
            max,
            actual,
        } => format!("{field} value {actual} is outside {min}..={max}"),
        ConfigValidationError::InvalidEnum { field, .. } => {
            format!("{field} has an invalid value")
        }
        ConfigValidationError::InvalidBoardScope { .. } => {
            "board version is not active hardware-verified scope".to_owned()
        }
        ConfigValidationError::InvalidNvsKeyName { max_bytes, .. } => {
            format!("NVS key name is invalid; maximum length is {max_bytes} bytes")
        }
    }
}

fn wifi_nvs_csv(credentials: &WifiCredentials) -> String {
    [
        "key,type,encoding,value".to_owned(),
        format!("{NVS_NAMESPACE},namespace,,"),
        format!(
            "wifissid,data,string,{}",
            csv_cell(credentials.ssid.as_str())
        ),
        format!(
            "wifipass,data,string,{}",
            csv_cell(credentials.wifi_pass.as_str())
        ),
    ]
    .join("\n")
        + "\n"
}

fn csv_cell(value: &str) -> String {
    if !value
        .chars()
        .any(|character| matches!(character, ',' | '"' | '\n' | '\r'))
    {
        return value.to_owned();
    }

    format!("\"{}\"", value.replace('"', "\"\""))
}

fn resolve_flash_image(
    command: &FlashCommand,
    environment: &impl FlashEnvironment,
) -> Result<AdmittedFlashImage> {
    if command.common.dry_run && command.manifest.is_none() {
        let Some(image) = &command.image else {
            bail!("identity_admission=blocked reason=dry_run_requires_image_or_v3_manifest");
        };
        return Ok(AdmittedFlashImage::DeveloperDryRun {
            display_path: environment.workspace_path(image),
        });
    }

    if command.image.is_some() && command.manifest.is_none() {
        bail!("identity_admission=blocked reason=explicit_image_requires_v3_manifest");
    }

    if command.manifest.is_none() {
        environment.build_package()?;
    }
    let manifest = match &command.manifest {
        Some(path) => environment.workspace_path(path),
        None => environment
            .bazel_bin()?
            .join(PACKAGE_MANIFEST_RELATIVE_PATH),
    };
    let manifest_contents = environment.read_to_string(&manifest)?;
    let package_manifest: PackageManifest = serde_json::from_str(&manifest_contents)
        .with_context(|| format!("failed to parse package manifest {manifest}"))?;
    let current_provenance = environment.current_provenance()?;
    let admitted_factory = validate_identity_admission(
        &manifest,
        &package_manifest,
        &current_provenance,
        environment,
    )?;
    if let Some(image) = &command.image {
        let explicit_image = environment.workspace_path(image);
        if explicit_image != admitted_factory.display_path {
            bail!("identity_admission=blocked reason=explicit_image_not_admitted_factory");
        }
    }

    Ok(AdmittedFlashImage::Factory(admitted_factory))
}

fn validate_identity_admission(
    manifest_path: &Utf8Path,
    manifest: &PackageManifest,
    current_provenance: &BuildProvenance,
    environment: &impl FlashEnvironment,
) -> Result<AdmittedFactoryImage> {
    if manifest.schema_version != 3 {
        bail!("identity_admission=blocked reason=manifest_schema_not_v3");
    }
    validate_required_artifact_kinds(manifest)?;
    let manifest_provenance = BuildProvenance::new(
        &manifest.semantic_version,
        &manifest.source_commit,
        manifest.build_identity.source_dirty,
        manifest.build_identity.release_tag.as_deref(),
        &manifest.reference_commit,
    )
    .context("identity_admission=blocked reason=manifest_provenance_invalid")?;
    let identity = manifest_provenance.build_identity();
    if manifest.build_identity.label != identity.build_label()
        || manifest.build_identity.channel != identity.build_channel().as_str()
    {
        bail!("identity_admission=blocked reason=manifest_identity_contradictory");
    }
    if identity.source_dirty() {
        bail!("identity_admission=blocked reason=package_source_dirty");
    }
    if current_provenance.build_identity().source_dirty() {
        bail!("identity_admission=blocked reason=current_workspace_dirty");
    }
    if &manifest_provenance != current_provenance {
        bail!("identity_admission=blocked reason=package_workspace_identity_mismatch");
    }
    validate_lower_hex("app_elf_sha256", &manifest.app_elf_sha256, true)?;
    let _ = resolve_manifest_default(manifest_path, Utf8Path::new(&manifest.default_flash_image))?;

    let elf_artifact = require_artifact(manifest, "firmware_elf")?;
    let elf_path = resolve_manifest_sibling(manifest_path, Utf8Path::new(&elf_artifact.path))?;
    let elf_bytes = read_validated_artifact(elf_artifact, &elf_path, environment)?;
    if sha256_bytes(&elf_bytes) != manifest.app_elf_sha256 {
        bail!("identity_admission=blocked reason=firmware_elf_app_sha_mismatch");
    }

    let ota_artifact = require_artifact(manifest, "firmware_ota_image")?;
    let ota_path = resolve_manifest_sibling(manifest_path, Utf8Path::new(&ota_artifact.path))?;
    let ota_bytes = read_validated_artifact(ota_artifact, &ota_path, environment)?;
    let app_elf_sha256 = decode_lower_hex(&manifest.app_elf_sha256)?;
    let factory_artifact = require_artifact(manifest, "factory_merged_image")?;
    let factory_path =
        resolve_manifest_factory_artifact(manifest_path, Utf8Path::new(&factory_artifact.path))?;
    let factory_bytes = read_validated_artifact(factory_artifact, &factory_path, environment)?;
    package_admission::validate_factory_ota_identity(
        &factory_bytes,
        &ota_bytes,
        package_admission::ExpectedApplicationIdentity {
            build_label: &manifest.build_identity.label,
            source_commit: &manifest.source_commit,
            app_elf_sha256: &app_elf_sha256,
        },
    )?;

    Ok(AdmittedFactoryImage {
        manifest: manifest_path.to_owned(),
        display_path: factory_path,
        bytes: factory_bytes,
    })
}

fn require_artifact<'a>(manifest: &'a PackageManifest, kind: &str) -> Result<&'a PackageArtifact> {
    let mut matches = manifest
        .artifacts
        .iter()
        .filter(|artifact| artifact.kind == kind);
    let Some(artifact) = matches.next() else {
        bail!("identity_admission=blocked reason=missing_{kind}_artifact");
    };
    if matches.next().is_some() {
        bail!("identity_admission=blocked reason=duplicate_{kind}_artifact");
    }

    Ok(artifact)
}

fn validate_required_artifact_kinds(manifest: &PackageManifest) -> Result<()> {
    for kind in [
        "firmware_elf",
        "firmware_ota_image",
        "www_spiffs_image",
        "factory_merged_image",
        "partition_table",
        "otadata_initial",
    ] {
        require_artifact(manifest, kind)?;
    }

    Ok(())
}

fn read_validated_artifact(
    artifact: &PackageArtifact,
    path: &Utf8Path,
    environment: &impl FlashEnvironment,
) -> Result<Vec<u8>> {
    validate_lower_hex("artifact_sha256", &artifact.sha256, false)?;
    let bytes = environment.read_bytes(path)?;
    if sha256_bytes(&bytes) != artifact.sha256 {
        bail!("identity_admission=blocked reason=package_artifact_digest_mismatch");
    }
    Ok(bytes)
}

fn validate_lower_hex(label: &str, value: &str, reject_zero: bool) -> Result<()> {
    let valid = value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte));
    if valid && (!reject_zero || value.bytes().any(|byte| byte != b'0')) {
        return Ok(());
    }

    bail!("identity_admission=blocked reason=invalid_{label}")
}

fn decode_lower_hex(value: &str) -> Result<Vec<u8>> {
    validate_lower_hex("app_elf_sha256", value, true)?;
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let high = hex_nibble(pair[0])?;
            let low = hex_nibble(pair[1])?;
            Ok((high << 4) | low)
        })
        .collect()
}

fn hex_nibble(byte: u8) -> Result<u8> {
    match byte {
        b'0'..=b'9' => Ok(byte - b'0'),
        b'a'..=b'f' => Ok(byte - b'a' + 10),
        _ => bail!("identity_admission=blocked reason=invalid_hex_nibble"),
    }
}

fn sha256_bytes(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(digest.len() * 2);
    const HEX: &[u8; 16] = b"0123456789abcdef";
    for byte in digest {
        encoded.push(char::from(HEX[usize::from(byte >> 4)]));
        encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    encoded
}

fn resolve_manifest_default(
    manifest: &Utf8Path,
    default_flash_image: &Utf8Path,
) -> Result<Utf8PathBuf> {
    let Some(file_name) = default_flash_image.file_name() else {
        bail!("default_flash_image must resolve to {DEFAULT_ELF_NAME}");
    };

    if file_name != DEFAULT_ELF_NAME {
        if file_name == FACTORY_IMAGE_NAME {
            bail!(
                "default_flash_image must resolve to {DEFAULT_ELF_NAME}; {FACTORY_IMAGE_NAME} is only an additional artifact"
            );
        }

        bail!("default_flash_image must resolve to {DEFAULT_ELF_NAME}, not {file_name}");
    }

    resolve_manifest_sibling(manifest, default_flash_image)
}

fn resolve_manifest_factory_artifact(
    manifest: &Utf8Path,
    factory_image: &Utf8Path,
) -> Result<Utf8PathBuf> {
    let Some(file_name) = factory_image.file_name() else {
        bail!("factory_merged_image artifact must resolve to {FACTORY_IMAGE_NAME}");
    };

    if file_name != FACTORY_IMAGE_NAME {
        bail!(
            "factory_merged_image artifact must resolve to {FACTORY_IMAGE_NAME}, not {file_name}"
        );
    }

    resolve_manifest_sibling(manifest, factory_image)
}

fn resolve_manifest_sibling(manifest: &Utf8Path, image: &Utf8Path) -> Result<Utf8PathBuf> {
    if image.is_absolute() {
        return Ok(image.to_owned());
    }

    let Some(manifest_dir) = manifest.parent() else {
        bail!("manifest path has no parent directory: {manifest}");
    };

    Ok(manifest_dir.join(image))
}

fn resolve_port(maybe_port: Option<&str>, environment: &impl FlashEnvironment) -> Result<String> {
    if let Some(port) = maybe_port {
        return Ok(port.to_owned());
    }

    let ports_output = environment.list_ports()?;
    let candidates = likely_port_candidates(&ports_output);
    match candidates.len() {
        0 => bail!(
            "No serial ports found. Connect an Ultra 205 over USB or pass an explicit port, for example: --port /dev/cu.usbmodem101"
        ),
        1 => Ok(candidates[0].clone()),
        _ => bail!(
            "Ambiguous serial ports:\n{}",
            candidates
                .iter()
                .map(|port| format!("- use --port {port}"))
                .collect::<Vec<_>>()
                .join("\n")
        ),
    }
}

fn prepare_monitor_command(
    common: &CommonArgs,
    environment: &impl FlashEnvironment,
) -> Result<CommandSpec> {
    ensure_ultra_205(common.board)?;
    let port = resolve_port(common.port.as_deref(), environment)?;
    Ok(CommandSpec::new(
        "espflash",
        ["monitor", "--port", port.as_str()],
    ))
}

fn prepare_evidence_monitor_command(
    common: &CommonArgs,
    environment: &impl FlashEnvironment,
) -> Result<CommandSpec> {
    ensure_ultra_205(common.board)?;
    let port = resolve_port(common.port.as_deref(), environment)?;
    Ok(CommandSpec::new(
        "espflash",
        [
            "monitor",
            "--chip",
            "esp32s3",
            "--port",
            port.as_str(),
            "--non-interactive",
        ],
    ))
}

fn monitor_log_has_trusted_boot_markers(log: &str) -> bool {
    monitor_log_has_message(log, "bitaxe-rust boot: board=Ultra 205 asic=BM1366")
        && monitor_log_has_message(
            log,
            "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled",
        )
        && monitor_log_has_token(log, "spiffs_mount=available")
        && monitor_log_has_token(log, "axeos_api_route_shell=started")
        && [
            "ota_boot_validation=",
            "reset_reason=",
            "firmware_commit=",
            "reference_commit=",
            "esp_idf_version=",
        ]
        .iter()
        .all(|marker| monitor_log_marker_value(log, marker) != UNAVAILABLE)
}

fn monitor_log_has_message(log: &str, marker: &str) -> bool {
    let prefixed_marker = format!(": {marker}");
    log.lines()
        .map(str::trim)
        .any(|line| line == marker || line.ends_with(&prefixed_marker))
}

fn monitor_log_has_token(log: &str, marker: &str) -> bool {
    log.lines()
        .flat_map(str::split_whitespace)
        .any(|token| token == marker)
}

fn monitor_capture_outcome(
    process_status: &CaptureProcessStatus,
    monitor_log: &str,
    capture_timeout_seconds: u64,
    expected_firmware_commit: &str,
    expected_reference_commit: &str,
) -> MonitorCaptureOutcome {
    let observed_firmware_commit = monitor_log_marker_value(monitor_log, "firmware_commit=");
    let observed_reference_commit = monitor_log_marker_value(monitor_log, "reference_commit=");
    let maybe_trust_failure = monitor_trust_failure(
        monitor_log,
        &observed_firmware_commit,
        expected_firmware_commit,
        &observed_reference_commit,
        expected_reference_commit,
    );
    let trusted_output = maybe_trust_failure.is_none();
    let capture_status = match process_status {
        CaptureProcessStatus::ExitedSuccess if trusted_output => CaptureStatus::Completed,
        CaptureProcessStatus::TimedOut if trusted_output => {
            CaptureStatus::TimedOutAfterTrustedOutput
        }
        CaptureProcessStatus::TimedOut => CaptureStatus::TimedOutWithoutTrustedOutput,
        CaptureProcessStatus::SpawnFailed
        | CaptureProcessStatus::ExitedSuccess
        | CaptureProcessStatus::ExitedFailure(_) => CaptureStatus::Failed,
    };
    let conclusion = if trusted_output
        && matches!(
            capture_status,
            CaptureStatus::Completed | CaptureStatus::TimedOutAfterTrustedOutput
        ) {
        "passed - wrapper-owned serial boot evidence captured; HTTP/static/recovery/OTA/rollback parity not claimed".to_owned()
    } else if let Some(trust_failure) = maybe_trust_failure {
        format!("failed - evidence capture is not trusted: {trust_failure}")
    } else {
        "failed - evidence capture is not trusted".to_owned()
    };

    MonitorCaptureOutcome {
        capture_mode: "noninteractive".to_owned(),
        capture_status,
        capture_timeout_seconds,
        trusted_output,
        observed_firmware_commit,
        observed_reference_commit,
        conclusion,
    }
}

fn monitor_log_marker_value(log: &str, marker: &str) -> String {
    log.lines()
        .flat_map(str::split_whitespace)
        .find_map(|token| token.strip_prefix(marker))
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .unwrap_or_else(|| UNAVAILABLE.to_owned())
}

fn monitor_trust_failure(
    monitor_log: &str,
    observed_firmware_commit: &str,
    expected_firmware_commit: &str,
    observed_reference_commit: &str,
    expected_reference_commit: &str,
) -> Option<String> {
    if !monitor_log_has_trusted_boot_markers(monitor_log) {
        return Some("missing trusted Ultra 205 boot markers".to_owned());
    }

    if !commit_marker_matches_expected(observed_firmware_commit, expected_firmware_commit) {
        return Some(format!(
            "observed firmware_commit={observed_firmware_commit} did not match source commit={expected_firmware_commit}"
        ));
    }

    if !commit_marker_matches_expected(observed_reference_commit, expected_reference_commit) {
        return Some(format!(
            "observed reference_commit={observed_reference_commit} did not match reference commit={expected_reference_commit}"
        ));
    }

    None
}

fn commit_marker_matches_expected(observed: &str, expected: &str) -> bool {
    observed != UNAVAILABLE
        && expected != UNAVAILABLE
        && observed.len() >= MIN_COMMIT_PREFIX_LEN
        && observed.len() <= expected.len()
        && observed
            .chars()
            .all(|character| character.is_ascii_hexdigit())
        && expected
            .chars()
            .all(|character| character.is_ascii_hexdigit())
        && expected.starts_with(observed)
}

fn dry_run_monitor_capture_outcome(capture_timeout_seconds: u64) -> MonitorCaptureOutcome {
    MonitorCaptureOutcome {
        capture_mode: "dry_run".to_owned(),
        capture_status: CaptureStatus::DryRun,
        capture_timeout_seconds,
        trusted_output: false,
        observed_firmware_commit: UNAVAILABLE.to_owned(),
        observed_reference_commit: UNAVAILABLE.to_owned(),
        conclusion: "not run - dry-run did not capture hardware evidence".to_owned(),
    }
}

fn no_monitor_capture_outcome() -> MonitorCaptureOutcome {
    MonitorCaptureOutcome {
        capture_mode: "not_applicable".to_owned(),
        capture_status: CaptureStatus::DryRun,
        capture_timeout_seconds: 0,
        trusted_output: false,
        observed_firmware_commit: UNAVAILABLE.to_owned(),
        observed_reference_commit: UNAVAILABLE.to_owned(),
        conclusion: "not run - no monitor capture requested".to_owned(),
    }
}

fn evidence_capture_failure_guidance(port: &str, evidence_dir: &Utf8Path) -> String {
    [
        "evidence capture failed and is not trusted".to_owned(),
        "rerun: just detect-ultra205".to_owned(),
        format!("rerun: just flash-monitor board=205 port={port} evidence-dir={evidence_dir}"),
        format!("diagnostic only: just monitor port={port}"),
        "use the wrapper noninteractive evidence path before treating serial logs as proof"
            .to_owned(),
    ]
    .join("\n")
}

fn likely_port_candidates(ports_output: &str) -> Vec<String> {
    let mut candidates = BTreeSet::new();
    for token in ports_output.split_whitespace() {
        let port = token.trim_matches(|character: char| {
            matches!(character, ',' | ';' | ':' | '(' | ')' | '[' | ']')
        });

        if is_likely_port(port) {
            candidates.insert(port.to_owned());
        }
    }

    candidates.into_iter().collect()
}

fn is_likely_port(port: &str) -> bool {
    if port.starts_with("/dev/cu.usbmodem")
        || port.starts_with("/dev/cu.usbserial")
        || port.starts_with("/dev/ttyUSB")
        || port.starts_with("/dev/ttyACM")
    {
        return true;
    }

    let Some(suffix) = port.strip_prefix("COM") else {
        return false;
    };

    !suffix.is_empty() && suffix.chars().all(|character| character.is_ascii_digit())
}

fn ensure_ultra_205(board: BoardId) -> Result<()> {
    if board != BoardId::Ultra205 {
        bail!("Phase 1 supports board=205 only");
    }

    Ok(())
}

fn emit_flash_outcome(outcome: &FlashOutcome, expose_operational: bool) -> Result<()> {
    if !expose_operational {
        if outcome.manifest.is_some() {
            emit_line("manifest", operational_console_value("", false))?;
        }
        emit_line("flash_image", operational_console_value("", false))?;
        emit_line("flash_command", operational_console_value("", false))?;
        if outcome.nvs_seed.is_some() {
            emit_line("nvs_seed_status", "provided")?;
            emit_line("nvs_seed_image", operational_console_value("", false))?;
            emit_line("nvs_seed_command", operational_console_value("", false))?;
        }
        return Ok(());
    }
    if let Some(manifest) = &outcome.manifest {
        emit_line("manifest", manifest.as_str())?;
    }
    emit_line("flash_image", outcome.flash_image.as_str())?;
    emit_command("flash_command", &outcome.command)?;
    if let Some(nvs_seed) = &outcome.nvs_seed {
        emit_line("nvs_seed_status", "provided")?;
        emit_line("nvs_seed_image", nvs_seed.image.as_str())?;
        emit_command("nvs_seed_command", &nvs_seed.command)?;
    }
    Ok(())
}

fn emit_operational_command(
    label: &str,
    command: &CommandSpec,
    expose_operational: bool,
) -> Result<()> {
    if expose_operational {
        return emit_command(label, command);
    }
    emit_line(label, operational_console_value("", false))
}

fn operational_console_value(value: &str, expose_operational: bool) -> &str {
    if expose_operational {
        return value;
    }
    PROTECTED_OPERATIONAL
}

fn emit_command(label: &str, command: &CommandSpec) -> Result<()> {
    emit_line(label, &command.display())
}

fn emit_line(label: &str, value: &str) -> Result<()> {
    let mut stdout = io::stdout().lock();
    writeln!(stdout, "{label}: {value}").context("failed to write command output")
}

fn write_evidence_if_requested(
    common: &CommonArgs,
    outcome: &FlashOutcome,
    command_kind: &str,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let Some(evidence_dir) = resolved_evidence_dir(common, environment) else {
        return Ok(());
    };

    let log_path = evidence_dir.join("flash-monitor.log");
    let capture_outcome = no_monitor_capture_outcome();
    let command_display = flash_workflow_command(outcome);
    let flash_command_display = outcome.command.display();
    write_evidence_record(
        common,
        outcome,
        &evidence_dir,
        EvidenceRecordInput {
            command_kind,
            command: &command_display,
            flash_command: &flash_command_display,
            monitor_command: UNAVAILABLE,
            log_path: &log_path,
            private_log_path: None,
            private_log_sha256: None,
            admitted_log_sha256: None,
            capture_outcome: &capture_outcome,
        },
        environment,
    )
}

fn write_flash_monitor_evidence_if_requested(
    common: &CommonArgs,
    outcome: &FlashOutcome,
    monitor_command: &CommandSpec,
    evidence_dir: &Utf8Path,
    artifacts: MonitorEvidenceArtifacts<'_>,
    capture_outcome: &MonitorCaptureOutcome,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let flash_workflow_command = flash_workflow_command(outcome);
    let monitor_command_display = monitor_command.display();
    let command = format!("{flash_workflow_command}\nmonitor: {monitor_command_display}");
    let flash_command_display = outcome.command.display();
    write_evidence_record(
        common,
        outcome,
        evidence_dir,
        EvidenceRecordInput {
            command_kind: "flash-monitor",
            command: &command,
            flash_command: &flash_command_display,
            monitor_command: &monitor_command_display,
            log_path: artifacts.admitted_log,
            private_log_path: artifacts
                .dual_paths
                .map(|paths| paths.private_log.as_path()),
            private_log_sha256: artifacts.private_log_sha256,
            admitted_log_sha256: None,
            capture_outcome,
        },
        environment,
    )
}

fn write_evidence_record(
    common: &CommonArgs,
    outcome: &FlashOutcome,
    evidence_dir: &Utf8Path,
    input: EvidenceRecordInput<'_>,
    environment: &impl FlashEnvironment,
) -> Result<()> {
    let redaction_mode = EvidenceRedactionMode::from_common(common);
    let dual_mode = common.evidence_mode == Some(EvidenceMode::Dual);
    let record = EvidenceRecord {
        command: input.command.to_owned(),
        command_kind: input.command_kind.to_owned(),
        board: common.board.to_string(),
        port: command_port(&outcome.command).unwrap_or_else(|| UNAVAILABLE.to_owned()),
        firmware_commit: environment.firmware_commit(),
        reference_commit: environment.reference_commit(),
        manifest_path: outcome
            .manifest
            .as_ref()
            .map(|path| path.as_str().to_owned())
            .unwrap_or_else(|| UNAVAILABLE.to_owned()),
        flash_image_path: outcome.flash_image.as_str().to_owned(),
        timestamp: unix_timestamp(),
        log_path: input.log_path.as_str().to_owned(),
        flash_command: input.flash_command.to_owned(),
        monitor_command: input.monitor_command.to_owned(),
        nvs_seed_status: if outcome.nvs_seed.is_some() {
            "provided".to_owned()
        } else {
            "not_provided".to_owned()
        },
        nvs_seed_command: outcome
            .nvs_seed
            .as_ref()
            .map(|seed| seed.command.display())
            .unwrap_or_else(|| UNAVAILABLE.to_owned()),
        nvs_seed_partition_offset: if outcome.nvs_seed.is_some() {
            NVS_PARTITION_OFFSET.to_owned()
        } else {
            UNAVAILABLE.to_owned()
        },
        nvs_seed_partition_size: if outcome.nvs_seed.is_some() {
            NVS_PARTITION_SIZE.to_owned()
        } else {
            UNAVAILABLE.to_owned()
        },
        redaction_mode: if dual_mode {
            "dual".to_owned()
        } else {
            redaction_mode.as_str().to_owned()
        },
        commit_ready: !dual_mode && redaction_mode.commit_ready(),
        wifi_credentials_source: if outcome.nvs_seed.is_some() {
            "provided-redacted".to_owned()
        } else {
            "not-provided".to_owned()
        },
        monitor_log_path: input.log_path.as_str().to_owned(),
        private_log_role: input
            .private_log_path
            .map(|_| "classifier-input-private".to_owned()),
        private_monitor_log_path: input.private_log_path.map(|path| path.as_str().to_owned()),
        private_monitor_log_sha256: input.private_log_sha256.map(str::to_owned),
        monitor_log_sha256: input.admitted_log_sha256.map(str::to_owned),
        capture_mode: input.capture_outcome.capture_mode.clone(),
        capture_status: input.capture_outcome.capture_status,
        capture_timeout_seconds: input.capture_outcome.capture_timeout_seconds,
        trusted_output: input.capture_outcome.trusted_output,
        observed_firmware_commit: input.capture_outcome.observed_firmware_commit.clone(),
        observed_reference_commit: input.capture_outcome.observed_reference_commit.clone(),
        conclusion: input.capture_outcome.conclusion.clone(),
    };
    if dual_mode {
        let paths = evidence::DualEvidencePaths {
            private_log: input
                .private_log_path
                .context("dual evidence record requires private log path")?
                .to_owned(),
            admitted_log: input.log_path.to_owned(),
            private_record: evidence_dir.join("flash-command-evidence.private.json"),
            admitted_record: evidence_dir.join("flash-command-evidence.json"),
        };
        let private_json = serde_json::to_string_pretty(&record)
            .context("failed to serialize private evidence")?;
        evidence::write_dual_private_text(&paths.private_record, &private_json)?;
        return Ok(());
    }

    let json = serde_json::to_string_pretty(&record).context("failed to serialize evidence")?;
    environment.write_evidence(
        &evidence_dir.join("flash-command-evidence.json"),
        &sanitize_evidence_text(&json, redaction_mode),
    )
}

fn flash_workflow_command(outcome: &FlashOutcome) -> String {
    let flash = format!("flash: {}", outcome.command.display());
    let Some(nvs_seed) = &outcome.nvs_seed else {
        return flash;
    };

    format!("{flash}\nnvs_seed: {}", nvs_seed.command.display())
}

fn sanitize_evidence_text(text: &str, redaction_mode: EvidenceRedactionMode) -> String {
    const NEVER_PERSIST_FIELDS: &[&str] = &[
        "wifiPass",
        "wifipass",
        "wifi_password",
        "password",
        "pass",
        "token",
        "apiKey",
        "api_key",
        "pool_password",
        "poolPassword",
        "stratumPassword",
        "nvsSecret",
        "secret",
        "poolURL",
        "poolPort",
        "poolUser",
        "poolWorker",
        "worker",
        "ownerAddress",
        "btcAddress",
    ];
    let without_secret_json_fields = redact_json_string_fields(text, NEVER_PERSIST_FIELDS);
    let without_secret_json_scalars =
        redact_json_scalar_fields(&without_secret_json_fields, NEVER_PERSIST_FIELDS);
    let without_secret_tokens =
        redact_key_value_tokens(&without_secret_json_scalars, NEVER_PERSIST_FIELDS);

    if redaction_mode == EvidenceRedactionMode::DeveloperRaw {
        return without_secret_tokens;
    }

    let without_network_json_fields =
        redact_json_string_fields(&without_secret_tokens, &["ssid", "hostname", "hostName"]);
    let without_urls = redact_urls(&without_network_json_fields);
    let without_macs = redact_mac_addresses(&without_urls);
    let without_ips = redact_ipv4_addresses(&without_macs);
    let without_wifi_driver_ssids = redact_wifi_driver_connected_ssids(&without_ips);
    let without_operational_tokens = redact_key_value_tokens(
        &without_wifi_driver_ssids,
        &[
            "ssid",
            "SSID",
            "hostname",
            "hostName",
            "pid",
            "pgid",
            "USB_serial",
            "usb_serial",
            "USB-serial",
        ],
    );
    let without_local_paths = redact_local_paths(&without_operational_tokens);
    redact_http_metadata(&without_local_paths)
}

fn redact_json_scalar_fields(text: &str, fields: &[&str]) -> String {
    fields.iter().fold(text.to_owned(), |sanitized, field| {
        redact_json_scalar_field(&sanitized, field)
    })
}

fn redact_json_scalar_field(text: &str, field: &str) -> String {
    let pattern = format!("\"{field}\"");
    let mut output = String::with_capacity(text.len());
    let mut index = 0;
    while index < text.len() {
        let Some(relative_start) = text[index..].find(&pattern) else {
            output.push_str(&text[index..]);
            break;
        };
        let field_start = index + relative_start;
        let field_end = field_start + pattern.len();
        output.push_str(&text[index..field_end]);
        let mut cursor = field_end;
        while text
            .as_bytes()
            .get(cursor)
            .is_some_and(u8::is_ascii_whitespace)
        {
            cursor += 1;
        }
        if text.as_bytes().get(cursor) != Some(&b':') {
            index = field_end;
            continue;
        }
        cursor += 1;
        while text
            .as_bytes()
            .get(cursor)
            .is_some_and(u8::is_ascii_whitespace)
        {
            cursor += 1;
        }
        if text.as_bytes().get(cursor) == Some(&b'"') {
            index = field_end;
            continue;
        }
        output.push_str(&text[field_end..cursor]);
        output.push_str("\"[redacted]\"");
        while let Some(byte) = text.as_bytes().get(cursor) {
            if matches!(byte, b',' | b'}' | b']') || byte.is_ascii_whitespace() {
                break;
            }
            cursor += 1;
        }
        index = cursor;
    }
    output
}

fn redact_local_paths(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let mut index = 0;
    while index < text.len() {
        let rest = &text[index..];
        let is_unix_path = ["/Users/", "/home/", "/dev/cu", "/dev/tty"]
            .iter()
            .any(|prefix| rest.starts_with(prefix));
        let is_windows_path = rest.as_bytes().get(1) == Some(&b':')
            && rest.as_bytes().first().is_some_and(u8::is_ascii_alphabetic)
            && matches!(rest.as_bytes().get(2), Some(b'\\'));
        if is_unix_path || is_windows_path {
            output.push_str("[redacted-path]");
            while index < text.len() {
                let character = text[index..].chars().next().expect("character");
                if character.is_whitespace() || matches!(character, '"' | '\'' | ',' | '}') {
                    break;
                }
                index += character.len_utf8();
            }
            continue;
        }
        let character = rest.chars().next().expect("character");
        output.push(character);
        index += character.len_utf8();
    }
    output
}

fn redact_http_metadata(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    for line in text.split_inclusive('\n') {
        let line_without_protocol = if let Some(index) = line.find("HTTP/") {
            let protocol_end = line[index..]
                .find(char::is_whitespace)
                .map(|end| index + end)
                .unwrap_or(line.len());
            format!("{}[redacted-http]{}", &line[..index], &line[protocol_end..])
        } else {
            line.to_owned()
        };
        if line_without_protocol
            .trim_start()
            .to_ascii_lowercase()
            .starts_with("host:")
        {
            let leading = line_without_protocol.len() - line_without_protocol.trim_start().len();
            let newline = if line_without_protocol.ends_with('\n') {
                "\n"
            } else {
                ""
            };
            output.push_str(&line_without_protocol[..leading]);
            output.push_str("Host: [redacted]");
            output.push_str(newline);
        } else {
            output.push_str(&line_without_protocol);
        }
    }
    output
}

fn redact_wifi_driver_connected_ssids(text: &str) -> String {
    const MARKER: &str = "wifi:connected with ";
    const AID_DELIMITER: &str = ", aid =";

    let mut output = String::with_capacity(text.len());
    let mut index = 0;

    while index < text.len() {
        let Some(relative_start) = text[index..].find(MARKER) else {
            output.push_str(&text[index..]);
            break;
        };

        let marker_start = index + relative_start;
        let ssid_start = marker_start + MARKER.len();
        output.push_str(&text[index..ssid_start]);
        output.push_str("[redacted-ssid]");

        let remaining = &text[ssid_start..];
        let relative_end = remaining
            .find(AID_DELIMITER)
            .or_else(|| remaining.find('\n'))
            .unwrap_or(remaining.len());
        index = ssid_start + relative_end;
    }

    output
}

fn redact_json_string_fields(text: &str, fields: &[&str]) -> String {
    fields.iter().fold(text.to_owned(), |sanitized, field| {
        redact_json_string_field(&sanitized, field)
    })
}

fn redact_json_string_field(text: &str, field: &str) -> String {
    let pattern = format!("\"{field}\"");
    let mut output = String::with_capacity(text.len());
    let mut index = 0;

    while index < text.len() {
        let Some(relative_start) = text[index..].find(&pattern) else {
            output.push_str(&text[index..]);
            break;
        };

        let field_start = index + relative_start;
        let field_end = field_start + pattern.len();
        output.push_str(&text[index..field_start]);

        let Some((value_open, value_close)) = json_string_value_bounds(text, field_end) else {
            output.push_str(&text[field_start..field_end]);
            index = field_end;
            continue;
        };

        output.push_str(&text[field_start..=value_open]);
        output.push_str("[redacted]");
        output.push('"');
        index = value_close + 1;
    }

    output
}

fn json_string_value_bounds(text: &str, after_field: usize) -> Option<(usize, usize)> {
    let bytes = text.as_bytes();
    let mut cursor = after_field;
    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }

    if bytes.get(cursor) != Some(&b':') {
        return None;
    }
    cursor += 1;

    while cursor < bytes.len() && bytes[cursor].is_ascii_whitespace() {
        cursor += 1;
    }

    if bytes.get(cursor) != Some(&b'"') {
        return None;
    }
    let value_open = cursor;
    cursor += 1;

    while cursor < bytes.len() {
        match bytes[cursor] {
            b'\\' => cursor += 2,
            b'"' => return Some((value_open, cursor)),
            _ => cursor += 1,
        }
    }

    None
}

fn redact_urls(text: &str) -> String {
    const URL_SCHEMES: [&str; 4] = ["http://", "https://", "ws://", "wss://"];

    let mut output = String::with_capacity(text.len());
    let mut index = 0;
    while index < text.len() {
        let rest = &text[index..];
        if let Some(scheme) = URL_SCHEMES.iter().find(|scheme| rest.starts_with(**scheme)) {
            output.push_str("[redacted-url]");
            index += scheme.len();
            while index < text.len() {
                let character = text[index..].chars().next().expect("character");
                if is_url_delimiter(character) {
                    break;
                }
                index += character.len_utf8();
            }
            continue;
        }

        let character = rest.chars().next().expect("character");
        output.push(character);
        index += character.len_utf8();
    }

    output
}

fn is_url_delimiter(character: char) -> bool {
    character.is_whitespace()
        || matches!(
            character,
            '"' | '\'' | '<' | '>' | ')' | '(' | '[' | ']' | '{' | '}'
        )
}

fn redact_ipv4_addresses(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut index = 0;

    while index < bytes.len() {
        if bytes[index].is_ascii_digit() {
            let start = index;
            while index < bytes.len() && (bytes[index].is_ascii_digit() || bytes[index] == b'.') {
                index += 1;
            }
            let token = &text[start..index];
            if is_ipv4_address(token) {
                output.push_str("[redacted-ip]");
            } else {
                output.push_str(token);
            }
            continue;
        }

        let character = text[index..].chars().next().expect("character");
        output.push(character);
        index += character.len_utf8();
    }

    output
}

fn is_ipv4_address(token: &str) -> bool {
    let parts = token.split('.').collect::<Vec<_>>();
    if parts.len() != 4 {
        return false;
    }

    parts.iter().all(|part| {
        !part.is_empty()
            && part.len() <= 3
            && part.chars().all(|character| character.is_ascii_digit())
            && part.parse::<u8>().is_ok()
    })
}

fn redact_mac_addresses(text: &str) -> String {
    let bytes = text.as_bytes();
    let mut output = String::with_capacity(text.len());
    let mut index = 0;

    while index < bytes.len() {
        if is_mac_address_at(bytes, index) {
            output.push_str("[redacted-mac]");
            index += 17;
            continue;
        }

        let character = text[index..].chars().next().expect("character");
        output.push(character);
        index += character.len_utf8();
    }

    output
}

fn is_mac_address_at(bytes: &[u8], index: usize) -> bool {
    if index + 17 > bytes.len() {
        return false;
    }

    if index > 0 && bytes[index - 1].is_ascii_hexdigit() {
        return false;
    }

    if index + 17 < bytes.len() && bytes[index + 17].is_ascii_hexdigit() {
        return false;
    }

    for offset in 0..17 {
        let byte = bytes[index + offset];
        if matches!(offset, 2 | 5 | 8 | 11 | 14) {
            if byte != b':' {
                return false;
            }
        } else if !byte.is_ascii_hexdigit() {
            return false;
        }
    }

    true
}

fn redact_key_value_tokens(text: &str, keys: &[&str]) -> String {
    keys.iter().fold(text.to_owned(), |sanitized, key| {
        redact_key_value_token(&sanitized, key)
    })
}

fn redact_key_value_token(text: &str, key: &str) -> String {
    let pattern = format!("{key}=");
    let mut output = String::with_capacity(text.len());
    let mut index = 0;

    while index < text.len() {
        let rest = &text[index..];
        if rest.starts_with(&pattern) {
            output.push_str(&pattern);
            output.push_str("[redacted]");
            index += pattern.len();
            while index < text.len() {
                let character = text[index..].chars().next().expect("character");
                if character.is_whitespace() {
                    break;
                }
                index += character.len_utf8();
            }
            continue;
        }

        let character = rest.chars().next().expect("character");
        output.push(character);
        index += character.len_utf8();
    }

    output
}

fn resolved_evidence_dir(
    common: &CommonArgs,
    environment: &impl FlashEnvironment,
) -> Option<Utf8PathBuf> {
    common
        .evidence_dir
        .as_deref()
        .map(|path| environment.workspace_path(path))
}

fn command_port(command: &CommandSpec) -> Option<String> {
    command
        .args
        .windows(2)
        .find(|window| window[0] == "--port")
        .map(|window| window[1].clone())
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct EvidenceRecord {
    command: String,
    command_kind: String,
    board: String,
    port: String,
    firmware_commit: String,
    reference_commit: String,
    manifest_path: String,
    flash_image_path: String,
    timestamp: String,
    log_path: String,
    flash_command: String,
    monitor_command: String,
    nvs_seed_status: String,
    nvs_seed_command: String,
    nvs_seed_partition_offset: String,
    nvs_seed_partition_size: String,
    redaction_mode: String,
    commit_ready: bool,
    wifi_credentials_source: String,
    monitor_log_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    private_log_role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    private_monitor_log_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    private_monitor_log_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    monitor_log_sha256: Option<String>,
    capture_mode: String,
    capture_status: CaptureStatus,
    capture_timeout_seconds: u64,
    trusted_output: bool,
    observed_firmware_commit: String,
    observed_reference_commit: String,
    conclusion: String,
}

fn unix_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs().to_string())
        .unwrap_or_else(|_| UNAVAILABLE.to_owned())
}

fn parse_board(value: &str) -> std::result::Result<BoardId, String> {
    value.parse()
}

fn parse_utf8_path(value: &str) -> std::result::Result<Utf8PathBuf, String> {
    Ok(Utf8PathBuf::from(value))
}

fn parse_sha256(value: &str) -> std::result::Result<String, String> {
    if value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Ok(value.to_ascii_lowercase());
    }
    Err("expected a 64-character SHA-256 digest".to_owned())
}

fn command_output_to_string(output: std::process::Output, description: &str) -> Result<String> {
    if !output.status.success() {
        bail!(
            "{description} failed: {}",
            command_stderr_or_status(&output)
        );
    }

    let stdout = String::from_utf8(output.stdout)
        .with_context(|| format!("{description} output was not valid UTF-8"))?;
    Ok(stdout.trim().to_owned())
}

fn command_stderr_or_status(output: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&output.stderr);
    let trimmed_stderr = stderr.trim();
    if !trimmed_stderr.is_empty() {
        return trimmed_stderr.to_owned();
    }

    format!("exit status {}", output.status)
}

fn detect_workspace_dir() -> Result<Utf8PathBuf> {
    if let Ok(workspace_dir) = env::var("BUILD_WORKSPACE_DIRECTORY") {
        if !workspace_dir.is_empty() {
            return Ok(Utf8PathBuf::from(workspace_dir));
        }
    }

    let output = Command::new("git")
        .arg("rev-parse")
        .arg("--show-toplevel")
        .output()
        .context("failed to detect workspace directory with git rev-parse --show-toplevel")?;

    command_output_to_string(output, "git rev-parse --show-toplevel").map(Utf8PathBuf::from)
}

fn resolve_espflash_executable() -> Result<Utf8PathBuf> {
    let requested = env::var("ESPFLASH_BIN").unwrap_or_else(|_| "espflash".to_owned());
    let requested_path = Utf8Path::new(&requested);
    let candidate = if requested_path.components().count() > 1 || requested_path.is_absolute() {
        requested_path.to_owned()
    } else {
        env::split_paths(&env::var_os("PATH").unwrap_or_default())
            .map(|directory| directory.join(&requested))
            .find(|path| path.is_file())
            .and_then(|path| Utf8PathBuf::from_path_buf(path).ok())
            .context("espflash executable not found")?
    };
    let canonical = fs::canonicalize(candidate.as_std_path())
        .context("failed to canonicalize espflash executable")?;
    let canonical = Utf8PathBuf::from_path_buf(canonical)
        .map_err(|_| anyhow::anyhow!("espflash executable path is not UTF-8"))?;
    let metadata = fs::metadata(canonical.as_std_path())?;
    if !metadata.is_file() {
        bail!("espflash executable is not a regular file");
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if metadata.permissions().mode() & 0o111 == 0 {
            bail!("espflash executable is not executable");
        }
    }
    Ok(canonical)
}

fn phase35_stage_for_command(command_spec: &CommandSpec) -> Option<&'static str> {
    if command_spec.args.first().map(String::as_str) != Some("write-bin") {
        return None;
    }
    if command_spec.args.iter().any(|argument| argument == "0x0") {
        return Some("factory");
    }
    if command_spec
        .args
        .iter()
        .any(|argument| argument == NVS_PARTITION_OFFSET)
    {
        return Some("nvs");
    }
    None
}

fn flash_log_connected(log: &str) -> bool {
    ["Connected to device", "Chip type:", "Chip:"]
        .iter()
        .any(|marker| log.contains(marker))
}

fn flash_log_device_info_complete(log: &str) -> bool {
    ["Flash size:", "Crystal frequency:", "MAC address:"]
        .iter()
        .any(|marker| log.contains(marker))
}

fn flash_log_transfer_started(log: &str) -> bool {
    [
        "Writing at",
        "Writing to",
        "Erasing",
        "Reading at",
        "checksum-md5",
    ]
    .iter()
    .any(|marker| log.contains(marker))
}

fn phase35_probe_checksum_observed(log: &str) -> bool {
    log.lines().any(|line| {
        let maybe_checksum = line.trim().strip_prefix("0x");
        matches!(
            maybe_checksum,
            Some(checksum)
                if checksum.len() == 32
                    && checksum.bytes().all(|byte| byte.is_ascii_hexdigit())
        )
    })
}

fn phase35_probe_command(espflash_bin: &Utf8Path, port: &str) -> CommandSpec {
    CommandSpec::new(
        espflash_bin.as_str(),
        [
            "checksum-md5",
            "--chip",
            "esp32s3",
            "--port",
            port,
            "--non-interactive",
            "--before",
            "usb-reset",
            "--after",
            "hard-reset",
            "--skip-update-check",
            "0x0",
            "4096",
        ],
    )
}

fn is_lower_hex_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_phase35_readiness_output(output: &str) -> Result<()> {
    let mut keys = BTreeSet::new();
    let mut category_ready = false;
    let mut digest_count = 0;
    for line in output.lines() {
        let Some((key, value)) = line.split_once('=') else {
            bail!("phase35_stage_readiness=failed reason=output_invalid");
        };
        if !keys.insert(key) {
            bail!("phase35_stage_readiness=failed reason=output_invalid");
        }
        match key {
            "category" if value == "ready" => category_ready = true,
            "combined_identity" | "physical_identity" | "enumeration_identity"
                if is_lower_hex_digest(value) =>
            {
                digest_count += 1;
            }
            _ => bail!("phase35_stage_readiness=failed reason=output_invalid"),
        }
    }
    if keys.len() != 4 || !category_ready || digest_count != 3 {
        bail!("phase35_stage_readiness=failed reason=output_invalid");
    }
    Ok(())
}

fn set_private_directory_mode(path: &Utf8Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path.as_std_path(), fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

fn write_private_new_bytes(path: &Utf8Path, bytes: &[u8]) -> Result<()> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path.as_std_path())?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

fn git_output<const N: usize>(workspace_dir: &Utf8Path, args: [&str; N]) -> Option<String> {
    let output = Command::new("git")
        .current_dir(workspace_dir.as_std_path())
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let stdout = String::from_utf8(output.stdout).ok()?;
    let trimmed = stdout.trim();
    if trimmed.is_empty() {
        return None;
    }

    Some(trimmed.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::{Cell, RefCell};
    use tempfile::{tempdir, TempDir};

    const SOURCE_COMMIT: &str = "0123456789abcdef0123456789abcdef01234567";
    const REFERENCE_COMMIT: &str = "abcdef0123456789abcdef0123456789abcdef01";
    const BUILD_LABEL: &str = "0123456789ab-dev";
    const APP_ELF_SHA256: &str = "ca16ef5bd57d7e4b2f2f016ffb9236c426e68f16072bc1c5a53ef0e515f1d063";

    #[test]
    fn phase35_probe_parses_bounded_read_contract() {
        // Arrange
        let args = [
            "bitaxe-flash",
            "phase35-probe",
            "board=205",
            "port=/dev/cu.usbmodem101",
            "stage-root=scratch/probe",
            "timeout-seconds=30",
        ];

        // Act
        let cli = parse_cli(args).expect("probe cli");

        // Assert
        let CliCommand::Phase35Probe(command) = cli.command else {
            panic!("expected Phase 35 probe command");
        };
        assert_eq!(command.board, BoardId::Ultra205);
        assert_eq!(command.port, "/dev/cu.usbmodem101");
        assert_eq!(command.stage_root, Utf8PathBuf::from("scratch/probe"));
        assert_eq!(command.timeout_seconds, 30);
    }

    #[test]
    fn phase35_probe_checksum_requires_one_complete_md5_line() {
        // Arrange
        let valid = "Connecting...\n0x0123456789abcdef0123456789abcdef\n";
        let truncated = "0x0123456789abcdef\n";
        let embedded = "checksum=0x0123456789abcdef0123456789abcdef\n";

        // Act and Assert
        assert!(phase35_probe_checksum_observed(valid));
        assert!(!phase35_probe_checksum_observed(truncated));
        assert!(!phase35_probe_checksum_observed(embedded));
    }

    #[test]
    fn phase35_probe_command_is_bounded_read_only_and_reset_explicit() {
        // Arrange
        let executable = Utf8Path::new("/opt/espflash");

        // Act
        let command = phase35_probe_command(executable, "/dev/private-device");

        // Assert
        assert_eq!(command.program, "/opt/espflash");
        assert_eq!(
            command.args,
            vec![
                "checksum-md5",
                "--chip",
                "esp32s3",
                "--port",
                "/dev/private-device",
                "--non-interactive",
                "--before",
                "usb-reset",
                "--after",
                "hard-reset",
                "--skip-update-check",
                "0x0",
                "4096",
            ]
        );
        assert!(!command.args.iter().any(|argument| {
            matches!(argument.as_str(), "write-bin" | "flash" | "erase-flash")
        }));
    }

    #[test]
    fn phase35_readiness_output_rejects_missing_duplicate_or_raw_fields() {
        // Arrange
        let digest = "a".repeat(64);
        let valid = format!(
            "category=ready\ncombined_identity={digest}\nphysical_identity={digest}\nenumeration_identity={digest}\n"
        );
        let missing =
            format!("category=ready\ncombined_identity={digest}\nphysical_identity={digest}\n");
        let raw = format!(
            "category=ready\ncombined_identity={digest}\nphysical_identity={digest}\nenumeration_identity={digest}\nport=/dev/private\n"
        );

        // Act and Assert
        assert!(validate_phase35_readiness_output(&valid).is_ok());
        assert!(validate_phase35_readiness_output(&missing).is_err());
        assert!(validate_phase35_readiness_output(&raw).is_err());
    }

    #[cfg(unix)]
    #[test]
    fn phase35_probe_real_process_uses_private_sanitized_no_clobber_artifacts() {
        use std::os::unix::fs::PermissionsExt;

        // Arrange
        let dir = tempdir().expect("tempdir");
        let workspace =
            Utf8PathBuf::from_path_buf(fs::canonicalize(dir.path()).expect("canonical tempdir"))
                .expect("UTF-8 tempdir");
        let git_status = Command::new("git")
            .current_dir(workspace.as_std_path())
            .args(["init", "--quiet"])
            .status()
            .expect("git init");
        assert!(git_status.success());
        fs::write(workspace.join(".gitignore").as_std_path(), "scratch/\n").expect("gitignore");
        let bin_dir = workspace.join("bin");
        fs::create_dir_all(bin_dir.as_std_path()).expect("bin dir");
        let espflash = bin_dir.join("espflash");
        fs::write(
            espflash.as_std_path(),
            "#!/usr/bin/env sh\nprintf '%s\\n' \"$@\" >\"$(dirname \"$0\")/args.log\"\nprintf 'password=probe-secret\\n' >&2\nprintf 'Connecting...\\n0x0123456789abcdef0123456789abcdef\\n'\n",
        )
        .expect("fake espflash");
        fs::set_permissions(espflash.as_std_path(), fs::Permissions::from_mode(0o700))
            .expect("fake espflash mode");
        let environment = LocalFlashEnvironment {
            workspace_dir: workspace.clone(),
            espflash_bin: espflash.clone(),
            espflash_version: "espflash 4.5.0".to_owned(),
            espflash_sha256: sha256_bytes(b"fake espflash"),
        };
        let command = Phase35ProbeCommand {
            board: BoardId::Ultra205,
            port: "/dev/private-device".to_owned(),
            stage_root: Utf8PathBuf::from("scratch/probe"),
            timeout_seconds: 30,
        };

        // Act
        run_phase35_probe(&command, &environment).expect("Phase 35 probe");

        // Assert
        let stage_root = workspace.join("scratch/probe");
        let private_log = stage_root.join("probe.private.log");
        let metrics = stage_root.join("probe.metrics.json");
        let captured = fs::read_to_string(private_log.as_std_path()).expect("private log");
        assert!(captured.contains("password=[redacted]"));
        assert!(!captured.contains("probe-secret"));
        assert!(captured.contains("0x0123456789abcdef0123456789abcdef"));
        assert_eq!(
            fs::metadata(private_log.as_std_path())
                .expect("private metadata")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        assert_eq!(
            fs::metadata(metrics.as_std_path())
                .expect("metrics metadata")
                .permissions()
                .mode()
                & 0o777,
            0o600
        );
        let args = fs::read_to_string(bin_dir.join("args.log").as_std_path()).expect("args");
        assert_eq!(
            args,
            "checksum-md5\n--chip\nesp32s3\n--port\n/dev/private-device\n--non-interactive\n--before\nusb-reset\n--after\nhard-reset\n--skip-update-check\n0x0\n4096\n"
        );
        let second = run_phase35_probe(&command, &environment).expect_err("no clobber");
        assert!(format!("{second:#}").contains("destination_exists"));
    }

    #[test]
    fn parses_key_value_aliases_for_flash() {
        // Arrange
        let args = [
            "bitaxe-flash",
            "flash",
            "board=205",
            "dry-run=true",
            "redact-evidence=true",
            "port=/dev/cu.usbmodem101",
            "image=/tmp/bitaxe-ultra205.elf",
        ];

        // Act
        let cli = parse_cli(args).expect("cli");

        // Assert
        let CliCommand::Flash(command) = cli.command else {
            panic!("expected flash command");
        };
        assert_eq!(command.common.board, BoardId::Ultra205);
        assert_eq!(command.common.port.as_deref(), Some("/dev/cu.usbmodem101"));
        assert!(command.common.dry_run);
        assert!(command.common.redact_evidence);
        assert_eq!(
            command.image.as_deref(),
            Some(Utf8Path::new("/tmp/bitaxe-ultra205.elf"))
        );
    }

    #[test]
    fn flash_monitor_parses_capture_timeout_alias() {
        // Arrange
        let hyphenated_args = [
            "bitaxe-flash",
            "flash-monitor",
            "port=/dev/cu.usbmodem101",
            "capture-timeout-seconds=30",
        ];
        let underscored_args = [
            "bitaxe-flash",
            "flash-monitor",
            "port=/dev/cu.usbmodem101",
            "capture_timeout_seconds=30",
        ];

        // Act
        let hyphenated_cli = parse_cli(hyphenated_args).expect("hyphenated cli");
        let underscored_cli = parse_cli(underscored_args).expect("underscored cli");

        // Assert
        let CliCommand::FlashMonitor(hyphenated_command) = hyphenated_cli.command else {
            panic!("expected flash-monitor command");
        };
        let CliCommand::FlashMonitor(underscored_command) = underscored_cli.command else {
            panic!("expected flash-monitor command");
        };
        assert_eq!(hyphenated_command.capture_timeout_seconds, 30);
        assert_eq!(underscored_command.capture_timeout_seconds, 30);
    }

    #[test]
    fn flash_monitor_parses_redact_evidence_aliases() {
        // Arrange
        let hyphenated_args = [
            "bitaxe-flash",
            "flash-monitor",
            "port=/dev/cu.usbmodem101",
            "redact-evidence=true",
        ];
        let underscored_args = [
            "bitaxe-flash",
            "flash-monitor",
            "port=/dev/cu.usbmodem101",
            "redact_evidence=true",
        ];

        // Act
        let hyphenated_cli = parse_cli(hyphenated_args).expect("hyphenated cli");
        let underscored_cli = parse_cli(underscored_args).expect("underscored cli");

        // Assert
        let CliCommand::FlashMonitor(hyphenated_command) = hyphenated_cli.command else {
            panic!("expected flash-monitor command");
        };
        let CliCommand::FlashMonitor(underscored_command) = underscored_cli.command else {
            panic!("expected flash-monitor command");
        };
        assert!(hyphenated_command.common.redact_evidence);
        assert!(underscored_command.common.redact_evidence);
    }

    #[test]
    fn flash_monitor_parses_dual_evidence_mode_aliases() {
        // Arrange
        let hyphenated_args = [
            "bitaxe-flash",
            "flash-monitor",
            "evidence-dir=/tmp/evidence",
            "evidence-mode=dual",
        ];
        let underscored_args = [
            "bitaxe-flash",
            "flash-monitor",
            "evidence_dir=/tmp/evidence",
            "evidence_mode=dual",
        ];

        // Act
        let hyphenated_cli = parse_cli(hyphenated_args).expect("hyphenated cli");
        let underscored_cli = parse_cli(underscored_args).expect("underscored cli");

        // Assert
        let CliCommand::FlashMonitor(hyphenated_command) = hyphenated_cli.command else {
            panic!("expected flash-monitor command");
        };
        let CliCommand::FlashMonitor(underscored_command) = underscored_cli.command else {
            panic!("expected flash-monitor command");
        };
        assert_eq!(
            hyphenated_command.common.evidence_mode,
            Some(EvidenceMode::Dual)
        );
        assert_eq!(
            underscored_command.common.evidence_mode,
            Some(EvidenceMode::Dual)
        );
    }

    #[test]
    fn finalize_evidence_parses_software_only_inputs() {
        // Arrange
        let digest = "a".repeat(64);
        let args = [
            "bitaxe-flash".to_owned(),
            "finalize-evidence".to_owned(),
            "evidence_dir=scratch/private-evidence".to_owned(),
            format!("expected_private_sha256={digest}"),
        ];

        // Act
        let cli = parse_cli(args).expect("finalize cli");

        // Assert
        let CliCommand::FinalizeEvidence(command) = cli.command else {
            panic!("expected finalize-evidence command");
        };
        assert_eq!(
            command.evidence_dir,
            Utf8PathBuf::from("scratch/private-evidence")
        );
        assert_eq!(command.expected_private_sha256, digest);
    }

    #[test]
    fn flash_monitor_rejects_conflicting_evidence_modes() {
        // Arrange
        let args = [
            "bitaxe-flash",
            "flash-monitor",
            "--evidence-dir",
            "/tmp/evidence",
            "--evidence-mode",
            "dual",
            "--redact-evidence",
        ];

        // Act
        let result = parse_cli(args);

        // Assert
        let error = result.expect_err("conflicting modes");
        assert!(format!("{error:#}").contains("cannot be used with"));
    }

    #[test]
    fn non_flash_monitor_commands_reject_dual_mode() {
        // Arrange
        let flash_args = [
            "bitaxe-flash",
            "flash",
            "--evidence-mode",
            "dual",
            "--evidence-dir",
            "/tmp/evidence",
        ];
        let monitor_args = [
            "bitaxe-flash",
            "monitor",
            "--evidence-mode",
            "dual",
            "--evidence-dir",
            "/tmp/evidence",
        ];

        // Act
        let flash_result = parse_cli(flash_args);
        let monitor_result = parse_cli(monitor_args);

        // Assert
        assert!(format!("{:#}", flash_result.expect_err("flash dual")).contains("only"));
        assert!(format!("{:#}", monitor_result.expect_err("monitor dual")).contains("only"));
    }

    #[test]
    fn dual_console_value_never_exposes_operational_input() {
        // Arrange
        let operational = "/Users/operator/private.log --port /dev/cu.usbmodem101";

        // Act
        let dual_value = operational_console_value(operational, false);
        let legacy_value = operational_console_value(operational, true);

        // Assert
        assert_eq!(dual_value, PROTECTED_OPERATIONAL);
        assert_eq!(legacy_value, operational);
        assert!(!dual_value.contains("/Users"));
        assert!(!dual_value.contains("/dev/"));
    }

    #[test]
    fn parses_wifi_credentials_aliases_for_flash_and_flash_monitor() {
        // Arrange
        let flash_args = [
            "bitaxe-flash",
            "flash",
            "port=/dev/cu.usbmodem101",
            "wifi-credentials=/tmp/wifi.json",
        ];
        let flash_monitor_args = [
            "bitaxe-flash",
            "flash-monitor",
            "port=/dev/cu.usbmodem101",
            "wifi_credentials=/tmp/wifi.json",
        ];

        // Act
        let flash_cli = parse_cli(flash_args).expect("flash cli");
        let flash_monitor_cli = parse_cli(flash_monitor_args).expect("flash-monitor cli");

        // Assert
        let CliCommand::Flash(flash_command) = flash_cli.command else {
            panic!("expected flash command");
        };
        let CliCommand::FlashMonitor(flash_monitor_command) = flash_monitor_cli.command else {
            panic!("expected flash-monitor command");
        };
        assert_eq!(
            flash_command.wifi_credentials.as_deref(),
            Some(Utf8Path::new("/tmp/wifi.json"))
        );
        assert_eq!(
            flash_monitor_command.wifi_credentials.as_deref(),
            Some(Utf8Path::new("/tmp/wifi.json"))
        );
    }

    #[test]
    fn dry_run_flash_with_explicit_image_renders_vector_command() {
        // Arrange
        let command = FlashCommand {
            common: common_args(),
            image: Some(Utf8PathBuf::from("/tmp/bitaxe-ultra205.elf")),
            manifest: None,
            wifi_credentials: None,
        };
        let environment = FakeFlashEnvironment::default();

        // Act
        let outcome = run_flash(&command, &environment).expect("flash");

        // Assert
        assert_eq!(
            outcome.command,
            CommandSpec::new(
                "espflash",
                [
                    "flash",
                    "--chip",
                    "esp32s3",
                    "--port",
                    "/dev/cu.usbmodem101",
                    "/tmp/bitaxe-ultra205.elf",
                ],
            )
        );
        assert!(environment.executed_commands().is_empty());
    }

    #[test]
    fn flash_with_wifi_credentials_generates_and_executes_nvs_seed_after_flash() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let credentials_path = write_wifi_credentials(&dir, "LabNet", "super-secret");
        let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
        let command = FlashCommand {
            common: CommonArgs {
                dry_run: false,
                ..common_args()
            },
            image: None,
            manifest: Some(manifest),
            wifi_credentials: Some(credentials_path),
        };
        let environment = FakeFlashEnvironment::default();

        // Act
        let outcome = run_flash(&command, &environment).expect("flash");

        // Assert
        let nvs_seed = outcome.nvs_seed.as_ref().expect("nvs seed");
        let observed = environment.observed_flashes();
        let executed_flash_path = observed[0].path.as_str();
        assert_eq!(
            environment.generated_nvs_partitions(),
            vec![(
                nvs_seed
                    .image
                    .parent()
                    .expect("nvs seed parent")
                    .join("wifi-nvs.csv"),
                nvs_seed.image.clone(),
                NVS_PARTITION_SIZE.to_owned(),
            )]
        );
        assert_eq!(
            environment.executed_commands(),
            vec![
                CommandSpec::new(
                    "espflash",
                    [
                        "write-bin",
                        "--chip",
                        "esp32s3",
                        "--port",
                        "/dev/cu.usbmodem101",
                        "--non-interactive",
                        "--before",
                        "usb-reset",
                        "--after",
                        "hard-reset",
                        "--skip-update-check",
                        "0x0",
                        executed_flash_path,
                    ],
                ),
                CommandSpec::new(
                    "espflash",
                    [
                        "write-bin",
                        "--chip",
                        "esp32s3",
                        "--port",
                        "/dev/cu.usbmodem101",
                        "--non-interactive",
                        "--before",
                        "usb-reset",
                        "--after",
                        "hard-reset",
                        "--skip-update-check",
                        NVS_PARTITION_OFFSET,
                        nvs_seed.image.as_str(),
                    ],
                ),
            ]
        );
        assert_eq!(
            environment.phase35_stage_gates(),
            vec![
                ("after-factory".to_owned(), "/dev/cu.usbmodem101".to_owned()),
                ("after-nvs".to_owned(), "/dev/cu.usbmodem101".to_owned()),
            ]
        );
    }

    #[test]
    fn wifi_credentials_nvs_csv_uses_main_namespace_and_upstream_keys() {
        // Arrange
        let credentials = WifiCredentials {
            ssid: "Lab,Net".to_owned(),
            wifi_pass: "quoted\"secret".to_owned(),
        };

        // Act
        let csv = wifi_nvs_csv(&credentials);

        // Assert
        assert!(csv.contains("main,namespace,,"));
        assert!(csv.contains("wifissid,data,string,\"Lab,Net\""));
        assert!(csv.contains("wifipass,data,string,\"quoted\"\"secret\""));
    }

    #[test]
    fn wifi_credentials_reject_invalid_lengths_without_secret_value() {
        // Arrange
        let file = WifiCredentialsFile {
            ssid: String::new(),
            wifi_pass: "p".repeat(64),
        };

        // Act
        let result = validate_wifi_credentials(file);

        // Assert
        let error = format!("{result:#?}");
        assert!(error.contains("ssid length 0 is outside 1..=32"));
        assert!(error.contains("wifiPass length 64 is outside 0..=63"));
        assert!(!error.contains(&"p".repeat(64)));
    }

    #[test]
    fn dry_run_flash_resolves_admitted_factory_artifact() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest(&dir, DEFAULT_ELF_NAME);
        let command = FlashCommand {
            common: common_args(),
            image: None,
            manifest: Some(manifest.clone()),
            wifi_credentials: None,
        };
        let environment = FakeFlashEnvironment::default();

        // Act
        let outcome = run_flash(&command, &environment).expect("flash");

        // Assert
        assert_eq!(outcome.manifest.as_ref(), Some(&manifest));
        assert_eq!(
            outcome.flash_image,
            manifest.parent().expect("parent").join(FACTORY_IMAGE_NAME)
        );
        assert_eq!(
            outcome.command.args,
            vec![
                "write-bin",
                "--chip",
                "esp32s3",
                "--port",
                "/dev/cu.usbmodem101",
                "--non-interactive",
                "--before",
                "usb-reset",
                "--after",
                "hard-reset",
                "--skip-update-check",
                "0x0",
                outcome.flash_image.as_str(),
            ]
        );
    }

    #[test]
    fn relative_image_argument_resolves_under_workspace_dir() {
        // Arrange
        let workspace = tempdir().expect("workspace");
        let workspace_dir = dir_path(&workspace);
        let command = FlashCommand {
            common: common_args(),
            image: Some(Utf8PathBuf::from("docs/evidence/bitaxe-ultra205.elf")),
            manifest: None,
            wifi_credentials: None,
        };
        let environment = FakeFlashEnvironment::default().with_workspace_dir(workspace_dir.clone());

        // Act
        let outcome = run_flash(&command, &environment).expect("flash");

        // Assert
        assert_eq!(
            outcome.flash_image,
            workspace_dir.join("docs/evidence/bitaxe-ultra205.elf")
        );
    }

    #[test]
    fn relative_manifest_argument_resolves_under_workspace_dir() {
        // Arrange
        let workspace = tempdir().expect("workspace");
        let workspace_dir = dir_path(&workspace);
        let manifest = write_manifest_at(
            &workspace_dir,
            "docs/evidence/package/bitaxe-ultra205-package.json",
            DEFAULT_ELF_NAME,
        );
        let command = FlashCommand {
            common: common_args(),
            image: None,
            manifest: Some(Utf8PathBuf::from(
                "docs/evidence/package/bitaxe-ultra205-package.json",
            )),
            wifi_credentials: None,
        };
        let environment = FakeFlashEnvironment::default().with_workspace_dir(workspace_dir.clone());

        // Act
        let outcome = run_flash(&command, &environment).expect("flash");

        // Assert
        assert_eq!(outcome.manifest.as_ref(), Some(&manifest));
        assert_eq!(
            outcome.flash_image,
            workspace_dir
                .join("docs/evidence/package")
                .join(FACTORY_IMAGE_NAME)
        );
    }

    #[test]
    fn rejects_manifest_default_factory_bin() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest(&dir, FACTORY_IMAGE_NAME);
        let command = FlashCommand {
            common: common_args(),
            image: None,
            manifest: Some(manifest),
            wifi_credentials: None,
        };
        let environment = FakeFlashEnvironment::default();

        // Act
        let result = run_flash(&command, &environment);

        // Assert
        assert!(format!("{result:#?}").contains(DEFAULT_ELF_NAME));
    }

    #[test]
    fn manifest_v3_uses_factory_artifact_for_full_flash() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
        let command = FlashCommand {
            common: common_args(),
            image: None,
            manifest: Some(manifest.clone()),
            wifi_credentials: None,
        };
        let environment = FakeFlashEnvironment::default();

        // Act
        let outcome = run_flash(&command, &environment).expect("flash");

        // Assert
        assert_eq!(outcome.manifest.as_ref(), Some(&manifest));
        assert_eq!(
            outcome.flash_image,
            manifest.parent().expect("parent").join(FACTORY_IMAGE_NAME)
        );
        assert_eq!(
            outcome.command.args,
            vec![
                "write-bin",
                "--chip",
                "esp32s3",
                "--port",
                "/dev/cu.usbmodem101",
                "--non-interactive",
                "--before",
                "usb-reset",
                "--after",
                "hard-reset",
                "--skip-update-check",
                "0x0",
                outcome.flash_image.as_str(),
            ]
        );
    }

    #[test]
    fn identity_admission_accepts_clean_dev_and_release_builds() {
        // Arrange
        let cases = [
            BuildProvenance::new(
                "0.1.0",
                SOURCE_COMMIT,
                false,
                None::<&str>,
                REFERENCE_COMMIT,
            )
            .expect("dev provenance"),
            BuildProvenance::new(
                "1.2.0",
                SOURCE_COMMIT,
                false,
                Some("v1.2"),
                REFERENCE_COMMIT,
            )
            .expect("release provenance"),
        ];

        for provenance in cases {
            let dir = tempdir().expect("tempdir");
            let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
            rewrite_manifest_provenance(&manifest, &provenance);
            let command = FlashCommand {
                common: common_args(),
                image: None,
                manifest: Some(manifest),
                wifi_credentials: None,
            };
            let environment =
                FakeFlashEnvironment::default().with_current_provenance(provenance.clone());

            // Act
            let outcome = run_flash(&command, &environment);

            // Assert
            assert!(outcome.is_ok(), "{outcome:#?}");
        }
    }

    #[test]
    fn identity_admission_rejects_dirty_package_before_port_or_credentials() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
        let dirty_provenance =
            BuildProvenance::new("0.1.0", SOURCE_COMMIT, true, None::<&str>, REFERENCE_COMMIT)
                .expect("dirty provenance");
        rewrite_manifest_provenance(&manifest, &dirty_provenance);
        let command = FlashCommand {
            common: CommonArgs {
                port: None,
                dry_run: false,
                ..common_args()
            },
            image: None,
            manifest: Some(manifest),
            wifi_credentials: Some(Utf8PathBuf::from("/missing/credentials.json")),
        };
        let environment = FakeFlashEnvironment::with_ports(
            "/dev/cu.usbmodem101 USB JTAG\n/dev/cu.usbmodem102 USB JTAG\n",
        );

        // Act
        let result = run_flash(&command, &environment);

        // Assert
        let error = format!("{result:#?}");
        assert!(error.contains("identity_admission=blocked reason=package_source_dirty"));
        assert!(!error.contains("Ambiguous serial ports"));
        assert!(!error.contains("credentials"));
    }

    #[test]
    fn identity_admission_rejects_dirty_current_workspace_before_port() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
        let dirty_provenance =
            BuildProvenance::new("0.1.0", SOURCE_COMMIT, true, None::<&str>, REFERENCE_COMMIT)
                .expect("dirty provenance");
        let command = FlashCommand {
            common: CommonArgs {
                port: None,
                dry_run: false,
                ..common_args()
            },
            image: None,
            manifest: Some(manifest),
            wifi_credentials: None,
        };
        let environment = FakeFlashEnvironment::with_ports(
            "/dev/cu.usbmodem101 USB JTAG\n/dev/cu.usbmodem102 USB JTAG\n",
        )
        .with_current_provenance(dirty_provenance);

        // Act
        let result = run_flash(&command, &environment);

        // Assert
        let error = format!("{result:#?}");
        assert!(error.contains("identity_admission=blocked reason=current_workspace_dirty"));
        assert!(!error.contains("Ambiguous serial ports"));
    }

    #[test]
    fn identity_admission_rejects_unmanifested_explicit_image_before_port() {
        // Arrange
        let command = FlashCommand {
            common: CommonArgs {
                port: None,
                dry_run: false,
                ..common_args()
            },
            image: Some(Utf8PathBuf::from("/tmp/firmware.bin")),
            manifest: None,
            wifi_credentials: None,
        };
        let environment = FakeFlashEnvironment::with_ports(
            "/dev/cu.usbmodem101 USB JTAG\n/dev/cu.usbmodem102 USB JTAG\n",
        );

        // Act
        let result = run_flash(&command, &environment);

        // Assert
        let error = format!("{result:#?}");
        assert!(
            error.contains("identity_admission=blocked reason=explicit_image_requires_v3_manifest")
        );
        assert!(!error.contains("Ambiguous serial ports"));
    }

    #[test]
    fn identity_admission_rejects_package_digest_mismatch() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
        let ota = manifest
            .parent()
            .expect("manifest parent")
            .join("esp-miner.bin");
        std::fs::write(ota.as_std_path(), b"tampered ota").expect("tamper ota");
        let command = FlashCommand {
            common: common_args(),
            image: None,
            manifest: Some(manifest),
            wifi_credentials: None,
        };
        let environment = FakeFlashEnvironment::default();

        // Act
        let result = run_flash(&command, &environment);

        // Assert
        assert!(format!("{result:#?}")
            .contains("identity_admission=blocked reason=package_artifact_digest_mismatch"));
    }

    #[test]
    fn identity_admission_rejects_duplicate_ota_before_port_or_credentials() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
        duplicate_manifest_artifact(&manifest, "firmware_ota_image");
        let command = FlashCommand {
            common: CommonArgs {
                port: None,
                dry_run: false,
                ..common_args()
            },
            image: None,
            manifest: Some(manifest),
            wifi_credentials: Some(Utf8PathBuf::from("/missing/credentials.json")),
        };
        let environment = FakeFlashEnvironment::with_ports(
            "/dev/cu.usbmodem101 USB JTAG\n/dev/cu.usbmodem102 USB JTAG\n",
        );

        // Act
        let result = run_flash(&command, &environment);

        // Assert
        let error = format!("{result:#?}");
        assert!(error
            .contains("identity_admission=blocked reason=duplicate_firmware_ota_image_artifact"));
        assert!(!error.contains("Ambiguous serial ports"));
        assert!(!error.contains("credentials"));
    }

    #[test]
    fn identity_admission_rejects_duplicate_factory_before_port_or_credentials() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
        duplicate_manifest_artifact(&manifest, "factory_merged_image");
        let command = FlashCommand {
            common: CommonArgs {
                port: None,
                dry_run: false,
                ..common_args()
            },
            image: None,
            manifest: Some(manifest),
            wifi_credentials: Some(Utf8PathBuf::from("/missing/credentials.json")),
        };
        let environment = FakeFlashEnvironment::with_ports(
            "/dev/cu.usbmodem101 USB JTAG\n/dev/cu.usbmodem102 USB JTAG\n",
        );

        // Act
        let result = run_flash(&command, &environment);

        // Assert
        let error = format!("{result:#?}");
        assert!(error
            .contains("identity_admission=blocked reason=duplicate_factory_merged_image_artifact"));
        assert!(!error.contains("Ambiguous serial ports"));
        assert!(!error.contains("credentials"));
    }

    #[test]
    fn identity_admission_rejects_digest_rewritten_factory_app_tamper_before_effects() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
        let factory_path = manifest
            .parent()
            .expect("manifest parent")
            .join(FACTORY_IMAGE_NAME);
        let mut factory = std::fs::read(factory_path.as_std_path()).expect("factory image");
        let tamper_offset = 0x10000 + 40;
        factory[tamper_offset] ^= 0x01;
        std::fs::write(factory_path.as_std_path(), &factory).expect("tampered factory image");
        rewrite_manifest_artifact_digest(&manifest, "factory_merged_image", &factory);
        let command = FlashCommand {
            common: CommonArgs {
                port: None,
                dry_run: false,
                ..common_args()
            },
            image: None,
            manifest: Some(manifest),
            wifi_credentials: Some(Utf8PathBuf::from("/missing/credentials.json")),
        };
        let environment = FakeFlashEnvironment::with_ports(
            "/dev/cu.usbmodem101 USB JTAG\n/dev/cu.usbmodem102 USB JTAG\n",
        );

        // Act
        let result = run_flash(&command, &environment);

        // Assert
        let error = result.expect_err("factory application tamper").to_string();
        assert!(error.contains("identity_admission=blocked reason=ota_segment_checksum_mismatch"));
        assert!(!error.contains("Ambiguous serial ports"));
        assert!(!error.contains("credentials"));
        assert!(environment.executed_commands().is_empty());
    }

    #[test]
    fn executable_admission_rejects_zero_load_address_in_parsed_dry_run_before_effects() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
        let mut ota = esp_application_fixture(SOURCE_COMMIT, BUILD_LABEL);
        ota[24..28].copy_from_slice(&0_u32.to_le_bytes());
        reseal_esp_application(&mut ota);
        rewrite_manifest_application(&manifest, &ota);
        let cli = parse_cli([
            "bitaxe-flash".to_owned(),
            "flash".to_owned(),
            "dry-run=true".to_owned(),
            "port=/dev/null".to_owned(),
            format!("manifest={manifest}"),
        ])
        .expect("parsed dry-run command");
        let CliCommand::Flash(command) = cli.command else {
            panic!("expected flash command");
        };
        let environment = FakeFlashEnvironment::default();

        // Act
        let result = run_flash(&command, &environment);

        // Assert
        let error = result.expect_err("zero load address").to_string();
        assert!(error.contains("ota_segment_load_address_unsupported"));
        assert!(environment.executed_commands().is_empty());
        assert!(environment.created_snapshot_paths().is_empty());
    }

    #[test]
    fn executable_admission_rejects_mapped_mismatch_in_parsed_non_dry_run_before_effects() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
        let mut ota = esp_application_fixture(SOURCE_COMMIT, BUILD_LABEL);
        ota[24..28].copy_from_slice(&0x3c00_0024_u32.to_le_bytes());
        reseal_esp_application(&mut ota);
        rewrite_manifest_application(&manifest, &ota);
        let cli = parse_cli([
            "bitaxe-flash".to_owned(),
            "flash".to_owned(),
            format!("manifest={manifest}"),
            "wifi-credentials=/missing/credentials.json".to_owned(),
        ])
        .expect("parsed non-dry command");
        let CliCommand::Flash(command) = cli.command else {
            panic!("expected flash command");
        };
        let environment = FakeFlashEnvironment::with_ports(
            "/dev/cu.usbmodem101 USB JTAG\n/dev/cu.usbmodem102 USB JTAG\n",
        );

        // Act
        let result = run_flash(&command, &environment);

        // Assert
        let error = result.expect_err("mapped mismatch").to_string();
        assert!(error.contains("ota_mapped_segment_misaligned"), "{error}");
        assert!(!error.contains("Ambiguous serial ports"));
        assert!(!error.contains("credentials"));
        assert!(environment.executed_commands().is_empty());
        assert!(environment.created_snapshot_paths().is_empty());
    }

    #[test]
    fn identity_admission_rejects_all_layout_classes_in_parsed_dry_run_before_effects() {
        for (fixture_kind, reason) in [
            (
                LayoutFixtureKind::DescriptorNotDrom,
                "app_descriptor_segment_not_drom",
            ),
            (
                LayoutFixtureKind::DestinationOverlap,
                "ota_segment_destination_overlap",
            ),
            (LayoutFixtureKind::AliasOverlap, "ota_segment_alias_overlap"),
        ] {
            assert_parsed_layout_rejected_before_effects(fixture_kind, reason, true);
        }
    }

    #[test]
    fn identity_admission_rejects_all_layout_classes_in_parsed_non_dry_run_before_effects() {
        for (fixture_kind, reason) in [
            (
                LayoutFixtureKind::DescriptorNotDrom,
                "app_descriptor_segment_not_drom",
            ),
            (
                LayoutFixtureKind::DestinationOverlap,
                "ota_segment_destination_overlap",
            ),
            (LayoutFixtureKind::AliasOverlap, "ota_segment_alias_overlap"),
        ] {
            assert_parsed_layout_rejected_before_effects(fixture_kind, reason, false);
        }
    }

    #[test]
    fn firmware_elf_app_sha_rejects_changed_elf_in_parsed_dry_run_before_later_reads() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
        rewrite_manifest_elf_artifact_only(&manifest, b"changed firmware elf");
        let cli = parse_cli([
            "bitaxe-flash".to_owned(),
            "flash".to_owned(),
            "dry-run=true".to_owned(),
            "port=/dev/null".to_owned(),
            format!("manifest={manifest}"),
        ])
        .expect("parsed dry-run command");
        let CliCommand::Flash(command) = cli.command else {
            panic!("expected flash command");
        };
        let ota_path = manifest
            .parent()
            .expect("manifest parent")
            .join("esp-miner.bin");
        std::fs::remove_file(ota_path.as_std_path()).expect("remove later OTA artifact");
        let environment = FakeFlashEnvironment::default();

        // Act
        let result = run_flash(&command, &environment);

        // Assert
        let error = result.expect_err("ELF relationship mismatch").to_string();
        assert!(error.contains("firmware_elf_app_sha_mismatch"));
        assert!(!error.contains("failed to read fake artifact"));
        assert!(environment.executed_commands().is_empty());
        assert!(environment.created_snapshot_paths().is_empty());
    }

    #[test]
    fn firmware_elf_app_sha_rejects_changed_elf_in_parsed_non_dry_run_before_effects() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
        rewrite_manifest_elf_artifact_only(&manifest, b"changed firmware elf");
        let cli = parse_cli([
            "bitaxe-flash".to_owned(),
            "flash".to_owned(),
            format!("manifest={manifest}"),
            "wifi-credentials=/missing/credentials.json".to_owned(),
        ])
        .expect("parsed non-dry command");
        let CliCommand::Flash(command) = cli.command else {
            panic!("expected flash command");
        };
        let environment = FakeFlashEnvironment::with_ports(
            "/dev/cu.usbmodem101 USB JTAG\n/dev/cu.usbmodem102 USB JTAG\n",
        );

        // Act
        let result = run_flash(&command, &environment);

        // Assert
        let error = result.expect_err("ELF relationship mismatch").to_string();
        assert!(error.contains("firmware_elf_app_sha_mismatch"));
        assert!(!error.contains("Ambiguous serial ports"));
        assert!(!error.contains("credentials"));
        assert!(environment.executed_commands().is_empty());
        assert!(environment.created_snapshot_paths().is_empty());
    }

    #[test]
    fn identity_admission_rejects_explicit_manifest_elf_before_effects() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
        let image = manifest
            .parent()
            .expect("manifest parent")
            .join(DEFAULT_ELF_NAME);

        // Act
        let error = run_explicit_image_admission(&manifest, image)
            .expect_err("manifest ELF must not enter full-flash execution");

        // Assert
        assert!(format!("{error:#}")
            .contains("identity_admission=blocked reason=explicit_image_not_admitted_factory"));
    }

    #[test]
    fn identity_admission_rejects_explicit_extra_artifact_before_effects() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
        let image = add_manifest_artifact(&manifest, "extra", "extra.bin", b"extra image");

        // Act
        let error = run_explicit_image_admission(&manifest, image)
            .expect_err("extra artifact must not enter full-flash execution");

        // Assert
        assert!(format!("{error:#}")
            .contains("identity_admission=blocked reason=explicit_image_not_admitted_factory"));
    }

    #[test]
    fn identity_admission_rejects_explicit_factory_path_alias_before_effects() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
        let manifest_dir = manifest.parent().expect("manifest parent");
        let factory = manifest_dir.join(FACTORY_IMAGE_NAME);
        let factory_bytes = std::fs::read(factory.as_std_path()).expect("factory image");
        let alias = add_manifest_artifact(
            &manifest,
            "factory_alias",
            "factory-alias.bin",
            &factory_bytes,
        );

        // Act
        let error = run_explicit_image_admission(&manifest, alias)
            .expect_err("factory path alias must not enter full-flash execution");

        // Assert
        assert!(format!("{error:#}")
            .contains("identity_admission=blocked reason=explicit_image_not_admitted_factory"));
    }

    #[test]
    fn identity_admission_rejects_explicit_factory_named_extra_before_effects() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
        let image = add_manifest_artifact(
            &manifest,
            "factory_named_extra",
            "nested/bitaxe-ultra205-factory.bin",
            b"factory-named extra",
        );

        // Act
        let error = run_explicit_image_admission(&manifest, image)
            .expect_err("factory-like basename must not enter full-flash execution");

        // Assert
        assert!(format!("{error:#}")
            .contains("identity_admission=blocked reason=explicit_image_not_admitted_factory"));
    }

    #[test]
    fn admitted_execution_uses_original_bytes_after_package_replacement() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
        let factory_path = manifest
            .parent()
            .expect("manifest parent")
            .join(FACTORY_IMAGE_NAME);
        let admitted_bytes = std::fs::read(factory_path.as_std_path()).expect("factory image");
        let command = FlashCommand {
            common: CommonArgs {
                dry_run: false,
                ..common_args()
            },
            image: None,
            manifest: Some(manifest),
            wifi_credentials: None,
        };
        let environment = FakeFlashEnvironment::default()
            .with_source_replacement(factory_path.clone(), b"replaced package bytes".to_vec());

        // Act
        run_flash(&command, &environment).expect("admitted flash");

        // Assert
        let observed = environment.observed_flashes();
        assert_eq!(observed.len(), 1);
        assert_ne!(observed[0].path, factory_path);
        assert_eq!(observed[0].bytes, admitted_bytes);
        #[cfg(unix)]
        assert_eq!(observed[0].unix_mode, Some(0o600));
        assert!(!observed[0].path.exists());
    }

    #[test]
    fn admitted_execution_child_failure_cleans_private_snapshot() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
        let command = FlashCommand {
            common: CommonArgs {
                dry_run: false,
                ..common_args()
            },
            image: None,
            manifest: Some(manifest),
            wifi_credentials: None,
        };
        let environment = FakeFlashEnvironment::default().with_execute_failure();

        // Act
        let error = run_flash(&command, &environment).expect_err("child failure");

        // Assert
        let error = format!("{error:#}");
        assert!(error.contains("flash_execution=failed reason=admitted_image_child_failed"));
        assert!(!error.contains("sentinel child failure"));
        let observed = environment.observed_flashes();
        assert_eq!(observed.len(), 1);
        assert!(!error.contains(observed[0].path.as_str()));
        assert!(!observed[0].path.exists());
    }

    #[test]
    fn admitted_execution_snapshot_write_failure_precedes_later_effects() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
        let command = FlashCommand {
            common: CommonArgs {
                port: None,
                dry_run: false,
                ..common_args()
            },
            image: None,
            manifest: Some(manifest),
            wifi_credentials: Some(Utf8PathBuf::from("/missing/credentials.json")),
        };
        let environment = FakeFlashEnvironment::with_ports(
            "/dev/cu.usbmodem101 USB JTAG\n/dev/cu.usbmodem102 USB JTAG\n",
        )
        .with_snapshot_write_failure();

        // Act
        let error = run_flash(&command, &environment).expect_err("snapshot write failure");

        // Assert
        let error = format!("{error:#}");
        assert!(error.contains("execution_snapshot_write_failed"));
        assert!(!error.contains("Ambiguous serial ports"));
        assert!(!error.contains("credentials"));
        assert!(environment.executed_commands().is_empty());
    }

    #[test]
    fn admitted_execution_later_preparation_failure_cleans_private_snapshot() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
        let command = FlashCommand {
            common: CommonArgs {
                dry_run: false,
                ..common_args()
            },
            image: None,
            manifest: Some(manifest),
            wifi_credentials: Some(Utf8PathBuf::from("/missing/credentials.json")),
        };
        let environment = FakeFlashEnvironment::default();

        // Act
        let error = run_flash(&command, &environment).expect_err("preparation failure");

        // Assert
        assert!(format!("{error:#}").contains("Wi-Fi credential file"));
        let paths = environment.created_snapshot_paths();
        assert_eq!(paths.len(), 1);
        assert!(!paths[0].exists());
        assert!(environment.executed_commands().is_empty());
    }

    #[test]
    fn admitted_execution_command_construction_failure_cleans_private_snapshot() {
        // Arrange
        let snapshot =
            AdmittedExecutionSnapshot::materialize(b"admitted bytes").expect("private snapshot");
        let snapshot_path = snapshot.path().to_owned();
        let developer_image = AdmittedFlashImage::DeveloperDryRun {
            display_path: Utf8PathBuf::from("developer.elf"),
        };

        // Act
        let error = flash_command_for_admitted_image(
            "/dev/cu.usbmodem101",
            &developer_image,
            snapshot.path(),
            false,
        )
        .expect_err("non-dry-run developer command");
        drop(snapshot);

        // Assert
        assert!(format!("{error:#}").contains("developer_image_requires_dry_run"));
        assert!(!snapshot_path.exists());
    }

    #[test]
    fn manifest_v3_rejects_wrong_factory_artifact_name() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v3_with_factory_artifact(&dir, DEFAULT_ELF_NAME, "wrong.bin");
        let command = FlashCommand {
            common: common_args(),
            image: None,
            manifest: Some(manifest),
            wifi_credentials: None,
        };
        let environment = FakeFlashEnvironment::default();

        // Act
        let result = run_flash(&command, &environment);

        // Assert
        let error = format!("{result:#?}");
        assert!(error.contains(FACTORY_IMAGE_NAME));
        assert!(error.contains("wrong.bin"));
    }

    #[test]
    fn zero_ports_error_includes_actionable_example() {
        // Arrange
        let environment = FakeFlashEnvironment::with_ports("");

        // Act
        let result = resolve_port(None, &environment);

        // Assert
        let error = format!("{result:#?}");
        assert!(error.contains("No serial ports found"));
        assert!(error.contains("--port /dev/"));
    }

    #[test]
    fn ambiguous_ports_error_lists_each_candidate() {
        // Arrange
        let environment = FakeFlashEnvironment::with_ports(
            "/dev/cu.usbmodem101 USB JTAG\n/dev/cu.usbserial-110 USB serial\n",
        );

        // Act
        let result = resolve_port(None, &environment);

        // Assert
        let error = format!("{result:#?}");
        assert!(error.contains("Ambiguous serial ports"));
        assert!(error.contains("--port /dev/cu.usbmodem101"));
        assert!(error.contains("--port /dev/cu.usbserial-110"));
    }

    #[test]
    fn bare_com_is_not_a_likely_port() {
        // Arrange
        let port = "COM";

        // Act
        let likely = is_likely_port(port);

        // Assert
        assert!(!likely);
    }

    #[test]
    fn numbered_com_is_a_likely_port() {
        // Arrange
        let port = "COM3";

        // Act
        let likely = is_likely_port(port);

        // Assert
        assert!(likely);
    }

    #[test]
    fn evidence_monitor_command_uses_noninteractive_esp32s3_flags() {
        // Arrange
        let common = common_args();
        let environment = FakeFlashEnvironment::default();

        // Act
        let command = prepare_evidence_monitor_command(&common, &environment).expect("command");

        // Assert
        assert_eq!(command.program, "espflash");
        assert_eq!(
            command.args,
            vec![
                "monitor",
                "--chip",
                "esp32s3",
                "--port",
                "/dev/cu.usbmodem101",
                "--non-interactive",
            ]
        );
    }

    #[test]
    fn interactive_monitor_command_remains_interactive() {
        // Arrange
        let common = common_args();
        let environment = FakeFlashEnvironment::default();

        // Act
        let command = prepare_monitor_command(&common, &environment).expect("command");

        // Assert
        assert_eq!(
            command.args,
            vec!["monitor", "--port", "/dev/cu.usbmodem101"]
        );
        assert!(!command.args.iter().any(|arg| arg == "--non-interactive"));
    }

    #[test]
    fn trusted_marker_classifier_requires_serial_scope_markers() {
        // Arrange
        let trusted_log = trusted_monitor_log();
        let untrusted_log = trusted_log.replace("reference_commit=", "reference_sha=");

        // Act
        let trusted = monitor_log_has_trusted_boot_markers(&trusted_log);
        let untrusted = monitor_log_has_trusted_boot_markers(&untrusted_log);

        // Assert
        assert!(trusted);
        assert!(!untrusted);
    }

    #[test]
    fn trusted_marker_classifier_requires_safe_noop_state() {
        // Arrange
        let trusted_log = trusted_monitor_log();
        let unsafe_log = trusted_log.replace("mining=disabled", "mining=enabled");
        let prefixed_safe_log = trusted_log.replace("safe_state:", "unsafe_state:");

        // Act
        let trusted = monitor_log_has_trusted_boot_markers(&trusted_log);
        let unsafe_markers = monitor_log_has_trusted_boot_markers(&unsafe_log);
        let prefixed_safe = monitor_log_has_trusted_boot_markers(&prefixed_safe_log);

        // Assert
        assert!(trusted);
        assert!(!unsafe_markers);
        assert!(!prefixed_safe);
    }

    #[test]
    fn trusted_marker_classifier_requires_reset_and_esp_idf_provenance() {
        // Arrange
        let trusted_log = trusted_monitor_log();
        let without_reset_reason = trusted_log.replace("reset_reason=11\n", "");
        let without_esp_idf = trusted_log.replace("esp_idf_version=v5.5.4", "");

        // Act
        let trusted = monitor_log_has_trusted_boot_markers(&trusted_log);
        let missing_reset = monitor_log_has_trusted_boot_markers(&without_reset_reason);
        let missing_esp_idf = monitor_log_has_trusted_boot_markers(&without_esp_idf);

        // Assert
        assert!(trusted);
        assert!(!missing_reset);
        assert!(!missing_esp_idf);
    }

    #[test]
    fn trusted_marker_classifier_requires_exact_spiffs_and_route_tokens() {
        // Arrange
        let trusted_log = trusted_monitor_log();
        let prefixed_spiffs =
            trusted_log.replace("spiffs_mount=available", "not_spiffs_mount=available");
        let prefixed_route = trusted_log.replace(
            "axeos_api_route_shell=started",
            "not_axeos_api_route_shell=started",
        );

        // Act
        let trusted = monitor_log_has_trusted_boot_markers(&trusted_log);
        let bad_spiffs = monitor_log_has_trusted_boot_markers(&prefixed_spiffs);
        let bad_route = monitor_log_has_trusted_boot_markers(&prefixed_route);

        // Assert
        assert!(trusted);
        assert!(!bad_spiffs);
        assert!(!bad_route);
    }

    #[test]
    fn flash_monitor_evidence_points_to_created_log() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let evidence_dir = dir_path(&dir).join("evidence");
        let command = flash_monitor_fixture(&dir, evidence_dir.clone());
        let environment = FakeFlashEnvironment::default();

        // Act
        run_flash_monitor(&command, &environment).expect("flash-monitor");

        // Assert
        let log_path = evidence_dir.join("flash-monitor.log");
        let evidence_path = evidence_dir.join("flash-command-evidence.json");
        assert!(log_path.is_file());
        assert!(evidence_path.is_file());
        let evidence = std::fs::read_to_string(evidence_path.as_std_path()).expect("evidence");
        assert!(evidence.contains(r#""command_kind": "flash-monitor""#));
        assert!(evidence.contains(log_path.as_str()));
    }

    #[test]
    fn relative_evidence_dir_writes_under_workspace_dir() {
        // Arrange
        let workspace = tempdir().expect("workspace");
        let workspace_dir = dir_path(&workspace);
        let evidence_dir = Utf8PathBuf::from("docs/parity/evidence/phase-09-test");
        let command = flash_monitor_fixture(&workspace, evidence_dir.clone());
        let environment = FakeFlashEnvironment::default().with_workspace_dir(workspace_dir.clone());

        // Act
        run_flash_monitor(&command, &environment).expect("flash-monitor");

        // Assert
        let log_path = workspace_dir
            .join(evidence_dir.as_str())
            .join("flash-monitor.log");
        let evidence_path = workspace_dir
            .join(evidence_dir.as_str())
            .join("flash-command-evidence.json");
        assert!(log_path.is_file());
        assert!(evidence_path.is_file());
        let evidence = std::fs::read_to_string(evidence_path.as_std_path()).expect("evidence");
        assert!(evidence.contains(log_path.as_str()));
    }

    #[test]
    fn flash_monitor_evidence_uses_noninteractive_capture_command() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let evidence_dir = dir_path(&dir).join("evidence");
        let command = flash_monitor_fixture(&dir, evidence_dir);
        let environment = FakeFlashEnvironment::default();

        // Act
        run_flash_monitor(&command, &environment).expect("flash-monitor");

        // Assert
        assert_eq!(
            environment.captured_commands(),
            vec![CommandSpec::new(
                "espflash",
                [
                    "monitor",
                    "--chip",
                    "esp32s3",
                    "--port",
                    "/dev/cu.usbmodem101",
                    "--non-interactive",
                ],
            )]
        );
    }

    #[test]
    fn flash_monitor_evidence_json_records_capture_contract() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let evidence_dir = dir_path(&dir).join("evidence");
        let command = flash_monitor_fixture(&dir, evidence_dir.clone());
        let environment = FakeFlashEnvironment::default();

        // Act
        run_flash_monitor(&command, &environment).expect("flash-monitor");

        // Assert
        let evidence_path = evidence_dir.join("flash-command-evidence.json");
        let evidence = std::fs::read_to_string(evidence_path.as_std_path()).expect("evidence");
        let json: serde_json::Value = serde_json::from_str(&evidence).expect("json");
        for field in [
            "flash_command",
            "monitor_command",
            "monitor_log_path",
            "capture_mode",
            "capture_status",
            "capture_timeout_seconds",
            "trusted_output",
            "observed_firmware_commit",
            "observed_reference_commit",
            "conclusion",
        ] {
            assert!(json.get(field).is_some(), "missing {field}");
        }
        assert_eq!(json["capture_mode"], "noninteractive");
        assert_eq!(json["capture_status"], "completed");
        assert_eq!(json["capture_timeout_seconds"], 25);
        assert_eq!(json["trusted_output"], true);
        assert_eq!(json["observed_firmware_commit"], "0123456789ab");
        assert_eq!(json["observed_reference_commit"], "abcdef012345");
    }

    #[test]
    fn flash_evidence_records_nvs_seed_without_credential_path_or_values() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let evidence_dir = dir_path(&dir).join("evidence");
        let credentials_path = write_wifi_credentials(&dir, "LabNet", "super-secret");
        let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
        let command = FlashCommand {
            common: CommonArgs {
                evidence_dir: Some(evidence_dir.clone()),
                dry_run: false,
                ..common_args()
            },
            image: None,
            manifest: Some(manifest),
            wifi_credentials: Some(credentials_path.clone()),
        };
        let environment = FakeFlashEnvironment::default();

        // Act
        run_flash(&command, &environment).expect("flash");

        // Assert
        let evidence_path = evidence_dir.join("flash-command-evidence.json");
        let evidence = std::fs::read_to_string(evidence_path.as_std_path()).expect("evidence");
        let json: serde_json::Value = serde_json::from_str(&evidence).expect("json");
        assert_eq!(json["nvs_seed_status"], "provided");
        assert_eq!(json["nvs_seed_partition_offset"], NVS_PARTITION_OFFSET);
        assert_eq!(json["nvs_seed_partition_size"], NVS_PARTITION_SIZE);
        assert_eq!(json["redaction_mode"], "developer-raw");
        assert_eq!(json["commit_ready"], false);
        assert_eq!(json["wifi_credentials_source"], "provided-redacted");
        assert!(!evidence.contains(credentials_path.as_str()));
        assert!(!evidence.contains("LabNet"));
        assert!(!evidence.contains("super-secret"));
    }

    #[test]
    fn flash_monitor_developer_raw_preserves_network_identifiers_and_redacts_secrets() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let evidence_dir = dir_path(&dir).join("evidence");
        let command = flash_monitor_fixture(&dir, evidence_dir.clone());
        let sensitive_log = format!(
            "{}\nI (3863) wifi:connected with LabNet, aid = 1, channel 11, BW20, bssid = aa:bb:cc:dd:ee:ff\nwifi_status=connected ssid=lab-net password=super-secret token=api-secret ipv4=192.168.1.24 mac=aa:bb:cc:dd:ee:ff device_url=http://192.168.1.24\n",
            trusted_monitor_log()
        );
        let environment = FakeFlashEnvironment::default().with_log_contents(&sensitive_log);

        // Act
        run_flash_monitor(&command, &environment).expect("flash-monitor");

        // Assert
        let log_path = evidence_dir.join("flash-monitor.log");
        let log = std::fs::read_to_string(log_path.as_std_path()).expect("log");
        assert!(log.contains("ssid=lab-net"));
        assert!(log.contains("wifi:connected with LabNet, aid = 1"));
        assert!(log.contains("password=[redacted]"));
        assert!(log.contains("token=[redacted]"));
        assert!(log.contains("ipv4=192.168.1.24"));
        assert!(log.contains("mac=aa:bb:cc:dd:ee:ff"));
        assert!(log.contains("device_url=http://192.168.1.24"));
        assert!(!log.contains("super-secret"));
        assert!(!log.contains("api-secret"));
    }

    #[test]
    fn flash_monitor_commit_redacted_sanitizes_network_identifiers() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let evidence_dir = dir_path(&dir).join("evidence");
        let mut command = flash_monitor_fixture(&dir, evidence_dir.clone());
        command.common.redact_evidence = true;
        let sensitive_log = format!(
            "{}\nI (3863) wifi:connected with LabNet, aid = 1, channel 11, BW20, bssid = aa:bb:cc:dd:ee:ff\nwifi_status=connected ssid=lab-net password=super-secret ipv4=192.168.1.24 mac=aa:bb:cc:dd:ee:ff device_url=http://192.168.1.24\n",
            trusted_monitor_log()
        );
        let environment = FakeFlashEnvironment::default().with_log_contents(&sensitive_log);

        // Act
        run_flash_monitor(&command, &environment).expect("flash-monitor");

        // Assert
        let log_path = evidence_dir.join("flash-monitor.log");
        let log = std::fs::read_to_string(log_path.as_std_path()).expect("log");
        assert!(log.contains("ssid=[redacted]"));
        assert!(log.contains("wifi:connected with [redacted-ssid], aid = 1"));
        assert!(log.contains("password=[redacted]"));
        assert!(log.contains("ipv4=[redacted-ip]"));
        assert!(log.contains("mac=[redacted-mac]"));
        assert!(log.contains("device_url=[redacted-url]"));
        assert!(!log.contains("LabNet"));
        assert!(!log.contains("lab-net"));
        assert!(!log.contains("super-secret"));
        assert!(!log.contains("192.168.1.24"));
        assert!(!log.contains("aa:bb:cc:dd:ee:ff"));
        assert!(!log.contains("http://192.168.1.24"));
    }

    #[test]
    fn flash_monitor_dual_mode_stages_private_input_until_explicit_finalization() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let evidence_dir = dir_path(&dir).join("evidence");
        let mut command = flash_monitor_fixture(&dir, evidence_dir.clone());
        command.common.evidence_mode = Some(EvidenceMode::Dual);
        let sensitive_log = format!(
            "{}\nwifi_status=connected ssid=lab-net password=super-secret ipv4=192.168.1.24 path=/Users/operator/private.log pid=123\n",
            trusted_monitor_log()
        );
        let environment = FakeFlashEnvironment::default().with_log_contents(&sensitive_log);

        // Act
        run_flash_monitor(&command, &environment).expect("flash-monitor");

        // Assert
        let private_path = evidence_dir.join("flash-monitor.classifier-input.log");
        let admitted_path = evidence_dir.join("flash-monitor.log");
        let private = std::fs::read_to_string(private_path.as_std_path()).expect("private");
        assert!(private.contains("ssid=lab-net"));
        assert!(private.contains("ipv4=192.168.1.24"));
        assert!(private.contains("/Users/operator/private.log"));
        assert!(private.contains("pid=123"));
        assert!(private.contains("password=[redacted]"));
        assert!(!private.contains("super-secret"));
        assert!(!admitted_path.exists());
        assert!(!evidence_dir.join("flash-command-evidence.json").exists());
        let evidence = std::fs::read_to_string(
            evidence_dir
                .join("flash-command-evidence.private.json")
                .as_std_path(),
        )
        .expect("private evidence");
        let json: serde_json::Value = serde_json::from_str(&evidence).expect("private json");
        assert_eq!(json["redaction_mode"], "dual");
        assert_eq!(json["monitor_log_path"], admitted_path.as_str());
        assert_eq!(json["private_monitor_log_path"], private_path.as_str());
        assert_eq!(json["private_log_role"], "classifier-input-private");
        assert_eq!(json["commit_ready"], false);
        assert_eq!(
            json["private_monitor_log_sha256"],
            sha256_bytes(private.as_bytes())
        );
        assert!(json.get("monitor_log_sha256").is_none());

        // Act
        run_finalize_evidence(
            &FinalizeEvidenceCommand {
                evidence_dir: evidence_dir.clone(),
                expected_private_sha256: sha256_bytes(private.as_bytes()),
            },
            &environment,
        )
        .expect("finalize evidence");

        // Assert
        let admitted = std::fs::read_to_string(admitted_path.as_std_path()).expect("admitted");
        assert!(!admitted.contains("lab-net"));
        assert!(!admitted.contains("192.168.1.24"));
        assert!(!admitted.contains("/Users/operator/private.log"));
        assert!(!admitted.contains("pid=123"));
        assert_eq!(
            sha256_bytes(private.as_bytes()),
            evidence::private_log_sha256(&private_path).expect("private digest after finalization")
        );
        let admitted_evidence = std::fs::read_to_string(
            evidence_dir
                .join("flash-command-evidence.json")
                .as_std_path(),
        )
        .expect("admitted evidence");
        let admitted_json: serde_json::Value =
            serde_json::from_str(&admitted_evidence).expect("admitted json");
        assert_eq!(admitted_json["commit_ready"], true);
        assert_eq!(admitted_json["monitor_log_path"], "flash-monitor.log");
        assert_eq!(
            admitted_json["monitor_log_sha256"],
            sha256_bytes(admitted.as_bytes())
        );
        assert!(admitted_json.get("private_monitor_log_path").is_none());
        assert!(admitted_json.get("private_monitor_log_sha256").is_none());
        assert!(!admitted_evidence.contains(private_path.as_str()));
        #[cfg(unix)]
        for path in [
            private_path,
            admitted_path,
            evidence_dir.join("flash-command-evidence.private.json"),
            evidence_dir.join("flash-command-evidence.json"),
        ] {
            use std::os::unix::fs::PermissionsExt;
            let mode = std::fs::metadata(path.as_std_path())
                .expect("evidence metadata")
                .permissions()
                .mode()
                & 0o777;
            assert_eq!(mode, 0o600);
        }
    }

    #[test]
    fn flash_monitor_dual_mode_rejects_unapproved_root_before_any_flash_effect() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let evidence_dir = dir_path(&dir).join("shareable-evidence");
        let mut command = flash_monitor_fixture(&dir, evidence_dir);
        command.common.evidence_mode = Some(EvidenceMode::Dual);
        let environment = FakeFlashEnvironment::default().with_private_root_rejected();

        // Act
        let result = run_flash_monitor(&command, &environment);

        // Assert
        let error = result.expect_err("unapproved private evidence root");
        assert!(format!("{error:#}").contains("root_admission_failed"));
        assert_eq!(environment.private_root_admission_calls(), 1);
        assert_eq!(environment.list_ports_calls(), 0);
        assert!(environment.executed_commands().is_empty());
        assert!(environment.captured_commands().is_empty());
    }

    #[test]
    fn local_private_root_admission_requires_workspace_containment_and_git_ignore() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let workspace = dir_path(&dir).join("workspace");
        std::fs::create_dir_all(workspace.join("docs/shareable").as_std_path())
            .expect("shareable dir");
        std::fs::write(workspace.join(".gitignore").as_std_path(), "scratch/\n")
            .expect("gitignore");
        std::fs::write(
            workspace.join("docs/shareable/marker.md").as_std_path(),
            "tracked\n",
        )
        .expect("tracked marker");
        let init_status = Command::new("git")
            .current_dir(workspace.as_std_path())
            .args(["init", "--quiet"])
            .status()
            .expect("git init");
        assert!(init_status.success());
        let add_status = Command::new("git")
            .current_dir(workspace.as_std_path())
            .args(["add", ".gitignore", "docs/shareable/marker.md"])
            .status()
            .expect("git add");
        assert!(add_status.success());

        // Act
        let ignored = approve_local_private_evidence_root(
            &workspace,
            &workspace.join("scratch/phase35-private"),
        );
        let tracked =
            approve_local_private_evidence_root(&workspace, &workspace.join("docs/shareable"));
        let outside =
            approve_local_private_evidence_root(&workspace, &dir_path(&dir).join("outside"));

        // Assert
        ignored.expect("ignored private root");
        assert!(format!("{:#}", tracked.expect_err("tracked root")).contains("not_repo_ignored"));
        assert!(format!("{:#}", outside.expect_err("outside root")).contains("outside_workspace"));
    }

    #[test]
    fn flash_monitor_dual_mode_rejects_existing_destination_before_flash() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let evidence_dir = dir_path(&dir).join("evidence");
        std::fs::create_dir_all(evidence_dir.as_std_path()).expect("evidence dir");
        std::fs::write(
            evidence_dir.join("flash-monitor.log").as_std_path(),
            "existing",
        )
        .expect("existing output");
        let mut command = flash_monitor_fixture(&dir, evidence_dir);
        command.common.evidence_mode = Some(EvidenceMode::Dual);
        let environment = FakeFlashEnvironment::default();

        // Act
        let result = run_flash_monitor(&command, &environment);

        // Assert
        let error = result.expect_err("existing destination");
        assert!(format!("{error:#}").contains("path_preflight_failed"));
        assert!(environment.executed_commands().is_empty());
        assert!(environment.captured_commands().is_empty());
    }

    #[test]
    fn evidence_sanitizer_developer_raw_preserves_network_fields_and_redacts_secrets() {
        // Arrange
        let text = r#"{"ssid":"lab-net","wifiPass":"super-secret","ipv4":"192.168.1.24","mac":"aa:bb:cc:dd:ee:ff","device_url":"http://192.168.1.24","token":"api-secret"}"#;

        // Act
        let sanitized = sanitize_evidence_text(text, EvidenceRedactionMode::DeveloperRaw);

        // Assert
        assert!(sanitized.contains(r#""ssid":"lab-net""#));
        assert!(sanitized.contains(r#""wifiPass":"[redacted]""#));
        assert!(sanitized.contains(r#""ipv4":"192.168.1.24""#));
        assert!(sanitized.contains(r#""mac":"aa:bb:cc:dd:ee:ff""#));
        assert!(sanitized.contains(r#""device_url":"http://192.168.1.24""#));
        assert!(sanitized.contains(r#""token":"[redacted]""#));
        assert!(!sanitized.contains("super-secret"));
        assert!(!sanitized.contains("api-secret"));
    }

    #[test]
    fn evidence_sanitizer_redacts_numeric_never_persist_json_scalars() {
        // Arrange
        let text = r#"{"poolPort":3333,"poolUser":"owner.worker","wifiPass":"super-secret"}"#;

        // Act
        let sanitized = sanitize_evidence_text(text, EvidenceRedactionMode::DeveloperRaw);

        // Assert
        assert!(sanitized.contains(r#""poolPort":"[redacted]""#));
        assert!(sanitized.contains(r#""poolUser":"[redacted]""#));
        assert!(sanitized.contains(r#""wifiPass":"[redacted]""#));
        assert!(!sanitized.contains("3333"));
        assert!(!sanitized.contains("owner.worker"));
        assert!(!sanitized.contains("super-secret"));
    }

    #[test]
    fn evidence_sanitizer_commit_redacted_redacts_json_wifi_fields_network_urls_ips_and_macs() {
        // Arrange
        let text = concat!(
            r#"{"ssid":"lab-net","wifiPass":"super-secret","ipv4":"192.168.1.24","#,
            r#""mac":"aa:bb:cc:dd:ee:ff","device_url":"http://192.168.1.24","#,
            r#""hostname":"miner.local","poolUser":"owner.worker"}"#,
            "\npath=/Users/operator/private.log port=/dev/cu.usbmodem101 pid=123 pgid=456\n",
            "GET /api/system/info HTTP/1.1\nHost: miner.local\n",
        );

        // Act
        let sanitized = sanitize_evidence_text(text, EvidenceRedactionMode::CommitRedacted);

        // Assert
        assert!(sanitized.contains(r#""ssid":"[redacted]""#));
        assert!(sanitized.contains(r#""wifiPass":"[redacted]""#));
        assert!(sanitized.contains(r#""ipv4":"[redacted-ip]""#));
        assert!(sanitized.contains(r#""mac":"[redacted-mac]""#));
        assert!(sanitized.contains(r#""device_url":"[redacted-url]""#));
        assert!(!sanitized.contains("lab-net"));
        assert!(!sanitized.contains("super-secret"));
        assert!(!sanitized.contains("192.168.1.24"));
        assert!(!sanitized.contains("aa:bb:cc:dd:ee:ff"));
        assert!(!sanitized.contains("http://192.168.1.24"));
        assert!(!sanitized.contains("miner.local"));
        assert!(!sanitized.contains("owner.worker"));
        assert!(!sanitized.contains("/Users/operator"));
        assert!(!sanitized.contains("/dev/cu.usbmodem101"));
        assert!(!sanitized.contains("pid=123"));
        assert!(!sanitized.contains("pgid=456"));
        assert!(!sanitized.contains("HTTP/1.1"));
        assert!(sanitized.contains("[redacted-path]"));
        assert!(sanitized.contains("[redacted-http]"));
    }

    #[test]
    fn trusted_timeout_capture_is_accepted() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let evidence_dir = dir_path(&dir).join("evidence");
        let command = flash_monitor_fixture(&dir, evidence_dir.clone());
        let environment =
            FakeFlashEnvironment::default().with_capture_status(CaptureProcessStatus::TimedOut);

        // Act
        let result = run_flash_monitor(&command, &environment);

        // Assert
        assert!(result.is_ok());
        let evidence_path = evidence_dir.join("flash-command-evidence.json");
        let evidence = std::fs::read_to_string(evidence_path.as_std_path()).expect("evidence");
        assert!(evidence.contains(r#""capture_status": "timed_out_after_trusted_output""#));
    }

    #[test]
    fn untrusted_timeout_capture_fails_after_writing_json() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let evidence_dir = dir_path(&dir).join("evidence");
        let command = flash_monitor_fixture(&dir, evidence_dir.clone());
        let environment = FakeFlashEnvironment::default()
            .with_capture_status(CaptureProcessStatus::TimedOut)
            .with_log_contents("untrusted monitor log\n");

        // Act
        let result = run_flash_monitor(&command, &environment);

        // Assert
        let error = format!("{result:#?}");
        assert!(error.contains("evidence capture failed and is not trusted"));
        let evidence_path = evidence_dir.join("flash-command-evidence.json");
        let evidence = std::fs::read_to_string(evidence_path.as_std_path()).expect("evidence");
        assert!(evidence.contains(r#""capture_status": "timed_out_without_trusted_output""#));
    }

    #[test]
    fn stale_firmware_commit_capture_fails_after_writing_json() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let evidence_dir = dir_path(&dir).join("evidence");
        let command = flash_monitor_fixture(&dir, evidence_dir.clone());
        let stale_log = trusted_monitor_log().replace(
            "firmware_commit=0123456789ab",
            "firmware_commit=fedcba987654",
        );
        let environment = FakeFlashEnvironment::default().with_log_contents(&stale_log);

        // Act
        let result = run_flash_monitor(&command, &environment);

        // Assert
        let error = format!("{result:#?}");
        assert!(error.contains("observed firmware_commit=fedcba987654"));
        let evidence_path = evidence_dir.join("flash-command-evidence.json");
        let evidence = std::fs::read_to_string(evidence_path.as_std_path()).expect("evidence");
        assert!(evidence.contains(r#""trusted_output": false"#));
        assert!(evidence.contains(r#""observed_firmware_commit": "fedcba987654""#));
    }

    #[test]
    fn truncated_firmware_commit_capture_fails_after_writing_json() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let evidence_dir = dir_path(&dir).join("evidence");
        let command = flash_monitor_fixture(&dir, evidence_dir.clone());
        let truncated_log =
            trusted_monitor_log().replace("firmware_commit=0123456789ab", "firmware_commit=0");
        let environment = FakeFlashEnvironment::default().with_log_contents(&truncated_log);

        // Act
        let result = run_flash_monitor(&command, &environment);

        // Assert
        let error = format!("{result:#?}");
        assert!(error.contains("observed firmware_commit=0"));
        let evidence_path = evidence_dir.join("flash-command-evidence.json");
        let evidence = std::fs::read_to_string(evidence_path.as_std_path()).expect("evidence");
        assert!(evidence.contains(r#""trusted_output": false"#));
        assert!(evidence.contains(r#""observed_firmware_commit": "0""#));
    }

    #[test]
    fn prefixed_firmware_commit_marker_capture_fails_after_writing_json() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let evidence_dir = dir_path(&dir).join("evidence");
        let command = flash_monitor_fixture(&dir, evidence_dir.clone());
        let prefixed_log = trusted_monitor_log().replace(
            "firmware_commit=0123456789ab",
            "not_firmware_commit=0123456789ab",
        );
        let environment = FakeFlashEnvironment::default().with_log_contents(&prefixed_log);

        // Act
        let result = run_flash_monitor(&command, &environment);

        // Assert
        let error = format!("{result:#?}");
        assert!(error.contains("missing trusted Ultra 205 boot markers"));
        let evidence_path = evidence_dir.join("flash-command-evidence.json");
        let evidence = std::fs::read_to_string(evidence_path.as_std_path()).expect("evidence");
        assert!(evidence.contains(r#""trusted_output": false"#));
        assert!(evidence.contains(r#""observed_firmware_commit": "Unavailable""#));
    }

    #[test]
    fn monitor_failure_guidance_uses_repo_commands() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let evidence_dir = dir_path(&dir).join("evidence");
        let command = flash_monitor_fixture(&dir, evidence_dir.clone());
        let environment = FakeFlashEnvironment::default().with_capture_status(
            CaptureProcessStatus::ExitedFailure("exit status 1".to_owned()),
        );

        // Act
        let result = run_flash_monitor(&command, &environment);

        // Assert
        let error = format!("{result:#?}");
        assert!(error.contains("just detect-ultra205"));
        assert!(error.contains(&format!(
            "just flash-monitor board=205 port=/dev/cu.usbmodem101 evidence-dir={evidence_dir}"
        )));
        assert!(error.contains("just monitor port=/dev/cu.usbmodem101"));
        assert!(error.contains("wrapper noninteractive evidence path"));
        let raw_timeout_command = ["timeout", "25", "espflash"].join(" ");
        assert!(!error.contains(&raw_timeout_command));
    }

    #[test]
    fn rejects_deferred_gamma_601_board() {
        // Arrange
        let input = "601";

        // Act
        let result = input.parse::<BoardId>();

        // Assert
        let error = result.expect_err("deferred board");
        assert!(error.contains("deferred"));
    }

    #[test]
    fn accepts_ultra_205_board() {
        // Arrange
        let input = "205";

        // Act
        let result = input.parse::<BoardId>();

        // Assert
        assert_eq!(result.expect("board"), BoardId::Ultra205);
    }

    fn common_args() -> CommonArgs {
        CommonArgs {
            board: BoardId::Ultra205,
            port: Some("/dev/cu.usbmodem101".to_owned()),
            dry_run: true,
            redact_evidence: false,
            evidence_mode: None,
            evidence_dir: None,
        }
    }

    fn trusted_monitor_log() -> String {
        [
            "bitaxe-rust boot: board=Ultra 205 asic=BM1366",
            "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled",
            "ota_boot_validation=not_pending state=factory",
            "spiffs_mount=available partition=www total_bytes=2884241 used_bytes=4518",
            "axeos_api_route_shell=started registered_routes=15",
            "reset_reason=11",
            "firmware_commit=0123456789ab",
            "reference_commit=abcdef012345",
            "esp_idf_version=v5.5.4",
        ]
        .join("\n")
    }

    fn flash_monitor_fixture(dir: &TempDir, evidence_dir: Utf8PathBuf) -> FlashMonitorCommand {
        let manifest = write_manifest_v3(dir, DEFAULT_ELF_NAME);
        FlashMonitorCommand {
            common: CommonArgs {
                evidence_dir: Some(evidence_dir),
                dry_run: false,
                ..common_args()
            },
            image: None,
            manifest: Some(manifest),
            wifi_credentials: None,
            capture_timeout_seconds: DEFAULT_MONITOR_CAPTURE_TIMEOUT_SECONDS,
        }
    }

    fn dir_path(dir: &TempDir) -> Utf8PathBuf {
        Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).expect("utf8 path")
    }

    fn write_wifi_credentials(dir: &TempDir, ssid: &str, wifi_pass: &str) -> Utf8PathBuf {
        let path = dir_path(dir).join("wifi.json");
        std::fs::write(
            path.as_std_path(),
            serde_json::json!({
                "ssid": ssid,
                "wifiPass": wifi_pass,
            })
            .to_string(),
        )
        .expect("write wifi credentials");
        path
    }

    fn write_manifest(dir: &TempDir, default_flash_image: &str) -> Utf8PathBuf {
        let dir_path = dir_path(dir);
        write_manifest_at(
            &dir_path,
            PACKAGE_MANIFEST_RELATIVE_PATH,
            default_flash_image,
        )
    }

    fn write_manifest_at(
        workspace_dir: &Utf8Path,
        manifest_relative_path: &str,
        default_flash_image: &str,
    ) -> Utf8PathBuf {
        let manifest = workspace_dir.join(manifest_relative_path);
        let manifest_dir = manifest.parent().expect("parent");
        std::fs::create_dir_all(manifest_dir.as_std_path()).expect("create manifest dir");
        write_manifest_v3_contents(&manifest, default_flash_image, FACTORY_IMAGE_NAME);
        manifest
    }

    fn write_manifest_v3(dir: &TempDir, default_flash_image: &str) -> Utf8PathBuf {
        write_manifest_v3_with_factory_artifact(dir, default_flash_image, FACTORY_IMAGE_NAME)
    }

    fn write_manifest_v3_with_factory_artifact(
        dir: &TempDir,
        default_flash_image: &str,
        factory_artifact_path: &str,
    ) -> Utf8PathBuf {
        let dir_path = dir_path(dir);
        let manifest = dir_path.join(PACKAGE_MANIFEST_RELATIVE_PATH);
        write_manifest_v3_contents(&manifest, default_flash_image, factory_artifact_path);
        manifest
    }

    fn write_manifest_v3_contents(
        manifest: &Utf8Path,
        default_flash_image: &str,
        factory_artifact_path: &str,
    ) {
        let manifest_dir = manifest.parent().expect("parent");
        std::fs::create_dir_all(manifest_dir.as_std_path()).expect("create manifest dir");
        let elf = b"synthetic firmware elf".to_vec();
        let ota = esp_application_fixture(SOURCE_COMMIT, BUILD_LABEL);
        let partition_table = factory_partition_table_fixture();
        let factory = factory_image_fixture(&partition_table, &ota);
        let www = b"synthetic www".to_vec();
        let otadata = b"synthetic otadata".to_vec();
        let artifacts = [
            ("firmware_elf", DEFAULT_ELF_NAME, elf.as_slice()),
            ("firmware_ota_image", "esp-miner.bin", ota.as_slice()),
            (
                "factory_merged_image",
                factory_artifact_path,
                factory.as_slice(),
            ),
            ("www_spiffs_image", "www.bin", www.as_slice()),
            (
                "partition_table",
                "partition-table.bin",
                partition_table.as_slice(),
            ),
            ("otadata_initial", "otadata-initial.bin", otadata.as_slice()),
        ];
        let mut artifact_values = Vec::new();
        for (kind, path, bytes) in artifacts {
            std::fs::write(manifest_dir.join(path).as_std_path(), bytes).expect("write artifact");
            artifact_values.push(serde_json::json!({
                "kind": kind,
                "path": path,
                "offset": "Unavailable",
                "sha256": sha256_bytes(bytes),
            }));
        }
        let value = serde_json::json!({
            "schema_version": 3,
            "release_name": "bitaxe-ultra205",
            "semantic_version": "0.1.0",
            "source_commit": SOURCE_COMMIT,
            "reference_commit": REFERENCE_COMMIT,
            "app_elf_sha256": APP_ELF_SHA256,
            "build_identity": {
                "label": BUILD_LABEL,
                "channel": "dev",
                "source_dirty": false,
                "release_tag": null
            },
            "default_flash_image": default_flash_image,
            "artifacts": artifact_values,
        });
        std::fs::write(
            manifest.as_std_path(),
            serde_json::to_string_pretty(&value).expect("manifest json"),
        )
        .expect("write manifest");
    }

    fn rewrite_manifest_provenance(manifest: &Utf8Path, provenance: &BuildProvenance) {
        let contents = std::fs::read_to_string(manifest.as_std_path()).expect("read manifest");
        let mut value: serde_json::Value = serde_json::from_str(&contents).expect("manifest json");
        let identity = provenance.build_identity();
        value["semantic_version"] = serde_json::json!(provenance.semantic_version());
        value["source_commit"] = serde_json::json!(identity.source_commit());
        value["reference_commit"] = serde_json::json!(provenance.reference_commit());
        value["build_identity"] = serde_json::json!({
            "label": identity.build_label(),
            "channel": identity.build_channel().as_str(),
            "source_dirty": identity.source_dirty(),
            "release_tag": identity.maybe_release_tag(),
        });

        let ota = esp_application_fixture(identity.source_commit(), identity.build_label());
        let ota_path = manifest
            .parent()
            .expect("manifest parent")
            .join("esp-miner.bin");
        std::fs::write(ota_path.as_std_path(), &ota).expect("rewrite ota");
        let partition_table = factory_partition_table_fixture();
        let factory = factory_image_fixture(&partition_table, &ota);
        let factory_path = manifest
            .parent()
            .expect("manifest parent")
            .join(FACTORY_IMAGE_NAME);
        std::fs::write(factory_path.as_std_path(), &factory).expect("rewrite factory");
        let artifacts = value["artifacts"].as_array_mut().expect("artifacts array");
        let ota_artifact = artifacts
            .iter_mut()
            .find(|artifact| artifact["kind"] == "firmware_ota_image")
            .expect("ota artifact");
        ota_artifact["sha256"] = serde_json::json!(sha256_bytes(&ota));
        let factory_artifact = artifacts
            .iter_mut()
            .find(|artifact| artifact["kind"] == "factory_merged_image")
            .expect("factory artifact");
        factory_artifact["sha256"] = serde_json::json!(sha256_bytes(&factory));

        std::fs::write(
            manifest.as_std_path(),
            serde_json::to_string_pretty(&value).expect("manifest json"),
        )
        .expect("rewrite manifest");
    }

    fn duplicate_manifest_artifact(manifest: &Utf8Path, kind: &str) {
        let contents = std::fs::read_to_string(manifest.as_std_path()).expect("read manifest");
        let mut value: serde_json::Value = serde_json::from_str(&contents).expect("manifest json");
        let artifacts = value["artifacts"].as_array_mut().expect("artifacts array");
        let duplicate = artifacts
            .iter()
            .find(|artifact| artifact["kind"] == kind)
            .expect("artifact kind")
            .clone();
        artifacts.push(duplicate);
        std::fs::write(
            manifest.as_std_path(),
            serde_json::to_string_pretty(&value).expect("manifest json"),
        )
        .expect("rewrite manifest");
    }

    fn add_manifest_artifact(
        manifest: &Utf8Path,
        kind: &str,
        relative_path: &str,
        bytes: &[u8],
    ) -> Utf8PathBuf {
        let manifest_dir = manifest.parent().expect("manifest parent");
        let path = manifest_dir.join(relative_path);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent.as_std_path()).expect("create artifact parent");
        }
        std::fs::write(path.as_std_path(), bytes).expect("write extra artifact");
        let contents = std::fs::read_to_string(manifest.as_std_path()).expect("read manifest");
        let mut value: serde_json::Value = serde_json::from_str(&contents).expect("manifest json");
        value["artifacts"]
            .as_array_mut()
            .expect("artifacts array")
            .push(serde_json::json!({
                "kind": kind,
                "path": relative_path,
                "offset": "Unavailable",
                "sha256": sha256_bytes(bytes),
            }));
        std::fs::write(
            manifest.as_std_path(),
            serde_json::to_string_pretty(&value).expect("manifest json"),
        )
        .expect("rewrite manifest");
        path
    }

    fn run_explicit_image_admission(
        manifest: &Utf8Path,
        image: Utf8PathBuf,
    ) -> Result<FlashOutcome> {
        let command = FlashCommand {
            common: CommonArgs {
                port: None,
                dry_run: false,
                ..common_args()
            },
            image: Some(image),
            manifest: Some(manifest.to_owned()),
            wifi_credentials: Some(Utf8PathBuf::from("/missing/credentials.json")),
        };
        let environment = FakeFlashEnvironment::with_ports(
            "/dev/cu.usbmodem101 USB JTAG\n/dev/cu.usbmodem102 USB JTAG\n",
        );
        run_flash(&command, &environment)
    }

    fn rewrite_manifest_artifact_digest(manifest: &Utf8Path, kind: &str, bytes: &[u8]) {
        let contents = std::fs::read_to_string(manifest.as_std_path()).expect("read manifest");
        let mut value: serde_json::Value = serde_json::from_str(&contents).expect("manifest json");
        let artifact = value["artifacts"]
            .as_array_mut()
            .expect("artifacts array")
            .iter_mut()
            .find(|artifact| artifact["kind"] == kind)
            .expect("artifact kind");
        artifact["sha256"] = serde_json::json!(sha256_bytes(bytes));
        std::fs::write(
            manifest.as_std_path(),
            serde_json::to_string_pretty(&value).expect("manifest json"),
        )
        .expect("rewrite manifest");
    }

    fn rewrite_manifest_application(manifest: &Utf8Path, ota: &[u8]) {
        let manifest_dir = manifest.parent().expect("manifest parent");
        let ota_path = manifest_dir.join("esp-miner.bin");
        std::fs::write(ota_path.as_std_path(), ota).expect("rewrite OTA image");
        let partition_table = factory_partition_table_fixture();
        let factory = factory_image_fixture(&partition_table, ota);
        let factory_path = manifest_dir.join(FACTORY_IMAGE_NAME);
        std::fs::write(factory_path.as_std_path(), &factory).expect("rewrite factory image");
        rewrite_manifest_artifact_digest(manifest, "firmware_ota_image", ota);
        rewrite_manifest_artifact_digest(manifest, "factory_merged_image", &factory);
    }

    fn rewrite_manifest_elf_artifact_only(manifest: &Utf8Path, elf: &[u8]) {
        let elf_path = manifest
            .parent()
            .expect("manifest parent")
            .join(DEFAULT_ELF_NAME);
        std::fs::write(elf_path.as_std_path(), elf).expect("rewrite firmware ELF");
        rewrite_manifest_artifact_digest(manifest, "firmware_elf", elf);
    }

    #[derive(Clone, Copy)]
    enum LayoutFixtureKind {
        DescriptorNotDrom,
        DestinationOverlap,
        AliasOverlap,
    }

    fn assert_parsed_layout_rejected_before_effects(
        fixture_kind: LayoutFixtureKind,
        expected_reason: &str,
        dry_run: bool,
    ) {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v3(&dir, DEFAULT_ELF_NAME);
        let ota = layout_fixture(fixture_kind);
        rewrite_manifest_application(&manifest, &ota);
        let mut args = vec![
            "bitaxe-flash".to_owned(),
            "flash".to_owned(),
            "--board".to_owned(),
            "205".to_owned(),
            "--manifest".to_owned(),
            manifest.to_string(),
        ];
        let environment = if dry_run {
            args.extend([
                "--port".to_owned(),
                "/dev/null".to_owned(),
                "--dry-run".to_owned(),
            ]);
            FakeFlashEnvironment::default()
        } else {
            args.extend([
                "--wifi-credentials".to_owned(),
                "/missing/credentials.json".to_owned(),
            ]);
            FakeFlashEnvironment::with_ports(
                "/dev/cu.usbmodem101 USB JTAG\n/dev/cu.usbmodem102 USB JTAG\n",
            )
        };
        let cli = parse_cli(args).expect("parsed flash command");
        let CliCommand::Flash(command) = cli.command else {
            panic!("expected flash command");
        };

        // Act
        let error = run_flash(&command, &environment)
            .expect_err("layout admission")
            .to_string();

        // Assert
        assert_eq!(
            error,
            format!("identity_admission=blocked reason={expected_reason}")
        );
        assert_eq!(environment.list_ports_calls(), 0);
        assert!(!environment
            .read_string_paths()
            .iter()
            .any(|path| path.as_str().contains("credentials")));
        assert!(environment.generated_nvs_partitions().is_empty());
        assert!(environment.created_snapshot_paths().is_empty());
        assert!(environment.captured_commands().is_empty());
        assert!(environment.executed_commands().is_empty());
        assert!(environment.observed_flashes().is_empty());
    }

    fn layout_fixture(fixture_kind: LayoutFixtureKind) -> Vec<u8> {
        let mut image = esp_application_fixture(SOURCE_COMMIT, BUILD_LABEL);
        match fixture_kind {
            LayoutFixtureKind::DescriptorNotDrom => {
                image[24..28].copy_from_slice(&0x3fc8_8000_u32.to_le_bytes());
            }
            LayoutFixtureKind::DestinationOverlap => {
                append_esp_segment(&mut image, 0x4037_4000, &[0; 4]);
            }
            LayoutFixtureKind::AliasOverlap => {
                image[4..8].copy_from_slice(&0x4037_8000_u32.to_le_bytes());
                let executable_header = second_esp_segment_header(&image);
                image[executable_header..executable_header + 4]
                    .copy_from_slice(&0x4037_8000_u32.to_le_bytes());
                append_esp_segment(&mut image, 0x3fc8_8000, &[0; 4]);
            }
        }
        reseal_esp_application(&mut image);
        image
    }

    fn append_esp_segment(image: &mut Vec<u8>, load_address: u32, payload: &[u8]) {
        let data_end = esp_segment_data_end(image);
        image.truncate(data_end);
        image[1] = image[1].checked_add(1).expect("fixture segment count");
        image.extend_from_slice(&load_address.to_le_bytes());
        image.extend_from_slice(
            &u32::try_from(payload.len())
                .expect("fixture payload length")
                .to_le_bytes(),
        );
        image.extend_from_slice(payload);
    }

    fn esp_segment_data_end(image: &[u8]) -> usize {
        const IMAGE_HEADER_LEN: usize = 24;
        const SEGMENT_HEADER_LEN: usize = 8;

        let mut cursor = IMAGE_HEADER_LEN;
        for _ in 0..usize::from(image[1]) {
            let payload_len = usize::try_from(u32::from_le_bytes(
                image[cursor + 4..cursor + 8]
                    .try_into()
                    .expect("fixture segment length"),
            ))
            .expect("fixture payload length");
            cursor += SEGMENT_HEADER_LEN + payload_len;
        }
        cursor
    }

    fn second_esp_segment_header(image: &[u8]) -> usize {
        const IMAGE_HEADER_LEN: usize = 24;
        const SEGMENT_HEADER_LEN: usize = 8;

        let first_payload_len = usize::try_from(u32::from_le_bytes(
            image[IMAGE_HEADER_LEN + 4..IMAGE_HEADER_LEN + 8]
                .try_into()
                .expect("fixture segment length"),
        ))
        .expect("fixture payload length");
        IMAGE_HEADER_LEN + SEGMENT_HEADER_LEN + first_payload_len
    }

    fn esp_application_fixture(source_commit: &str, build_label: &str) -> Vec<u8> {
        const IMAGE_HEADER_LEN: usize = 24;
        const APP_DESCRIPTOR_LEN: usize = 256;
        const VERSION_OFFSET: usize = 16;
        const VERSION_LEN: usize = 32;
        const ELF_SHA_OFFSET: usize = 144;
        const MMU_PAGE_SIZE_OFFSET: usize = 180;

        let mut descriptor = vec![0_u8; APP_DESCRIPTOR_LEN];
        descriptor[..4].copy_from_slice(&0xABCD_5432_u32.to_le_bytes());
        descriptor[VERSION_OFFSET..VERSION_OFFSET + build_label.len()]
            .copy_from_slice(build_label.as_bytes());
        descriptor[ELF_SHA_OFFSET..ELF_SHA_OFFSET + 32]
            .copy_from_slice(&decode_lower_hex(APP_ELF_SHA256).expect("valid app hash"));
        descriptor[MMU_PAGE_SIZE_OFFSET] = 16;
        assert!(build_label.len() < VERSION_LEN);

        let mut payload = descriptor;
        payload.extend_from_slice(source_commit.as_bytes());
        let mut image = vec![0_u8; IMAGE_HEADER_LEN];
        image[0] = 0xe9;
        image[1] = 2;
        image[2] = 2;
        image[3] = 0x4f;
        image[4..8].copy_from_slice(&0x4037_4000_u32.to_le_bytes());
        image[8] = 0xee;
        image[12..14].copy_from_slice(&9_u16.to_le_bytes());
        image[15..17].copy_from_slice(&0_u16.to_le_bytes());
        image[17..19].copy_from_slice(&99_u16.to_le_bytes());
        image[23] = 1;
        image.extend_from_slice(&0x3c00_0020_u32.to_le_bytes());
        image.extend_from_slice(
            &u32::try_from(payload.len())
                .expect("fixture payload length")
                .to_le_bytes(),
        );
        image.extend_from_slice(&payload);
        image.extend_from_slice(&0x4037_4000_u32.to_le_bytes());
        image.extend_from_slice(&4_u32.to_le_bytes());
        image.extend_from_slice(&[0x13, 0, 0, 0]);
        reseal_esp_application(&mut image);
        image
    }

    fn reseal_esp_application(image: &mut Vec<u8>) {
        const IMAGE_HEADER_LEN: usize = 24;
        const SEGMENT_HEADER_LEN: usize = 8;

        let mut cursor = IMAGE_HEADER_LEN;
        let mut checksum = 0xef_u8;
        for _ in 0..usize::from(image[1]) {
            let payload_start = cursor + SEGMENT_HEADER_LEN;
            let payload_len = usize::try_from(u32::from_le_bytes([
                image[cursor + 4],
                image[cursor + 5],
                image[cursor + 6],
                image[cursor + 7],
            ]))
            .expect("fixture payload length");
            let payload_end = payload_start + payload_len;
            checksum = image[payload_start..payload_end]
                .iter()
                .fold(checksum, |value, byte| value ^ byte);
            cursor = payload_end;
        }
        let padding_len = (15 - (cursor % 16)) % 16;
        image.truncate(cursor);
        image.resize(cursor + padding_len, 0);
        image.push(checksum);
        let digest = Sha256::digest(&*image);
        image.extend_from_slice(&digest);
    }

    fn factory_partition_table_fixture() -> Vec<u8> {
        let mut record = [0_u8; 32];
        record[..2].copy_from_slice(&[0xaa, 0x50]);
        record[2] = 0x00;
        record[3] = 0x00;
        record[4..8].copy_from_slice(&0x10000_u32.to_le_bytes());
        record[8..12].copy_from_slice(&0x400000_u32.to_le_bytes());
        record[12..19].copy_from_slice(b"factory");
        let mut table = record.to_vec();
        table.extend_from_slice(&[0xff; 32]);
        table
    }

    fn factory_image_fixture(partition_table: &[u8], ota: &[u8]) -> Vec<u8> {
        const PARTITION_TABLE_OFFSET: usize = 0x8000;
        const FACTORY_APP_OFFSET: usize = 0x10000;

        let mut factory = vec![0xff; FACTORY_APP_OFFSET + ota.len()];
        factory[PARTITION_TABLE_OFFSET..PARTITION_TABLE_OFFSET + partition_table.len()]
            .copy_from_slice(partition_table);
        factory[FACTORY_APP_OFFSET..FACTORY_APP_OFFSET + ota.len()].copy_from_slice(ota);
        factory
    }

    #[derive(Debug)]
    struct ObservedFlash {
        path: Utf8PathBuf,
        bytes: Vec<u8>,
        unix_mode: Option<u32>,
    }

    #[derive(Debug)]
    struct FakeFlashEnvironment {
        ports: String,
        workspace_dir: Utf8PathBuf,
        executed_commands: RefCell<Vec<CommandSpec>>,
        captured_commands: RefCell<Vec<CommandSpec>>,
        generated_nvs_partitions: RefCell<Vec<(Utf8PathBuf, Utf8PathBuf, String)>>,
        capture_status: CaptureProcessStatus,
        log_contents: String,
        current_provenance: BuildProvenance,
        source_replacement: Option<(Utf8PathBuf, Vec<u8>)>,
        execute_failure: bool,
        snapshot_write_failure: bool,
        list_ports_calls: Cell<usize>,
        read_string_paths: RefCell<Vec<Utf8PathBuf>>,
        created_snapshot_paths: RefCell<Vec<Utf8PathBuf>>,
        observed_flash: RefCell<Vec<ObservedFlash>>,
        private_root_admitted: bool,
        private_root_admission_calls: Cell<usize>,
        phase35_stage_gates: RefCell<Vec<(String, String)>>,
    }

    impl Default for FakeFlashEnvironment {
        fn default() -> Self {
            Self::with_ports("/dev/cu.usbmodem101 USB JTAG")
        }
    }

    impl FakeFlashEnvironment {
        fn with_ports(ports: &str) -> Self {
            Self {
                ports: ports.to_owned(),
                workspace_dir: Utf8PathBuf::from_path_buf(env::current_dir().expect("current dir"))
                    .expect("utf8 current dir"),
                executed_commands: RefCell::new(Vec::new()),
                captured_commands: RefCell::new(Vec::new()),
                generated_nvs_partitions: RefCell::new(Vec::new()),
                capture_status: CaptureProcessStatus::ExitedSuccess,
                log_contents: trusted_monitor_log(),
                current_provenance: BuildProvenance::new(
                    "0.1.0",
                    SOURCE_COMMIT,
                    false,
                    None::<&str>,
                    REFERENCE_COMMIT,
                )
                .expect("default provenance"),
                source_replacement: None,
                execute_failure: false,
                snapshot_write_failure: false,
                list_ports_calls: Cell::new(0),
                read_string_paths: RefCell::new(Vec::new()),
                created_snapshot_paths: RefCell::new(Vec::new()),
                observed_flash: RefCell::new(Vec::new()),
                private_root_admitted: true,
                private_root_admission_calls: Cell::new(0),
                phase35_stage_gates: RefCell::new(Vec::new()),
            }
        }

        fn executed_commands(&self) -> Vec<CommandSpec> {
            self.executed_commands.borrow().clone()
        }

        fn captured_commands(&self) -> Vec<CommandSpec> {
            self.captured_commands.borrow().clone()
        }

        fn generated_nvs_partitions(&self) -> Vec<(Utf8PathBuf, Utf8PathBuf, String)> {
            self.generated_nvs_partitions.borrow().clone()
        }

        fn with_capture_status(mut self, capture_status: CaptureProcessStatus) -> Self {
            self.capture_status = capture_status;
            self
        }

        fn with_log_contents(mut self, log_contents: &str) -> Self {
            self.log_contents = log_contents.to_owned();
            self
        }

        fn with_workspace_dir(mut self, workspace_dir: Utf8PathBuf) -> Self {
            self.workspace_dir = workspace_dir;
            self
        }

        fn with_current_provenance(mut self, current_provenance: BuildProvenance) -> Self {
            self.current_provenance = current_provenance;
            self
        }

        fn with_source_replacement(mut self, path: Utf8PathBuf, bytes: Vec<u8>) -> Self {
            self.source_replacement = Some((path, bytes));
            self
        }

        fn with_execute_failure(mut self) -> Self {
            self.execute_failure = true;
            self
        }

        fn with_snapshot_write_failure(mut self) -> Self {
            self.snapshot_write_failure = true;
            self
        }

        fn with_private_root_rejected(mut self) -> Self {
            self.private_root_admitted = false;
            self
        }

        fn private_root_admission_calls(&self) -> usize {
            self.private_root_admission_calls.get()
        }

        fn created_snapshot_paths(&self) -> std::cell::Ref<'_, Vec<Utf8PathBuf>> {
            self.created_snapshot_paths.borrow()
        }

        fn list_ports_calls(&self) -> usize {
            self.list_ports_calls.get()
        }

        fn read_string_paths(&self) -> std::cell::Ref<'_, Vec<Utf8PathBuf>> {
            self.read_string_paths.borrow()
        }

        fn observed_flashes(&self) -> std::cell::Ref<'_, Vec<ObservedFlash>> {
            self.observed_flash.borrow()
        }

        fn phase35_stage_gates(&self) -> Vec<(String, String)> {
            self.phase35_stage_gates.borrow().clone()
        }
    }

    impl FlashEnvironment for FakeFlashEnvironment {
        fn build_package(&self) -> Result<()> {
            Ok(())
        }

        fn bazel_bin(&self) -> Result<Utf8PathBuf> {
            Ok(Utf8PathBuf::from("/tmp/bazel-bin"))
        }

        fn workspace_path(&self, path: &Utf8Path) -> Utf8PathBuf {
            if path.is_absolute() {
                return path.to_owned();
            }

            self.workspace_dir.join(path)
        }

        fn read_to_string(&self, path: &Utf8Path) -> Result<String> {
            self.read_string_paths.borrow_mut().push(path.to_owned());
            std::fs::read_to_string(path.as_std_path())
                .with_context(|| format!("failed to read fake manifest {path}"))
        }

        fn read_bytes(&self, path: &Utf8Path) -> Result<Vec<u8>> {
            std::fs::read(path.as_std_path())
                .with_context(|| format!("failed to read fake artifact {path}"))
        }

        fn create_admitted_execution_snapshot(
            &self,
            bytes: &[u8],
        ) -> Result<AdmittedExecutionSnapshot> {
            if self.snapshot_write_failure {
                bail!("identity_admission=blocked reason=execution_snapshot_write_failed");
            }
            let snapshot = AdmittedExecutionSnapshot::materialize(bytes)?;
            self.created_snapshot_paths
                .borrow_mut()
                .push(snapshot.path().to_owned());
            Ok(snapshot)
        }

        fn approve_private_evidence_root(&self, _path: &Utf8Path) -> Result<()> {
            self.private_root_admission_calls
                .set(self.private_root_admission_calls.get().saturating_add(1));
            if !self.private_root_admitted {
                bail!("private evidence root rejected by fixture");
            }
            Ok(())
        }

        fn current_provenance(&self) -> Result<BuildProvenance> {
            Ok(self.current_provenance.clone())
        }

        fn list_ports(&self) -> Result<String> {
            self.list_ports_calls
                .set(self.list_ports_calls.get().saturating_add(1));
            Ok(self.ports.clone())
        }

        fn write_file(&self, path: &Utf8Path, contents: &str) -> Result<()> {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent.as_std_path()).expect("create fake file dir");
            }
            std::fs::write(path.as_std_path(), contents).expect("write fake file");
            Ok(())
        }

        fn generate_nvs_partition(
            &self,
            csv_path: &Utf8Path,
            bin_path: &Utf8Path,
            size: &str,
        ) -> Result<()> {
            self.generated_nvs_partitions.borrow_mut().push((
                csv_path.to_owned(),
                bin_path.to_owned(),
                size.to_owned(),
            ));
            if let Some(parent) = bin_path.parent() {
                std::fs::create_dir_all(parent.as_std_path()).expect("create fake nvs dir");
            }
            std::fs::write(bin_path.as_std_path(), b"nvs-image").expect("write fake nvs image");
            Ok(())
        }

        fn execute(&self, command_spec: &CommandSpec) -> Result<()> {
            self.executed_commands
                .borrow_mut()
                .push(command_spec.clone());
            if command_spec.args.first().map(String::as_str) == Some("write-bin")
                && command_spec.args.iter().any(|argument| argument == "0x0")
            {
                if let Some((path, bytes)) = &self.source_replacement {
                    std::fs::write(path.as_std_path(), bytes).expect("replace package source");
                }
                let path = Utf8PathBuf::from(
                    command_spec
                        .args
                        .last()
                        .expect("full flash command image path"),
                );
                let bytes = std::fs::read(path.as_std_path()).expect("read executed image");
                #[cfg(unix)]
                let unix_mode = {
                    use std::os::unix::fs::PermissionsExt;
                    Some(
                        std::fs::metadata(path.as_std_path())
                            .expect("executed image metadata")
                            .permissions()
                            .mode()
                            & 0o777,
                    )
                };
                #[cfg(not(unix))]
                let unix_mode = None;
                self.observed_flash.borrow_mut().push(ObservedFlash {
                    path,
                    bytes,
                    unix_mode,
                });
            }
            if self.execute_failure {
                bail!("sentinel child failure");
            }
            Ok(())
        }

        fn phase35_stage_readiness_gate(&self, stage: &str, port: &str) -> Result<()> {
            self.phase35_stage_gates
                .borrow_mut()
                .push((stage.to_owned(), port.to_owned()));
            Ok(())
        }

        fn execute_capturing(
            &self,
            command_spec: &CommandSpec,
            log_path: &Utf8Path,
            _timeout_seconds: u64,
            redaction_mode: EvidenceRedactionMode,
            create_new: bool,
        ) -> Result<CaptureProcessResult> {
            self.captured_commands
                .borrow_mut()
                .push(command_spec.clone());
            if let Some(parent) = log_path.parent() {
                std::fs::create_dir_all(parent.as_std_path()).expect("create fake log dir");
            }
            let sanitized = sanitize_evidence_text(&self.log_contents, redaction_mode);
            if create_new {
                evidence::write_dual_private_text(log_path, &sanitized)
                    .expect("write fake private monitor log");
            } else {
                std::fs::write(log_path.as_std_path(), sanitized).expect("write fake monitor log");
            }
            Ok(CaptureProcessResult {
                status: self.capture_status.clone(),
            })
        }

        fn firmware_commit(&self) -> String {
            "0123456789abcdef0123456789abcdef01234567".to_owned()
        }

        fn reference_commit(&self) -> String {
            "abcdef012345abcdef012345abcdef012345abcd".to_owned()
        }

        fn write_evidence(&self, path: &Utf8Path, contents: &str) -> Result<()> {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent.as_std_path()).expect("create fake evidence dir");
            }
            std::fs::write(path.as_std_path(), contents).expect("write fake evidence");
            Ok(())
        }
    }
}
