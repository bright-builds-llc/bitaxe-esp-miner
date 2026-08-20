use std::{path::PathBuf, process::ExitCode, time::Duration};

use bitaxe_pool_readiness::{execute, ReadinessDisposition, ReadinessOptions};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "pool-readiness")]
struct Cli {
    #[arg(long)]
    private_root: PathBuf,
    #[arg(long)]
    pool_credentials: PathBuf,
    #[arg(long)]
    attempt_ordinal: u8,
    #[arg(long)]
    samples: u8,
    #[arg(long)]
    sample_timeout_seconds: u64,
    #[arg(long)]
    sample_delay_seconds: u64,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let options = ReadinessOptions {
        private_root: cli.private_root,
        pool_credentials: cli.pool_credentials,
        attempt_ordinal: cli.attempt_ordinal,
        samples: cli.samples,
        sample_timeout: Duration::from_secs(cli.sample_timeout_seconds),
        sample_delay: Duration::from_secs(cli.sample_delay_seconds),
    };

    match execute(options) {
        Ok(ReadinessDisposition::Ready(report)) => {
            println!(
                "pool_readiness=ready samples={} shares_submitted=false",
                report.ready_samples
            );
            ExitCode::SUCCESS
        }
        Ok(ReadinessDisposition::Unavailable(report)) => {
            eprintln!(
                "pool_readiness=unavailable category={} completed={} shares_submitted=false",
                report.terminal_category.as_str(),
                report.samples_completed
            );
            ExitCode::FAILURE
        }
        Err(error) => {
            eprintln!(
                "pool_readiness=unavailable category={} completed=0 shares_submitted=false",
                error.category().as_str()
            );
            ExitCode::FAILURE
        }
    }
}
