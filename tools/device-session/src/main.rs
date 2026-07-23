use std::fs;
use std::time::Duration;

use anyhow::{Context, Result};
use bitaxe_device_session::{
    run_fixture_session, run_live_session, validate_private_input, FixtureTranscript,
    SessionArtifacts, SessionRequest, TerminalCategory,
};
use camino::Utf8PathBuf;
use clap::{Args, Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "device-session")]
#[command(about = "Deterministic receive-only ESP device session owner.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Reboot(RebootArgs),
}

#[derive(Debug, Args)]
struct RebootArgs {
    #[arg(long = "private-root")]
    private_root: Utf8PathBuf,

    #[arg(long = "request-input")]
    request_input: Utf8PathBuf,

    #[arg(long = "projection-output")]
    projection_output: Utf8PathBuf,

    #[arg(long = "fixture-input")]
    fixture_input: Option<Utf8PathBuf>,

    #[arg(long = "timeout-seconds", default_value_t = 360)]
    timeout_seconds: u64,
}

fn main() {
    match run() {
        Ok(TerminalCategory::Ready) => {}
        Ok(category) => {
            eprintln!(
                "device_session_status=failed category={}",
                category.as_str()
            );
            std::process::exit(1);
        }
        Err(_) => {
            eprintln!("device_session_status=failed category=host_error");
            std::process::exit(1);
        }
    }
}

fn run() -> Result<TerminalCategory> {
    let cli = Cli::parse();
    match cli.command {
        Command::Reboot(args) => run_reboot(args),
    }
}

fn run_reboot(args: RebootArgs) -> Result<TerminalCategory> {
    if args.timeout_seconds == 0 || args.timeout_seconds > 600 {
        anyhow::bail!("timeout-seconds must be between 1 and 600");
    }
    validate_private_input(&args.request_input)?;
    let request_bytes = fs::read(args.request_input.as_std_path())
        .context("failed to read private request input")?;
    let request: SessionRequest = serde_json::from_slice(&request_bytes)
        .context("private request input does not match the device-session schema")?;
    if let Some(fixture_input) = args.fixture_input {
        validate_private_input(&fixture_input)?;
        let fixture_bytes = fs::read(fixture_input.as_std_path())
            .context("failed to read private fixture input")?;
        let fixture: FixtureTranscript = serde_json::from_slice(&fixture_bytes)
            .context("private fixture input does not match the device-session fixture schema")?;
        let artifacts = SessionArtifacts::create(&args.private_root, &args.projection_output)?;
        return run_fixture_session(request, fixture, artifacts);
    }
    let artifacts = SessionArtifacts::create(&args.private_root, &args.projection_output)?;
    run_live_session(
        request,
        artifacts,
        Duration::from_secs(args.timeout_seconds),
    )
}
