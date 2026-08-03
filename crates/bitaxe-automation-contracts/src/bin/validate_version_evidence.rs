use std::fs::File;

use bitaxe_automation_contracts::VersionEvidence;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("missing version evidence path")?;
    if std::env::args_os().nth(2).is_some() {
        return Err("unexpected validation argument".into());
    }
    let evidence: VersionEvidence = serde_json::from_reader(File::open(path)?)?;
    evidence.validate()?;
    Ok(())
}
