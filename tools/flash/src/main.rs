use std::collections::BTreeSet;
use std::env;
use std::fmt;
use std::fs;
use std::io::{self, Write};
use std::process::{Command, Stdio};
use std::str::FromStr;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use anyhow::{bail, Context, Result};
use camino::{Utf8Path, Utf8PathBuf};
use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};

const PACKAGE_BUILD_DISPLAY: &str = "bazel build //firmware/bitaxe:firmware_image";
const PACKAGE_BUILD_TARGET: &str = "//firmware/bitaxe:firmware_image";
const PACKAGE_MANIFEST_RELATIVE_PATH: &str = "firmware/bitaxe/bitaxe-ultra205-package.json";
const DEFAULT_ELF_NAME: &str = "bitaxe-ultra205.elf";
const FACTORY_IMAGE_NAME: &str = "bitaxe-ultra205-factory.bin";
const DEFAULT_MONITOR_CAPTURE_TIMEOUT_SECONDS: u64 = 25;
const MIN_COMMIT_PREFIX_LEN: usize = 12;
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

    #[arg(long = "capture-timeout-seconds", default_value_t = DEFAULT_MONITOR_CAPTURE_TIMEOUT_SECONDS)]
    capture_timeout_seconds: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BoardId {
    Ultra205,
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
    default_flash_image: String,
    #[serde(default)]
    artifacts: Vec<PackageArtifact>,
}

#[derive(Debug, Deserialize)]
struct PackageArtifact {
    kind: String,
    path: String,
}

trait FlashEnvironment {
    fn build_package(&self) -> Result<()>;
    fn bazel_bin(&self) -> Result<Utf8PathBuf>;
    fn workspace_path(&self, path: &Utf8Path) -> Utf8PathBuf {
        path.to_owned()
    }
    fn read_to_string(&self, path: &Utf8Path) -> Result<String>;
    fn path_exists(&self, path: &Utf8Path) -> bool;
    fn list_ports(&self) -> Result<String>;
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

    fn path_exists(&self, path: &Utf8Path) -> bool {
        path.is_file()
    }

