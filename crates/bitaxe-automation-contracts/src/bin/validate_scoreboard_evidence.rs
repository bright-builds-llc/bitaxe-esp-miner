use std::fs::File;

use bitaxe_automation_contracts::ScoreboardEvidence;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("evidence path is required")?;
    let evidence: ScoreboardEvidence = serde_json::from_reader(File::open(path)?)?;
    evidence.validate()?;
    Ok(())
}
