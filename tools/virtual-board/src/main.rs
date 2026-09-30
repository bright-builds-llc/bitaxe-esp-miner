//! Host scenario CLI. Physical device discovery is absent from this binary.
mod evidence;
#[path = "../snapshot.rs"]
mod snapshot;
use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(about = "Run deterministic functional Ultra 205 scenarios")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    List,
    Run {
        #[arg(long)]
        scenario: String,
        #[arg(long, default_value_t = 1)]
        seed: u64,
        #[arg(long)]
        evidence_dir: PathBuf,
        #[arg(long)]
        manifest: Option<PathBuf>,
        #[arg(long)]
        workspace_root: Option<PathBuf>,
    },
}
fn execute(cli: Cli) -> Result<bool> {
    match cli.command {
        Command::List => {
            let bytes = serde_json::to_vec_pretty(&bitaxe_simulation::scenario_names())?;
            use std::io::Write;
            std::io::stdout().lock().write_all(&bytes)?;
            Ok(true)
        }
        Command::Run {
            scenario,
            seed,
            evidence_dir,
            manifest,
            workspace_root,
        } => {
            let root = workspace_root
                .or_else(|| std::env::var_os("BUILD_WORKSPACE_DIRECTORY").map(PathBuf::from))
                .unwrap_or(std::env::current_dir()?);
            evidence::run(&root, &evidence_dir, &scenario, seed, manifest.as_deref())
        }
    }
}
fn main() {
    match execute(Cli::parse()) {
        Ok(true) => {}
        Ok(false) => std::process::exit(2),
        Err(error) => {
            tracing::error!(error = %error, "virtual board command failed");
            use std::io::Write;
            if writeln!(std::io::stderr().lock(), "virtual_board_error={error}").is_err() {
                std::process::exit(1);
            }
            std::process::exit(1);
        }
    }
}
