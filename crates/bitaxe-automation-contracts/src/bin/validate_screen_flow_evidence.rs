use std::{env, fs, process::ExitCode};

use bitaxe_automation_contracts::ScreenFlowEvidence;

fn main() -> ExitCode {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: validate_screen_flow_evidence <projection.json>");
        return ExitCode::FAILURE;
    };
    let document = match fs::read_to_string(path) {
        Ok(document) => document,
        Err(error) => {
            eprintln!("failed to read screen-flow evidence: {error}");
            return ExitCode::FAILURE;
        }
    };
    let evidence: ScreenFlowEvidence = match serde_json::from_str(&document) {
        Ok(evidence) => evidence,
        Err(error) => {
            eprintln!("failed to parse screen-flow evidence: {error}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(error) = evidence.validate() {
        eprintln!("screen-flow evidence rejected: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
