use std::{env, fs, process::ExitCode};

use bitaxe_automation_contracts::InputUatEvidence;

fn main() -> ExitCode {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: validate_input_uat_evidence <projection.json>");
        return ExitCode::FAILURE;
    };
    let document = match fs::read_to_string(path) {
        Ok(document) => document,
        Err(error) => {
            eprintln!("failed to read input UAT evidence: {error}");
            return ExitCode::FAILURE;
        }
    };
    let evidence: InputUatEvidence = match serde_json::from_str(&document) {
        Ok(evidence) => evidence,
        Err(error) => {
            eprintln!("failed to parse input UAT evidence: {error}");
            return ExitCode::FAILURE;
        }
    };
    if let Err(error) = evidence.validate() {
        eprintln!("input UAT evidence rejected: {error}");
        return ExitCode::FAILURE;
    }
    ExitCode::SUCCESS
}
