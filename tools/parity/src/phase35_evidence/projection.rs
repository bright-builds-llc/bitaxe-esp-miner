use serde_json::Value;

use super::{
    parse_snapshot_identity, Phase35EvidenceError, Phase35EvidenceRootInput,
    ShareablePhase35Projection,
};

const FORBIDDEN_PROJECTION_KEYS: [&str; 17] = [
    "credential",
    "device",
    "endpoint",
    "hostname",
    "ip",
    "mac",
    "network",
    "origin",
    "password",
    "path",
    "pid",
    "pool",
    "port",
    "raw",
    "secret",
    "ssid",
    "target",
];

pub(super) fn raw_projection_canaries(input: &Phase35EvidenceRootInput) -> Vec<String> {
    let mut values = input
        .inventory
        .iter()
        .map(|entry| entry.path.clone())
        .collect::<Vec<_>>();
    for document in [
        &input.boot_a.system_info_document,
        &input.boot_a.websocket_document,
        &input.boot_b.system_info_document,
        &input.boot_b.websocket_document,
    ] {
        let field = if document.contains("system_info_json:") {
            "system_info_json"
        } else {
            "live_websocket_json"
        };
        if let Ok((session, _)) = parse_snapshot_identity(document, field) {
            values.push(session);
        }
    }
    values
}

pub(super) fn validate_projection(
    projection: &ShareablePhase35Projection,
    raw_canaries: &[String],
) -> Result<(), Phase35EvidenceError> {
    let value = serde_json::to_value(projection)
        .map_err(|_| Phase35EvidenceError::ForbiddenProjectionField)?;
    validate_projection_value(&value, raw_canaries)
}

pub(crate) fn validate_projection_value(
    value: &Value,
    raw_canaries: &[String],
) -> Result<(), Phase35EvidenceError> {
    let serialized =
        serde_json::to_string(value).map_err(|_| Phase35EvidenceError::ForbiddenProjectionField)?;
    if raw_canaries
        .iter()
        .filter(|canary| !canary.is_empty())
        .any(|canary| serialized.contains(canary))
    {
        return Err(Phase35EvidenceError::ForbiddenProjectionField);
    }
    validate_projection_keys(value)
}

fn validate_projection_keys(value: &Value) -> Result<(), Phase35EvidenceError> {
    match value {
        Value::Object(fields) => {
            for (key, child) in fields {
                let normalized = key.to_ascii_lowercase();
                if FORBIDDEN_PROJECTION_KEYS
                    .iter()
                    .any(|forbidden| normalized.contains(forbidden))
                {
                    return Err(Phase35EvidenceError::ForbiddenProjectionField);
                }
                validate_projection_keys(child)?;
            }
        }
        Value::Array(values) => {
            for child in values {
                validate_projection_keys(child)?;
            }
        }
        _ => {}
    }
    Ok(())
}
