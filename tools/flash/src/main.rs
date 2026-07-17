use std::collections::BTreeSet;
use std::env;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use bitaxe_api::BuildProvenance;
use bitaxe_config::{
    apply_settings_patch, ConfigValidationError, NvsWrite, RawSettingValue, SettingsPatch,
    SettingsUpdateDecision, NVS_NAMESPACE,
};
use camino::{Utf8Path, Utf8PathBuf};
use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

mod esp32s3_image;
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BoardId {
    Ultra205,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum EvidenceRedactionMode {
    DeveloperRaw,
    CommitRedacted,
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
    ExitedSuccess,
    ExitedFailure(String),
    TimedOut,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Serialize)]
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
    capture_outcome: &'a MonitorCaptureOutcome,
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
    fn execute_capturing(
        &self,
        command_spec: &CommandSpec,
        log_path: &Utf8Path,
        timeout_seconds: u64,
    ) -> Result<CaptureProcessResult>;
    fn firmware_commit(&self) -> String;
    fn reference_commit(&self) -> String;
    fn write_evidence(&self, path: &Utf8Path, contents: &str) -> Result<()>;
}

#[derive(Debug)]
struct LocalFlashEnvironment {
    workspace_dir: Utf8PathBuf,
}

impl LocalFlashEnvironment {
    fn detect() -> Result<Self> {
        Ok(Self {
            workspace_dir: detect_workspace_dir()?,
        })
    }
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
    ) -> Result<CaptureProcessResult> {
        if command_spec.program != "espflash" {
            bail!("unsupported command program: {}", command_spec.program);
        }

        if let Some(parent) = log_path.parent() {
            fs::create_dir_all(parent.as_std_path())
                .with_context(|| format!("failed to create log directory {parent}"))?;
        }

        let log_file = fs::File::create(log_path.as_std_path())
            .with_context(|| format!("failed to create monitor log {log_path}"))?;
        let stderr_log = log_file
            .try_clone()
            .with_context(|| format!("failed to clone monitor log handle {log_path}"))?;

        let mut command = Command::new("espflash");
        for arg in &command_spec.args {
            command.arg(arg);
        }

        let mut child = command
            .stdout(Stdio::from(log_file))
            .stderr(Stdio::from(stderr_log))
            .spawn()
            .with_context(|| format!("failed to spawn {}", command_spec.display()))?;

        let deadline = Duration::from_secs(timeout_seconds);
        let started = Instant::now();
        loop {
            if let Some(status) = child
                .try_wait()
                .with_context(|| format!("failed to poll {}", command_spec.display()))?
            {
                let capture_status = if status.success() {
                    CaptureProcessStatus::ExitedSuccess
                } else {
                    CaptureProcessStatus::ExitedFailure(status.to_string())
                };
                return Ok(CaptureProcessResult {
                    status: capture_status,
                });
            }

            if started.elapsed() >= deadline {
                child
                    .kill()
                    .with_context(|| format!("failed to stop {}", command_spec.display()))?;
                child
                    .wait()
                    .with_context(|| format!("failed to reap {}", command_spec.display()))?;
                return Ok(CaptureProcessResult {
                    status: CaptureProcessStatus::TimedOut,
                });
            }

            std::thread::sleep(Duration::from_millis(200));
        }
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
        let output = Command::new("espflash")
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

        let mut command = Command::new("espflash");
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
    }

    Ok(())
}

