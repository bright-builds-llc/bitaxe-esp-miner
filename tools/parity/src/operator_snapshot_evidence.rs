use std::collections::HashSet;
use std::fmt;

use bitaxe_api::{BootSessionId, OperatorSnapshotIdentity, OperatorSnapshotRevision};
use serde::de::{IgnoredAny, MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

const SYSTEM_INFO_JSON_FIELD: &str = "system_info_json";
const LIVE_WEBSOCKET_JSON_FIELD: &str = "live_websocket_json";
const PROJECTED_SESSION_FIELD: &str = "operator_snapshot_boot_session";
const PROJECTED_REVISION_FIELD: &str = "operator_snapshot_revision";

#[derive(Clone, Copy)]
struct JsonProjection<'a> {
    label: &'static str,
    json: &'a str,
}

#[derive(Debug, Eq, PartialEq)]
struct OperatorSnapshotEvidenceReport {
    identities: Vec<(usize, OperatorSnapshotIdentity)>,
    validation_errors: Vec<String>,
}

#[derive(Default)]
struct RawIdentityFields {
    maybe_boot_session: Option<serde_json::Value>,
    maybe_revision: Option<serde_json::Value>,
}

impl<'de> Deserialize<'de> for RawIdentityFields {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(RawIdentityFieldsVisitor)
    }
}

struct RawIdentityFieldsVisitor;

impl<'de> Visitor<'de> for RawIdentityFieldsVisitor {
    type Value = RawIdentityFields;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("an operator snapshot JSON object")
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut fields = RawIdentityFields::default();
        while let Some(key) = map.next_key::<String>()? {
            match key.as_str() {
                "bootSession" => {
                    if fields.maybe_boot_session.is_some() {
                        return Err(serde::de::Error::custom(
                            "operator_snapshot_duplicate_field: bootSession",
                        ));
                    }
                    fields.maybe_boot_session = Some(map.next_value()?);
                }
                "operatorSnapshotRevision" => {
                    if fields.maybe_revision.is_some() {
                        return Err(serde::de::Error::custom(
                            "operator_snapshot_duplicate_field: operatorSnapshotRevision",
                        ));
                    }
                    fields.maybe_revision = Some(map.next_value()?);
                }
                _ => {
                    map.next_value::<IgnoredAny>()?;
                }
            }
        }
        Ok(fields)
    }
}

