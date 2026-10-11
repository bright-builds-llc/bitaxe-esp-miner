//! `bap-client`: request, subscribe, unsubscribe and set over a BAP adapter.
//!
//! Every inbound frame is printed as one JSON line with sensitive values redacted,
//! followed by one outcome line. The exit status is 0 only for a confirmed or
//! answered operation.

use anyhow::{anyhow, Context, Result};
use bap_client::exchange::{run, Operation, Outcome, Report, Timing};
use bap_client::policy::{admit_setting, parameter, value_source, ValueSource};
use bap_client::port::{admit_adapter_path, AdapterLink};
use bap_client::redaction::printable;
use bap_client::secret::read_private_value;
use clap::{Parser, Subcommand};
use serde_json::json;
use std::io::Write;
use std::path::PathBuf;

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