fn parse_cli<I, S>(args: I) -> Result<Cli>
where
    I: IntoIterator<Item = S>,
    S: Into<String>,
{
    let normalized = normalize_args(args);
    Cli::try_parse_from(normalized).map_err(anyhow::Error::new)
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
            "capture-timeout-seconds" | "capture_timeout_seconds" => {
                push_flag_value(&mut normalized, "--capture-timeout-seconds", value)
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
    emit_flash_outcome(&outcome)?;

    if !command.common.dry_run {
        environment.execute(&execution_command).map_err(|_| {
            anyhow::anyhow!("flash_execution=failed reason=admitted_image_child_failed")
        })?;
        if let Some(nvs_seed) = &outcome.nvs_seed {
            environment.execute(&nvs_seed.command)?;
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
    let mut flash_common = command.common.clone();
    flash_common.evidence_dir = None;
    let flash_command = FlashCommand {
        common: flash_common,
        image: command.image.clone(),
        manifest: command.manifest.clone(),
        wifi_credentials: command.wifi_credentials.clone(),
    };
    let flash_outcome = run_flash(&flash_command, environment)?;

    if let Some(evidence_dir) = resolved_evidence_dir(&command.common, environment) {
        let monitor_command = prepare_evidence_monitor_command(&command.common, environment)?;
        emit_command("monitor_command", &monitor_command)?;
        let log_path = evidence_dir.join("flash-monitor.log");
        let capture_outcome = if command.common.dry_run {
            environment.write_evidence(
                &log_path,
                "dry-run: espflash monitor was not executed; no hardware log captured\n",
            )?;
            dry_run_monitor_capture_outcome(command.capture_timeout_seconds)
        } else {
            let capture_result = environment.execute_capturing(
                &monitor_command,
                &log_path,
                command.capture_timeout_seconds,
            )?;
            let monitor_log = environment
                .read_to_string(&log_path)
                .with_context(|| format!("failed to read monitor log {log_path}"))?;
            let capture_outcome = monitor_capture_outcome(
                &capture_result.status,
                &monitor_log,
                command.capture_timeout_seconds,
                &environment.firmware_commit(),
                &environment.reference_commit(),
            );
            environment.write_evidence(
                &log_path,
                &sanitize_evidence_text(
                    &monitor_log,
                    EvidenceRedactionMode::from_common(&command.common),
                ),
            )?;
            capture_outcome
        };
        write_flash_monitor_evidence_if_requested(
            &command.common,
            &flash_outcome,
            &monitor_command,
            &evidence_dir,
            &log_path,
            &capture_outcome,
            environment,
        )?;
        if !command.common.dry_run && !capture_outcome.accepted() {
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
        CaptureProcessStatus::ExitedSuccess | CaptureProcessStatus::ExitedFailure(_) => {
            CaptureStatus::Failed
        }
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

fn emit_flash_outcome(outcome: &FlashOutcome) -> Result<()> {
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
    log_path: &Utf8Path,
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
            log_path,
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
        redaction_mode: redaction_mode.as_str().to_owned(),
        commit_ready: redaction_mode.commit_ready(),
        wifi_credentials_source: if outcome.nvs_seed.is_some() {
            "provided-redacted".to_owned()
        } else {
            "not-provided".to_owned()
        },
        monitor_log_path: input.log_path.as_str().to_owned(),
        capture_mode: input.capture_outcome.capture_mode.clone(),
        capture_status: input.capture_outcome.capture_status,
        capture_timeout_seconds: input.capture_outcome.capture_timeout_seconds,
        trusted_output: input.capture_outcome.trusted_output,
        observed_firmware_commit: input.capture_outcome.observed_firmware_commit.clone(),
        observed_reference_commit: input.capture_outcome.observed_reference_commit.clone(),
        conclusion: input.capture_outcome.conclusion.clone(),
    };
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
    let without_secret_json_fields = redact_json_string_fields(
        text,
        &[
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
        ],
    );
    let without_secret_tokens = redact_key_value_tokens(
        &without_secret_json_fields,
        &[
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
        ],
    );

    if redaction_mode == EvidenceRedactionMode::DeveloperRaw {
        return without_secret_tokens;
    }

    let without_network_json_fields = redact_json_string_fields(&without_secret_tokens, &["ssid"]);
    let without_urls = redact_urls(&without_network_json_fields);
    let without_macs = redact_mac_addresses(&without_urls);
    let without_ips = redact_ipv4_addresses(&without_macs);
    let without_wifi_driver_ssids = redact_wifi_driver_connected_ssids(&without_ips);
    redact_key_value_tokens(&without_wifi_driver_ssids, &["ssid"])
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

#[derive(Debug, Serialize)]
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
                        NVS_PARTITION_OFFSET,
                        nvs_seed.image.as_str(),
                    ],
                ),
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
    fn evidence_sanitizer_commit_redacted_redacts_json_wifi_fields_network_urls_ips_and_macs() {
        // Arrange
        let text = r#"{"ssid":"lab-net","wifiPass":"super-secret","ipv4":"192.168.1.24","mac":"aa:bb:cc:dd:ee:ff","device_url":"http://192.168.1.24"}"#;

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

        fn execute_capturing(
            &self,
            command_spec: &CommandSpec,
            log_path: &Utf8Path,
            _timeout_seconds: u64,
        ) -> Result<CaptureProcessResult> {
            self.captured_commands
                .borrow_mut()
                .push(command_spec.clone());
            if let Some(parent) = log_path.parent() {
                std::fs::create_dir_all(parent.as_std_path()).expect("create fake log dir");
            }
            std::fs::write(log_path.as_std_path(), &self.log_contents)
                .expect("write fake monitor log");
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