pub(crate) fn validate_operator_snapshot_documents(
    api_document: &str,
    websocket_document: &str,
    retained_log_document: &str,
) -> Vec<String> {
    let mut validation_errors = Vec::new();
    let maybe_api_json = maybe_parse_single_document_field(
        &mut validation_errors,
        "api.md",
        api_document,
        SYSTEM_INFO_JSON_FIELD,
    );
    let maybe_websocket_json = maybe_parse_single_document_field(
        &mut validation_errors,
        "websocket.md",
        websocket_document,
        LIVE_WEBSOCKET_JSON_FIELD,
    );
    let projections = [
        maybe_api_json.map(|json| JsonProjection {
            label: "api.md",
            json,
        }),
        maybe_websocket_json.map(|json| JsonProjection {
            label: "websocket.md",
            json,
        }),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>();

    let report = validate_operator_snapshot_evidence(&projections, retained_log_document);
    validation_errors.extend(report.validation_errors);
    for (projection_index, identity) in report.identities {
        let projection = projections[projection_index];
        let document = match projection.label {
            "api.md" => api_document,
            "websocket.md" => websocket_document,
            _ => continue,
        };
        validate_redacted_projection(&mut validation_errors, projection.label, document, identity);
    }
    validation_errors
}

fn validate_operator_snapshot_evidence(
    projections: &[JsonProjection<'_>],
    retained_log: &str,
) -> OperatorSnapshotEvidenceReport {
    let mut validation_errors = Vec::new();
    let identities = projections
        .iter()
        .enumerate()
        .filter_map(|projection| {
            maybe_parse_json_identity(
                projection.1.label,
                projection.1.json,
                &mut validation_errors,
            )
            .map(|identity| (projection.0, identity))
        })
        .collect::<Vec<_>>();
    let projected_identities = identities
        .iter()
        .map(|(_, identity)| *identity)
        .collect::<Vec<_>>();
    validate_identity_chronology(&projected_identities, &mut validation_errors);

    let retained_identities = parse_retained_identities(retained_log, &mut validation_errors);
    validate_identity_chronology(&retained_identities, &mut validation_errors);
    validate_one_boot_session(
        projected_identities
            .iter()
            .chain(&retained_identities)
            .copied(),
        &mut validation_errors,
    );
    validate_retained_membership(
        &projected_identities,
        &retained_identities,
        &mut validation_errors,
    );

    OperatorSnapshotEvidenceReport {
        identities,
        validation_errors,
    }
}

fn maybe_parse_json_identity(
    label: &str,
    json: &str,
    validation_errors: &mut Vec<String>,
) -> Option<OperatorSnapshotIdentity> {
    let fields = match serde_json::from_str::<RawIdentityFields>(json) {
        Ok(fields) => fields,
        Err(error) => {
            validation_errors.push(format!("operator_snapshot_json: {label} {error}"));
            return None;
        }
    };
    let (Some(session_value), Some(revision_value)) =
        (fields.maybe_boot_session, fields.maybe_revision)
    else {
        validation_errors.push(format!(
            "operator_snapshot_missing_half: {label} requires bootSession and operatorSnapshotRevision"
        ));
        return None;
    };
    maybe_parse_identity_values(label, &session_value, &revision_value, validation_errors)
}

fn maybe_parse_identity_values(
    label: &str,
    session_value: &serde_json::Value,
    revision_value: &serde_json::Value,
    validation_errors: &mut Vec<String>,
) -> Option<OperatorSnapshotIdentity> {
    let Some(session_text) = session_value.as_str() else {
        validation_errors.push(format!(
            "operator_snapshot_malformed_session: {label} bootSession must be a string"
        ));
        return None;
    };
    if is_host_checkout_shape(session_text) {
        validation_errors.push(format!(
            "operator_snapshot_host_checkout_substitution: {label} bootSession has commit shape"
        ));
        return None;
    }
    let session = match session_text.parse::<BootSessionId>() {
        Ok(session) if session.to_string() != "0".repeat(32) => session,
        Ok(_) => {
            validation_errors.push(format!(
                "operator_snapshot_synthetic_identity: {label} uses the fixture-only boot session"
            ));
            return None;
        }
        Err(error) => {
            validation_errors.push(format!(
                "operator_snapshot_malformed_session: {label} {error}"
            ));
            return None;
        }
    };
    let Some(revision_value) = revision_value.as_u64() else {
        validation_errors.push(format!(
            "operator_snapshot_malformed_revision: {label} operatorSnapshotRevision must be an unsigned integer"
        ));
        return None;
    };
    let revision = match OperatorSnapshotRevision::new(revision_value) {
        Ok(revision) => revision,
        Err(error) => {
            validation_errors.push(format!(
                "operator_snapshot_malformed_revision: {label} {error}"
            ));
            return None;
        }
    };
    Some(OperatorSnapshotIdentity::new(session, revision))
}

fn parse_retained_identities(
    retained_log: &str,
    validation_errors: &mut Vec<String>,
) -> Vec<OperatorSnapshotIdentity> {
    let mut identities = Vec::new();
    let mut seen = HashSet::new();
    for line in retained_log.lines().map(str::trim) {
        if !line.starts_with("operator_snapshot ") {
            continue;
        }
        let Some((session_text, revision_text)) = maybe_parse_retained_marker_fields(line) else {
            validation_errors.push(
                "operator_snapshot_malformed_marker: retained log marker is not exact".to_owned(),
            );
            continue;
        };
        let session_value = serde_json::Value::String(session_text.to_owned());
        let revision_value = match revision_text.parse::<u64>() {
            Ok(revision) => serde_json::Value::from(revision),
            Err(_) => {
                validation_errors.push(
                    "operator_snapshot_malformed_marker: retained log revision is invalid"
                        .to_owned(),
                );
                continue;
            }
        };
        let Some(identity) = maybe_parse_identity_values(
            "retained log",
            &session_value,
            &revision_value,
            validation_errors,
        ) else {
            continue;
        };
        let key = identity_key(identity);
        if !seen.insert(key) {
            validation_errors.push(
                "operator_snapshot_duplicate_marker: retained log repeats one identity".to_owned(),
            );
            continue;
        }
        identities.push(identity);
    }
    identities
}

fn maybe_parse_retained_marker_fields(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix("operator_snapshot session=")?;
    let (session, rest) = rest.split_once(" revision=")?;
    let revision = rest.strip_suffix(" redacted=true")?;
    (!session.is_empty() && !revision.is_empty()).then_some((session, revision))
}

fn validate_identity_chronology(
    identities: &[OperatorSnapshotIdentity],
    validation_errors: &mut Vec<String>,
) {
    for pair in identities.windows(2) {
        let previous = pair[0];
        let current = pair[1];
        if current.revision() < previous.revision() {
            validation_errors.push(format!(
                "operator_snapshot_revision_regression: {} follows {}",
                current.revision(),
                previous.revision()
            ));
        }
        if current.revision() == previous.revision() && current != previous {
            validation_errors.push(
                "operator_snapshot_conflicting_repeat: repeated revision has a different full pair"
                    .to_owned(),
            );
        }
    }
}

fn validate_one_boot_session(
    identities: impl IntoIterator<Item = OperatorSnapshotIdentity>,
    validation_errors: &mut Vec<String>,
) {
    let mut maybe_session = None;
    for identity in identities {
        if let Some(session) = maybe_session {
            if identity.boot_session() != session {
                validation_errors.push(
                    "operator_snapshot_mixed_session: evidence contains more than one boot session"
                        .to_owned(),
                );
                return;
            }
        } else {
            maybe_session = Some(identity.boot_session());
        }
    }
}

fn validate_retained_membership(
    identities: &[OperatorSnapshotIdentity],
    retained_identities: &[OperatorSnapshotIdentity],
    validation_errors: &mut Vec<String>,
) {
    let retained = retained_identities
        .iter()
        .copied()
        .map(identity_key)
        .collect::<HashSet<_>>();
    for identity in identities {
        if !retained.contains(&identity_key(*identity)) {
            validation_errors.push(format!(
                "operator_snapshot_missing_marker: retained log lacks revision {}",
                identity.revision()
            ));
        }
    }
}

fn validate_redacted_projection(
    validation_errors: &mut Vec<String>,
    label: &str,
    document: &str,
    identity: OperatorSnapshotIdentity,
) {
    let maybe_session = maybe_parse_single_document_field(
        validation_errors,
        label,
        document,
        PROJECTED_SESSION_FIELD,
    );
    let maybe_revision = maybe_parse_single_document_field(
        validation_errors,
        label,
        document,
        PROJECTED_REVISION_FIELD,
    );
    let (Some(session), Some(revision)) = (maybe_session, maybe_revision) else {
        return;
    };
    let session_value = serde_json::Value::String(session.to_owned());
    let revision_value = match revision.parse::<u64>() {
        Ok(revision) => serde_json::Value::from(revision),
        Err(_) => {
            validation_errors.push(format!(
                "operator_snapshot_malformed_revision: {label} projected revision is invalid"
            ));
            return;
        }
    };
    let Some(projected_identity) =
        maybe_parse_identity_values(label, &session_value, &revision_value, validation_errors)
    else {
        return;
    };
    if redacted_document_projection(projected_identity) != redacted_document_projection(identity) {
        validation_errors.push(format!(
            "operator_snapshot_projection_mismatch: {label} redacted fields contradict JSON"
        ));
    }
}

fn redacted_document_projection(identity: OperatorSnapshotIdentity) -> String {
    format!(
        "{PROJECTED_SESSION_FIELD}: {}\n{PROJECTED_REVISION_FIELD}: {}\n",
        identity.boot_session(),
        identity.revision()
    )
}

fn maybe_parse_single_document_field<'a>(
    validation_errors: &mut Vec<String>,
    label: &str,
    document: &'a str,
    field: &str,
) -> Option<&'a str> {
    let prefix = format!("{field}:");
    let values = document
        .lines()
        .filter_map(|line| line.trim().strip_prefix(&prefix))
        .map(str::trim)
        .collect::<Vec<_>>();
    match values.as_slice() {
        [value] if !value.is_empty() => Some(*value),
        [] => {
            validation_errors.push(format!(
                "operator_snapshot_missing_field: {label} requires exactly one {field}"
            ));
            None
        }
        _ => {
            validation_errors.push(format!(
                "operator_snapshot_duplicate_field: {label} requires exactly one {field}"
            ));
            None
        }
    }
}

fn is_host_checkout_shape(value: &str) -> bool {
    matches!(value.len(), 12 | 40)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn identity_key(identity: OperatorSnapshotIdentity) -> (String, u64) {
    (
        identity.boot_session().to_string(),
        identity.revision().get(),
    )
}

#[cfg(test)]
mod tests;
