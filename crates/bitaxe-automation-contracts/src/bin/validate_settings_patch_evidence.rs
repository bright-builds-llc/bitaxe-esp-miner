use std::{env, fs::File};

use bitaxe_automation_contracts::SettingsPatchEvidence;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).ok_or("missing evidence path")?;
    let evidence: SettingsPatchEvidence = serde_json::from_reader(File::open(path)?)?;
    evidence.validate().map_err(Into::into)
}
