use std::fs::File;

use bitaxe_automation_contracts::RuntimeHealthEvidence;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("missing runtime health evidence path")?;
    if std::env::args_os().nth(2).is_some() {
        return Err("unexpected validation argument".into());
    }
    let evidence: RuntimeHealthEvidence = serde_json::from_reader(File::open(path)?)?;
    evidence.validate()?;
    Ok(())
}
