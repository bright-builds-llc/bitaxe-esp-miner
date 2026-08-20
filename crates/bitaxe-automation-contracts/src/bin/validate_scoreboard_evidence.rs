use std::fs;

use bitaxe_automation_contracts::{ScoreboardEvidence, ScoreboardEvidenceV2};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args().nth(1).ok_or("evidence path is required")?;
    let document = fs::read_to_string(path)?;
    let value: serde_json::Value = serde_json::from_str(&document)?;
    match value
        .get("schema_version")
        .and_then(serde_json::Value::as_str)
    {
        Some("bitaxe-scoreboard-evidence-v1") => {
            serde_json::from_value::<ScoreboardEvidence>(value)?.validate()?;
        }
        Some("bitaxe-scoreboard-evidence-v2") => {
            serde_json::from_value::<ScoreboardEvidenceV2>(value)?.validate()?;
        }
        _ => return Err("unsupported scoreboard evidence schema".into()),
    }
    Ok(())
}
