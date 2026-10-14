//! `bap-client`: request, subscribe, unsubscribe and set over a BAP adapter.
//!
//! Every inbound frame is printed as one JSON line with sensitive values redacted,
//! followed by one outcome line. The exit status is 0 only for a confirmed or
//! answered operation.

use anyhow::{anyhow, Context, Result};
use bap_client::evidence::{admit_detector, AttemptRoot};
use bap_client::exchange::{run, Operation, Outcome, Report, Timing};
use bap_client::policy::{admit_setting, parameter, value_source, ValueSource};
use bap_client::port::{admit_adapter_path, AdapterLink};
use bap_client::probe::{probe, MAXIMUM_LISTEN_SECONDS};
use bap_client::redaction::printable;
use bap_client::secret::read_private_value;
use clap::{Parser, Subcommand};
use serde_json::{json, Value};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command as Process;

#[derive(Parser)]
#[command(about = "Talk to a Bitaxe Accessory Port through a USB-to-UART adapter")]
struct Cli {
    /// The adapter's macOS callout node, such as /dev/cu.usbserial-1410.
    #[arg(long)]
    port: String,
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Request one parameter's current value.
    Request { parameter: String },
    /// Subscribe, print updates for a while, then unsubscribe.
    Subscribe {
        parameter: String,
        #[arg(long)]
        interval_ms: Option<u32>,
        #[arg(long, default_value_t = 10)]
        listen_seconds: u64,
    },
    /// End a subscription.
    Unsubscribe { parameter: String },
    /// Run the task contract's read-only probe and seal its evidence.
    Probe {
        /// The attempt child to create inside a mode-0700 parent; it must not exist.
        #[arg(long)]
        evidence_dir: PathBuf,
        /// The detector's saved report, a mode-0600 sibling of the attempt child.
        #[arg(long)]
        detector_report: PathBuf,
        /// The attempt ordinal recorded in the active task.
        #[arg(long)]
        ordinal: u32,
        #[arg(long, default_value_t = 10)]
        listen_seconds: u64,
    },
    /// Change a setting on the device.
    Set {
        parameter: String,
        /// A non-sensitive value.
        #[arg(long)]
        value: Option<String>,
        /// A private (mode 0600) file holding the value; required for credentials.
        #[arg(long)]
        value_file: Option<PathBuf>,
        /// Repeat the parameter name to confirm the change.
        #[arg(long)]
        confirm: String,
        /// Required to change the ASIC frequency or core voltage.
        #[arg(long)]
        allow_asic_setting: bool,
    },
}

const ANSWER_MS: u64 = 3_000;
const QUIET_MS: u64 = 500;

fn main() {
    match execute(Cli::parse()) {
        Ok(true) => {}
        Ok(false) => std::process::exit(2),
        Err(error) => {
            // Categories only: no value or path is ever part of an error here.
            let _ = writeln!(std::io::stderr(), "bap_client_error={error:#}");
            std::process::exit(1);
        }
    }
}

fn execute(cli: Cli) -> Result<bool> {
    admit_adapter_path(&cli.port).map_err(|refusal| anyhow!("port_refused:{refusal:?}"))?;
    if let Command::Probe {
        evidence_dir,
        detector_report,
        ordinal,
        listen_seconds,
    } = cli.command
    {
        return run_probe(
            &cli.port,
            &evidence_dir,
            &detector_report,
            ordinal,
            listen_seconds,
        );
    }
    let (operation, listen_ms) = operation(cli.command)?;
    let mut link = AdapterLink::open(&cli.port).context("adapter_open_failed")?;
    let timing = Timing {
        answer_ms: ANSWER_MS,
        quiet_ms: QUIET_MS,
        listen_ms,
    };
    let report = run(&operation, &mut link, timing).context("adapter_io_failed")?;
    print(&report)?;
    Ok(matches!(
        report.outcome,
        Outcome::Confirmed | Outcome::Answered
    ))
}