    fn list_ports(&self) -> Result<String> {
        let output = Command::new("espflash")
            .arg("list-ports")
            .output()
            .context("failed to run espflash list-ports")?;
        command_output_to_string(output, "espflash list-ports")
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
            "evidence-dir" | "evidence_dir" => {
                push_flag_value(&mut normalized, "--evidence-dir", value)
            }
            "capture-timeout-seconds" | "capture_timeout_seconds" => {
                push_flag_value(&mut normalized, "--capture-timeout-seconds", value)
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
    let outcome = prepare_flash(command, environment)?;
    emit_flash_outcome(&outcome)?;

    if !command.common.dry_run {
        environment.execute(&outcome.command)?;
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
            monitor_capture_outcome(
                &capture_result.status,
                &monitor_log,
                command.capture_timeout_seconds,
                &environment.firmware_commit(),
                &environment.reference_commit(),
            )
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
) -> Result<FlashOutcome> {
    ensure_ultra_205(command.common.board)?;
    let port = resolve_port(command.common.port.as_deref(), environment)?;
    let (maybe_manifest, flash_image) = resolve_flash_image(command, environment)?;
    let command = flash_command_for_image(&port, &flash_image)?;

    Ok(FlashOutcome {
        manifest: maybe_manifest,
        flash_image,
        command,
    })
}

fn flash_command_for_image(port: &str, flash_image: &Utf8Path) -> Result<CommandSpec> {
    let file_name = flash_image.file_name().unwrap_or_default();
    if file_name == FACTORY_IMAGE_NAME {
        return Ok(CommandSpec::new(
            "espflash",
            [
                "write-bin",
                "--chip",
                "esp32s3",
                "--port",
                port,
                "0x0",
                flash_image.as_str(),
            ],
        ));
    }

    Ok(CommandSpec::new(
        "espflash",
        [
            "flash",
            "--chip",
            "esp32s3",
            "--port",
            port,
            flash_image.as_str(),
        ],
    ))
}

fn resolve_flash_image(
    command: &FlashCommand,
    environment: &impl FlashEnvironment,
) -> Result<(Option<Utf8PathBuf>, Utf8PathBuf)> {
    if let Some(image) = &command.image {
        return Ok((None, image.clone()));
    }

    environment.build_package()?;
    let manifest = match &command.manifest {
        Some(path) => path.clone(),
        None => environment
            .bazel_bin()?
            .join(PACKAGE_MANIFEST_RELATIVE_PATH),
    };
    let manifest_contents = environment.read_to_string(&manifest)?;
    let package_manifest: PackageManifest = serde_json::from_str(&manifest_contents)
        .with_context(|| format!("failed to parse package manifest {manifest}"))?;
    let flash_image = resolve_manifest_flash_image(&manifest, &package_manifest)?;

    if !environment.path_exists(&flash_image) {
        bail!("manifest default flash image does not exist: {flash_image}");
    }

    Ok((Some(manifest), flash_image))
}

fn resolve_manifest_flash_image(
    manifest: &Utf8Path,
    package_manifest: &PackageManifest,
) -> Result<Utf8PathBuf> {
    if let Some(factory_artifact) = package_manifest
        .artifacts
        .iter()
        .find(|artifact| artifact.kind == "factory_merged_image")
    {
        return resolve_manifest_factory_artifact(manifest, Utf8Path::new(&factory_artifact.path));
    }

    let default_flash_image = Utf8PathBuf::from(&package_manifest.default_flash_image);
    resolve_manifest_default(manifest, &default_flash_image)
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
    [
        "bitaxe-rust boot: board=Ultra 205 asic=BM1366",
        "safe_state: mining=disabled asic_work_submission=disabled hardware_control=disabled",
        "ota_boot_validation=",
        "spiffs_mount=available",
        "axeos_api_route_shell=started",
        "reset_reason=",
        "firmware_commit=",
        "reference_commit=",
        "esp_idf_version=",
    ]
    .iter()
    .all(|marker| log.contains(marker))
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
        .find_map(|line| {
            line.split_once(marker)
                .and_then(|(_, value)| value.split_whitespace().next())
                .map(str::to_owned)
        })
        .filter(|value| !value.is_empty())
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
    emit_command("flash_command", &outcome.command)
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
    write_evidence_record(
        common,
        outcome,
        &evidence_dir,
        EvidenceRecordInput {
            command_kind,
            command: &outcome.command.display(),
            flash_command: &outcome.command.display(),
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
    let command = format!(
        "flash: {}\nmonitor: {}",
        outcome.command.display(),
        monitor_command.display()
    );
    write_evidence_record(
        common,
        outcome,
        evidence_dir,
        EvidenceRecordInput {
            command_kind: "flash-monitor",
            command: &command,
            flash_command: &outcome.command.display(),
            monitor_command: &monitor_command.display(),
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
    environment.write_evidence(&evidence_dir.join("flash-command-evidence.json"), &json)
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
    use std::cell::RefCell;
    use tempfile::{tempdir, TempDir};

    #[test]
    fn parses_key_value_aliases_for_flash() {
        // Arrange
        let args = [
            "bitaxe-flash",
            "flash",
            "board=205",
            "dry-run=true",
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
    fn dry_run_flash_with_explicit_image_renders_vector_command() {
        // Arrange
        let command = FlashCommand {
            common: common_args(),
            image: Some(Utf8PathBuf::from("/tmp/bitaxe-ultra205.elf")),
            manifest: None,
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
    fn dry_run_flash_resolves_manifest_default_elf() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest(&dir, DEFAULT_ELF_NAME);
        let command = FlashCommand {
            common: common_args(),
            image: None,
            manifest: Some(manifest.clone()),
        };
        let environment = FakeFlashEnvironment::default();

        // Act
        let outcome = run_flash(&command, &environment).expect("flash");

        // Assert
        assert_eq!(outcome.manifest.as_ref(), Some(&manifest));
        assert_eq!(
            outcome.flash_image,
            manifest.parent().expect("parent").join(DEFAULT_ELF_NAME)
        );
        assert_eq!(
            outcome.command.args,
            vec![
                "flash",
                "--chip",
                "esp32s3",
                "--port",
                "/dev/cu.usbmodem101",
                outcome.flash_image.as_str(),
            ]
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
        };
        let environment = FakeFlashEnvironment::default();

        // Act
        let result = run_flash(&command, &environment);

        // Assert
        assert!(format!("{result:#?}").contains(DEFAULT_ELF_NAME));
    }

    #[test]
    fn manifest_v2_uses_factory_artifact_for_full_flash() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v2(&dir, DEFAULT_ELF_NAME);
        let command = FlashCommand {
            common: common_args(),
            image: None,
            manifest: Some(manifest.clone()),
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
    fn manifest_v2_rejects_wrong_factory_artifact_name() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let manifest = write_manifest_v2_with_factory_artifact(&dir, DEFAULT_ELF_NAME, "wrong.bin");
        let command = FlashCommand {
            common: common_args(),
            image: None,
            manifest: Some(manifest),
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

        // Act
        let trusted = monitor_log_has_trusted_boot_markers(&trusted_log);
        let unsafe_markers = monitor_log_has_trusted_boot_markers(&unsafe_log);

        // Assert
        assert!(trusted);
        assert!(!unsafe_markers);
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
    fn flash_monitor_evidence_points_to_created_log() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let evidence_dir = dir_path(&dir).join("evidence");
        let command = FlashMonitorCommand {
            common: CommonArgs {
                evidence_dir: Some(evidence_dir.clone()),
                dry_run: false,
                ..common_args()
            },
            image: Some(Utf8PathBuf::from("/tmp/bitaxe-ultra205.elf")),
            manifest: None,
            capture_timeout_seconds: DEFAULT_MONITOR_CAPTURE_TIMEOUT_SECONDS,
        };
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
        let command = flash_monitor_command(evidence_dir.clone());
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
        let command = flash_monitor_command(evidence_dir);
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
        let command = flash_monitor_command(evidence_dir.clone());
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
    fn trusted_timeout_capture_is_accepted() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let evidence_dir = dir_path(&dir).join("evidence");
        let command = flash_monitor_command(evidence_dir.clone());
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
        let command = flash_monitor_command(evidence_dir.clone());
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
        let command = flash_monitor_command(evidence_dir.clone());
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
        let command = flash_monitor_command(evidence_dir.clone());
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
    fn monitor_failure_guidance_uses_repo_commands() {
        // Arrange
        let dir = tempdir().expect("tempdir");
        let evidence_dir = dir_path(&dir).join("evidence");
        let command = flash_monitor_command(evidence_dir.clone());
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

    fn flash_monitor_command(evidence_dir: Utf8PathBuf) -> FlashMonitorCommand {
        FlashMonitorCommand {
            common: CommonArgs {
                evidence_dir: Some(evidence_dir),
                dry_run: false,
                ..common_args()
            },
            image: Some(Utf8PathBuf::from("/tmp/bitaxe-ultra205.elf")),
            manifest: None,
            capture_timeout_seconds: DEFAULT_MONITOR_CAPTURE_TIMEOUT_SECONDS,
        }
    }

    fn dir_path(dir: &TempDir) -> Utf8PathBuf {
        Utf8PathBuf::from_path_buf(dir.path().to_path_buf()).expect("utf8 path")
    }

    fn write_manifest(dir: &TempDir, default_flash_image: &str) -> Utf8PathBuf {
        let dir_path = dir_path(dir);
        let manifest = dir_path.join(PACKAGE_MANIFEST_RELATIVE_PATH);
        let manifest_dir = manifest.parent().expect("parent");
        std::fs::create_dir_all(manifest_dir.as_std_path()).expect("create manifest dir");
        let default_image = manifest_dir.join(default_flash_image);
        std::fs::write(default_image.as_std_path(), b"image").expect("write image");
        std::fs::write(
            manifest.as_std_path(),
            format!(r#"{{"default_flash_image":"{default_flash_image}"}}"#),
        )
        .expect("write manifest");
        manifest
    }

    fn write_manifest_v2(dir: &TempDir, default_flash_image: &str) -> Utf8PathBuf {
        write_manifest_v2_with_factory_artifact(dir, default_flash_image, FACTORY_IMAGE_NAME)
    }

    fn write_manifest_v2_with_factory_artifact(
        dir: &TempDir,
        default_flash_image: &str,
        factory_artifact_path: &str,
    ) -> Utf8PathBuf {
        let dir_path = dir_path(dir);
        let manifest = dir_path.join(PACKAGE_MANIFEST_RELATIVE_PATH);
        let manifest_dir = manifest.parent().expect("parent");
        std::fs::create_dir_all(manifest_dir.as_std_path()).expect("create manifest dir");
        let default_image = manifest_dir.join(default_flash_image);
        std::fs::write(default_image.as_std_path(), b"image").expect("write image");
        for file_name in [
            DEFAULT_ELF_NAME,
            "esp-miner.bin",
            "www.bin",
            "otadata-initial.bin",
            FACTORY_IMAGE_NAME,
        ] {
            let path = manifest_dir.join(file_name);
            if !path.is_file() {
                std::fs::write(path.as_std_path(), b"artifact").expect("write artifact");
            }
        }

        std::fs::write(
            manifest.as_std_path(),
            format!(
                r#"{{
  "schema_version": 2,
  "release_name": "bitaxe-ultra205",
  "source_commit": "source-commit",
  "reference_commit": "reference-commit",
  "default_flash_image": "{default_flash_image}",
  "image_metadata": {{
    "board": "205",
    "device_model": "Ultra 205",
    "asic": "BM1366",
    "esp_idf_version": "v5.5.4",
    "rust_target": "xtensa-esp32s3-espidf"
  }},
  "tool_versions": {{
    "cargo": "cargo 1.0.0",
    "rustc": "rustc 1.0.0",
    "bazel": "bazel 1.0.0",
    "espflash": "espflash 1.0.0"
  }},
  "install_notes": {{"path": "docs/release/ultra-205.md", "summary": "Ultra 205 guide"}},
  "license_inventory": "docs/release/license-inventory.md",
  "provenance_manifest": "docs/release/provenance-manifest.md",
  "otadata_source": "generated-erased-flash",
  "artifacts": [
    {{"kind": "firmware_elf", "path": "bitaxe-ultra205.elf", "offset": "Unavailable", "sha256": "00"}},
    {{"kind": "firmware_ota_image", "path": "esp-miner.bin", "offset": "0x10000", "sha256": "11"}},
    {{"kind": "www_spiffs_image", "path": "www.bin", "offset": "0x410000", "sha256": "22"}},
	    {{"kind": "factory_merged_image", "path": "{factory_artifact_path}", "offset": "0x0", "sha256": "33"}},
    {{"kind": "partition_table", "path": "firmware/bitaxe/partitions-ultra205.csv", "offset": "Unavailable", "sha256": "44"}},
    {{"kind": "otadata_initial", "path": "otadata-initial.bin", "offset": "0xf10000", "sha256": "55"}}
  ]
}}"#
            ),
        )
        .expect("write manifest");
        manifest
    }

    #[derive(Debug)]
    struct FakeFlashEnvironment {
        ports: String,
        workspace_dir: Utf8PathBuf,
        executed_commands: RefCell<Vec<CommandSpec>>,
        captured_commands: RefCell<Vec<CommandSpec>>,
        capture_status: CaptureProcessStatus,
        log_contents: String,
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
                capture_status: CaptureProcessStatus::ExitedSuccess,
                log_contents: trusted_monitor_log(),
            }
        }

        fn executed_commands(&self) -> Vec<CommandSpec> {
            self.executed_commands.borrow().clone()
        }

        fn captured_commands(&self) -> Vec<CommandSpec> {
            self.captured_commands.borrow().clone()
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
            std::fs::read_to_string(path.as_std_path())
                .with_context(|| format!("failed to read fake manifest {path}"))
        }

        fn path_exists(&self, path: &Utf8Path) -> bool {
            path.is_file()
        }

        fn list_ports(&self) -> Result<String> {
            Ok(self.ports.clone())
        }

        fn execute(&self, command_spec: &CommandSpec) -> Result<()> {
            self.executed_commands
                .borrow_mut()
                .push(command_spec.clone());
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
