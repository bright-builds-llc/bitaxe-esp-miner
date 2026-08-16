use std::{fs::File, path::PathBuf};

use bitaxe_automation_contracts::StatisticsHistoryEvidence;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(std::env::args().nth(1).ok_or("missing evidence path")?);
    let evidence: StatisticsHistoryEvidence = serde_json::from_reader(File::open(path)?)?;
    evidence.validate().map_err(Into::into)
}