/// The admitted operation and how long it listens; refusals happen before any I/O.
fn operation(command: Command) -> Result<(Operation, u64)> {
    let known = |token: &str| parameter(token).map_err(|refusal| anyhow!("refused:{refusal:?}"));
    Ok(match command {
        Command::Request { parameter } => (Operation::Request(known(&parameter)?), 0),
        Command::Subscribe {
            parameter,
            interval_ms,
            listen_seconds,
        } => (
            Operation::Subscribe {
                parameter: known(&parameter)?,
                maybe_interval_ms: interval_ms,
            },
            listen_seconds.saturating_mul(1_000),
        ),
        Command::Unsubscribe { parameter } => (Operation::Unsubscribe(known(&parameter)?), 0),
        Command::Probe { .. } => unreachable!("the probe runs before operations are built"),
        Command::Set {
            parameter,
            value,
            value_file,
            confirm,
            allow_asic_setting,
        } => {
            let parameter = known(&parameter)?;
            let source = value_source(value, value_file)
                .map_err(|refusal| anyhow!("refused:{refusal:?}"))?;
            admit_setting(parameter, &source, &confirm, allow_asic_setting)
                .map_err(|refusal| anyhow!("refused:{refusal:?}"))?;
            let value = match source {
                ValueSource::Inline(value) => value,
                ValueSource::PrivateFile(path) => {
                    read_private_value(&path).context("value_file_unreadable")?
                }
            };
            (Operation::Set { parameter, value }, 0)
        }
    })
}

fn print(report: &Report) -> Result<()> {
    let mut stdout = std::io::stdout().lock();
    for frame in &report.frames {
        let token = frame.parameter_token();
        let line = json!({
            "command": frame.command().token(),
            "parameter": token,
            "value": frame.value().map(|value| printable(token, value)),
        });
        writeln!(stdout, "{line}")?;
    }
    let (outcome, maybe_code) = match &report.outcome {
        Outcome::Confirmed => ("confirmed", None),
        Outcome::Answered => ("answered", None),
        Outcome::Rejected(code) => ("rejected", Some(code.as_str())),
        Outcome::TimedOut => ("timed_out", None),
        Outcome::UnsubscribeUnconfirmed => ("unsubscribe_unconfirmed", None),
    };
    let summary = json!({
        "outcome": outcome,
        "error_code": maybe_code,
        "malformed_lines": report.malformed_lines,
    });
    writeln!(stdout, "{summary}")?;
    Ok(())
}

/// The contract's one read-only attempt: admit, create the attempt child, probe, then
/// seal one record whose outcome keeps the earliest typed failure.
fn run_probe(
    port: &str,
    child: &Path,
    detector_report: &Path,
    ordinal: u32,
    listen_seconds: u64,
) -> Result<bool> {
    let root = AttemptRoot::create(child).context("evidence_root_refused")?;
    let usb_profile =
        admit_detector(detector_report, child).context("detector_admission_refused")?;
    let listen_seconds = listen_seconds.clamp(1, MAXIMUM_LISTEN_SECONDS);
    let (steps, outcome, maybe_signature) = match AdapterLink::open(port)
        .map_err(|_| "adapter_open_failed")
        .and_then(|mut link| probe(&mut link, listen_seconds).map_err(|_| "adapter_io_failed"))
    {
        Ok(record) => (record.steps, record.outcome, record.maybe_signature),
        Err(category) => (
            Vec::new(),
            "stop_hardware_blocker",
            Some(json!({"step": "adapter", "category": category})),
        ),
    };
    let record = json!({
        "schema": "bap-readonly-probe-v1",
        "task": "task-parity-bap-live-accessory-verification",
        "ordinal": ordinal,
        "board": "205",
        "detector_usb_profile": usb_profile,
        "source": source_identity(),
        "firmware_identity": "not_established_by_bap",
        "listen_seconds": listen_seconds,
        "steps": steps,
        "outcome": outcome,
        "boundary_signature": maybe_signature,
        "hardware_qualified": false,
        "promotion": "none",
    });
    root.seal(&record).context("evidence_seal_failed")?;
    writeln!(
        std::io::stdout().lock(),
        "{}",
        json!({"outcome": outcome, "boundary_signature": record["boundary_signature"]})
    )?;
    Ok(outcome == "complete")
}

/// The repository and pinned reference commits, and whether the worktree is clean.
fn source_identity() -> Value {
    let workspace = std::env::var_os("BUILD_WORKSPACE_DIRECTORY")
        .map_or_else(|| PathBuf::from("."), PathBuf::from);
    let git = |directory: &Path, arguments: &[&str]| {
        Process::new("git")
            .arg("-C")
            .arg(directory)
            .args(arguments)
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
    };
    json!({
        "commit": git(&workspace, &["rev-parse", "HEAD"]),
        "clean": git(&workspace, &["status", "--porcelain"]).map(|status| status.is_empty()),
        "reference_commit": git(&workspace.join("reference/esp-miner"), &["rev-parse", "HEAD"]),
    })
}
