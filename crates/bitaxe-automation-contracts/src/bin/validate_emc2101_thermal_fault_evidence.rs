use std::{env, fs::File};

use bitaxe_automation_contracts::Emc2101ThermalFaultEvidence;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args_os().nth(1).ok_or("missing evidence path")?;
    let evidence: Emc2101ThermalFaultEvidence = serde_json::from_reader(File::open(path)?)?;
    evidence.validate().map_err(Into::into)
}
