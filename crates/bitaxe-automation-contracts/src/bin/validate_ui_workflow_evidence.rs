use std::{env, fs::File};

use bitaxe_automation_contracts::UiWorkflowEvidence;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).ok_or("missing evidence path")?;
    if env::args().nth(2).is_some() {
        return Err("unexpected validation argument".into());
    }
    let evidence: UiWorkflowEvidence = serde_json::from_reader(File::open(path)?)?;
    evidence.validate().map_err(Into::into)
}
