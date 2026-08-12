use std::env;
use std::fs::File;

use bitaxe_automation_contracts::AsicResetEvidence;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args_os().nth(1).ok_or("missing evidence path")?;
    let evidence: AsicResetEvidence = serde_json::from_reader(File::open(path)?)?;
    evidence.validate().map_err(Into::into)
}
