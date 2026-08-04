use std::fs::File;

use bitaxe_automation_contracts::OperatorSnapshotEvidence;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("missing operator snapshot evidence path")?;
    if std::env::args_os().nth(2).is_some() {
        return Err("unexpected validation argument".into());
    }
    let evidence: OperatorSnapshotEvidence = serde_json::from_reader(File::open(path)?)?;
    evidence.validate()?;
    Ok(())
}
