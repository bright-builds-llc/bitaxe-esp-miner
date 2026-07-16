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
    let maybe_api_json = parse_single_document_field(
        &mut validation_errors,
        "api.md",
        api_document,
        SYSTEM_INFO_JSON_FIELD,
    );
    let maybe_websocket_json = parse_single_document_field(
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
            parse_json_identity(
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

fn parse_json_identity(
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
    parse_identity_values(label, &session_value, &revision_value, validation_errors)
}

fn parse_identity_values(
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
        let Some((session_text, revision_text)) = parse_retained_marker_fields(line) else {
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
        let Some(identity) = parse_identity_values(
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

fn parse_retained_marker_fields(line: &str) -> Option<(&str, &str)> {
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
    let maybe_session =
        parse_single_document_field(validation_errors, label, document, PROJECTED_SESSION_FIELD);
    let maybe_revision =
        parse_single_document_field(validation_errors, label, document, PROJECTED_REVISION_FIELD);
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
        parse_identity_values(label, &session_value, &revision_value, validation_errors)
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

fn parse_single_document_field<'a>(
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
mod tests {
    use std::sync::{mpsc, Arc, Mutex};
    use std::thread;

    use bitaxe_api::OperatorSnapshotPublisher;

    use super::*;

    const SESSION: &str = "0123456789abcdef0011223344556677";
    const OTHER_SESSION: &str = "fedcba9876543210ffeeddccbbaa9988";
    const API_IDENTITY_SOURCE: &str =
        include_str!("../../../crates/bitaxe-api/src/operator_snapshot.rs");
    const PUBLICATION_SOURCE: &str =
        include_str!("../../../crates/bitaxe-api/src/operator_snapshot_publication.rs");
    const BOOT_EVIDENCE_SOURCE: &str =
        include_str!("../../../firmware/bitaxe/src/boot_evidence.rs");
    const RUNTIME_SNAPSHOT_SOURCE: &str =
        include_str!("../../../firmware/bitaxe/src/runtime_snapshot.rs");

    #[test]
    fn coherent_same_capture_projects_exact_redacted_fields() {
        // Arrange
        let json = identity_json(SESSION, 7);
        let projections = [
            JsonProjection {
                label: "api.md",
                json: &json,
            },
            JsonProjection {
                label: "websocket.md",
                json: &json,
            },
        ];
        let retained_log = marker(SESSION, 7);

        // Act
        let report = validate_operator_snapshot_evidence(&projections, &retained_log);
        let projection = redacted_document_projection(report.identities[0].1);

        // Assert
        assert!(report.validation_errors.is_empty(), "{report:#?}");
        assert_eq!(
            projection,
            format!("operator_snapshot_boot_session: {SESSION}\noperator_snapshot_revision: 7\n")
        );
    }

    #[test]
    fn later_capture_accepts_strictly_greater_revision() {
        // Arrange
        let first = identity_json(SESSION, 7);
        let second = identity_json(SESSION, 8);
        let projections = [
            JsonProjection {
                label: "api.md",
                json: &first,
            },
            JsonProjection {
                label: "websocket.md",
                json: &second,
            },
        ];
        let retained_log = format!("{}\n{}", marker(SESSION, 7), marker(SESSION, 8));

        // Act
        let report = validate_operator_snapshot_evidence(&projections, &retained_log);

        // Assert
        assert!(report.validation_errors.is_empty(), "{report:#?}");
    }

    #[test]
    fn malformed_or_incoherent_inputs_fail_with_stable_categories() {
        // Arrange
        let cases = [
            (
                "mixed boot",
                identity_json(SESSION, 1),
                identity_json(OTHER_SESSION, 2),
                format!("{}\n{}", marker(SESSION, 1), marker(OTHER_SESSION, 2)),
                "operator_snapshot_mixed_session",
            ),
            (
                "revision regression",
                identity_json(SESSION, 2),
                identity_json(SESSION, 1),
                format!("{}\n{}", marker(SESSION, 2), marker(SESSION, 1)),
                "operator_snapshot_revision_regression",
            ),
            (
                "missing marker",
                identity_json(SESSION, 1),
                identity_json(SESSION, 2),
                marker(SESSION, 1),
                "operator_snapshot_missing_marker",
            ),
            (
                "duplicate field",
                format!(
                    r#"{{"bootSession":"{SESSION}","bootSession":"{SESSION}","operatorSnapshotRevision":1}}"#
                ),
                identity_json(SESSION, 2),
                format!("{}\n{}", marker(SESSION, 1), marker(SESSION, 2)),
                "operator_snapshot_duplicate_field",
            ),
            (
                "malformed session",
                identity_json("ABCDEF", 1),
                identity_json(SESSION, 2),
                marker(SESSION, 2),
                "operator_snapshot_malformed_session",
            ),
            (
                "partial pair",
                format!(r#"{{"bootSession":"{SESSION}"}}"#),
                identity_json(SESSION, 2),
                marker(SESSION, 2),
                "operator_snapshot_missing_half",
            ),
            (
                "zero revision",
                identity_json(SESSION, 0),
                identity_json(SESSION, 2),
                marker(SESSION, 2),
                "operator_snapshot_malformed_revision",
            ),
            (
                "fixture session",
                identity_json(&"0".repeat(32), 1),
                identity_json(SESSION, 2),
                marker(SESSION, 2),
                "operator_snapshot_synthetic_identity",
            ),
            (
                "host checkout",
                identity_json("0123456789abcdef0123456789abcdef01234567", 1),
                identity_json(SESSION, 2),
                marker(SESSION, 2),
                "operator_snapshot_host_checkout_substitution",
            ),
        ];

        for (name, api, websocket, retained_log, expected) in cases {
            // Act
            let report = validate_operator_snapshot_evidence(
                &[
                    JsonProjection {
                        label: "api.md",
                        json: &api,
                    },
                    JsonProjection {
                        label: "websocket.md",
                        json: &websocket,
                    },
                ],
                &retained_log,
            );

            // Assert
            assert!(
                report
                    .validation_errors
                    .iter()
                    .any(|error| error.contains(expected)),
                "case {name} expected {expected}, got {report:#?}"
            );
        }
    }

    #[test]
    fn operator_evidence_documents_require_matching_redacted_projection() {
        // Arrange
        let api = evidence_document(SYSTEM_INFO_JSON_FIELD, SESSION, 4);
        let websocket = evidence_document(LIVE_WEBSOCKET_JSON_FIELD, SESSION, 5);
        let log = format!("{}\n{}", marker(SESSION, 4), marker(SESSION, 5));

        // Act
        let accepted = validate_operator_snapshot_documents(&api, &websocket, &log);
        let rejected = validate_operator_snapshot_documents(
            &api.replace(
                "operator_snapshot_revision: 4",
                "operator_snapshot_revision: 9",
            ),
            &websocket,
            &log,
        );

        // Assert
        assert!(accepted.is_empty(), "{accepted:#?}");
        assert!(rejected
            .iter()
            .any(|error| error.contains("operator_snapshot_projection_mismatch")));
    }

    #[derive(Clone, Copy, Debug, Eq, PartialEq)]
    enum NamedCandidate {
        HttpSystemInfoCapture1,
        LiveWebSocketCapture2,
    }

    #[derive(Clone, Debug)]
    struct EvidencePublication {
        candidate: NamedCandidate,
        identity: OperatorSnapshotIdentity,
        payload: Vec<u8>,
        retained_marker: String,
        retained_runtime_health: String,
    }

    #[derive(Clone, Debug, Eq, PartialEq)]
    enum IssuedPayload {
        Http(Vec<u8>),
        LiveWebSocket(Vec<u8>),
    }

    fn complete_evidence_publication(
        candidate: NamedCandidate,
        identity: OperatorSnapshotIdentity,
    ) -> EvidencePublication {
        let payload = serde_json::to_vec(&serde_json::json!({
            "bootSession": identity.boot_session(),
            "operatorSnapshotRevision": identity.revision(),
            "runtimeHealth": {
                "checkpointHealth": "healthy",
            },
        }))
        .expect("identity-bearing evidence payload must serialize");
        EvidencePublication {
            candidate,
            identity,
            payload,
            retained_marker: identity.retained_marker(),
            retained_runtime_health: format!(
                "runtime_health boot_session={} operator_snapshot_revision={} checkpoint_health=healthy redacted=true",
                identity.boot_session(),
                identity.revision()
            ),
        }
    }

    fn issue_http_response(
        issued: &Mutex<Vec<IssuedPayload>>,
        publication: EvidencePublication,
    ) -> Result<(), &'static str> {
        assert_eq!(
            publication.candidate,
            NamedCandidate::HttpSystemInfoCapture1
        );
        assert!(publication.payload.windows(32).any(|window| {
            window == publication.identity.boot_session().to_string().as_bytes()
        }));
        issued
            .lock()
            .expect("issued history must be available")
            .push(IssuedPayload::Http(publication.payload));
        Ok(())
    }

    fn issue_live_websocket_frame(
        issued: &Mutex<Vec<IssuedPayload>>,
        publication: EvidencePublication,
    ) -> Result<(), &'static str> {
        assert_eq!(publication.candidate, NamedCandidate::LiveWebSocketCapture2);
        assert!(publication.payload.windows(32).any(|window| {
            window == publication.identity.boot_session().to_string().as_bytes()
        }));
        issued
            .lock()
            .expect("issued history must be available")
            .push(IssuedPayload::LiveWebSocket(publication.payload));
        Ok(())
    }

    #[test]
    fn operator_snapshot_publication_reverse_completion_preserves_direct_chronology() {
        // Arrange
        let publisher = Arc::new(OperatorSnapshotPublisher::new());
        let retained = Arc::new(Mutex::new(Vec::<(String, String)>::new()));
        let issued = Arc::new(Mutex::new(Vec::<IssuedPayload>::new()));
        let completed = Arc::new(Mutex::new(Vec::<NamedCandidate>::new()));
        let (capture1_entered_tx, capture1_entered_rx) = mpsc::channel();
        let (release_capture1_tx, release_capture1_rx) = mpsc::channel();
        let session = SESSION
            .parse::<BootSessionId>()
            .expect("test boot session must be valid");
        let capture1_publisher = Arc::clone(&publisher);
        let capture1_retained = Arc::clone(&retained);
        let capture1_issued = Arc::clone(&issued);
        let capture1_completed = Arc::clone(&completed);

        // Act
        let capture1 = thread::spawn(move || {
            capture1_publisher.publish(
                session,
                || {
                    capture1_entered_tx
                        .send(())
                        .expect("capture 1 collection entry must be observable");
                    release_capture1_rx
                        .recv()
                        .expect("capture 1 release must arrive");
                    NamedCandidate::HttpSystemInfoCapture1
                },
                |candidate, identity| {
                    capture1_completed
                        .lock()
                        .expect("completion history must be available")
                        .push(candidate);
                    complete_evidence_publication(candidate, identity)
                },
                |publication| {
                    capture1_retained
                        .lock()
                        .expect("retained history must be available")
                        .push((
                            publication.retained_marker.clone(),
                            publication.retained_runtime_health.clone(),
                        ));
                    Ok::<(), &'static str>(())
                },
                |publication| issue_http_response(&capture1_issued, publication),
            )
        });
        capture1_entered_rx
            .recv()
            .expect("capture 1 must enter collection");

        let capture2_publisher = Arc::clone(&publisher);
        let capture2_retained = Arc::clone(&retained);
        let capture2_issued = Arc::clone(&issued);
        let capture2_completed = Arc::clone(&completed);
        let capture2 = thread::spawn(move || {
            capture2_publisher.publish(
                session,
                || NamedCandidate::LiveWebSocketCapture2,
                |candidate, identity| {
                    capture2_completed
                        .lock()
                        .expect("completion history must be available")
                        .push(candidate);
                    complete_evidence_publication(candidate, identity)
                },
                |publication| {
                    capture2_retained
                        .lock()
                        .expect("retained history must be available")
                        .push((
                            publication.retained_marker.clone(),
                            publication.retained_runtime_health.clone(),
                        ));
                    Ok::<(), &'static str>(())
                },
                |publication| issue_live_websocket_frame(&capture2_issued, publication),
            )
        });
        capture2
            .join()
            .expect("capture 2 thread must not panic")
            .expect("capture 2 publication must succeed");
        release_capture1_tx
            .send(())
            .expect("capture 1 collection must be releasable");
        capture1
            .join()
            .expect("capture 1 thread must not panic")
            .expect("capture 1 publication must succeed");

        // Assert
        assert_eq!(
            *completed
                .lock()
                .expect("completion history must be available"),
            [
                NamedCandidate::LiveWebSocketCapture2,
                NamedCandidate::HttpSystemInfoCapture1,
            ]
        );
        let retained = retained.lock().expect("retained history must be available");
        let retained_identities = retained
            .iter()
            .map(|(marker, _health)| {
                let (session, revision) = parse_retained_marker_fields(marker)
                    .expect("retained marker must remain exact");
                (
                    session.to_owned(),
                    revision
                        .parse::<u64>()
                        .expect("retained revision must be numeric"),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            retained_identities
                .iter()
                .map(|(_, revision)| *revision)
                .collect::<Vec<_>>(),
            [1, 2]
        );
        for ((session, revision), (_, runtime_health)) in retained_identities.iter().zip(&*retained)
        {
            assert!(runtime_health.contains(&format!("boot_session={session}")));
            assert!(runtime_health.contains(&format!("operator_snapshot_revision={revision}")));
            assert!(runtime_health.contains("checkpoint_health=healthy"));
        }

        let issued = issued.lock().expect("issued history must be available");
        assert!(matches!(issued[0], IssuedPayload::LiveWebSocket(_)));
        assert!(matches!(issued[1], IssuedPayload::Http(_)));
        let issued_json = issued
            .iter()
            .map(|payload| match payload {
                IssuedPayload::Http(bytes) | IssuedPayload::LiveWebSocket(bytes) => {
                    String::from_utf8(bytes.clone()).expect("issued JSON must be UTF-8")
                }
            })
            .collect::<Vec<_>>();
        let issued_revisions = issued_json
            .iter()
            .map(|json| {
                serde_json::from_str::<serde_json::Value>(json).expect("issued JSON must parse")
                    ["operatorSnapshotRevision"]
                    .as_u64()
                    .expect("issued revision must be numeric")
            })
            .collect::<Vec<_>>();
        assert_eq!(issued_revisions, [1, 2]);
        let retained_log = retained
            .iter()
            .flat_map(|(marker, health)| [marker.as_str(), health.as_str()])
            .collect::<Vec<_>>()
            .join("\n");
        let projections = issued_json
            .iter()
            .enumerate()
            .map(|(index, json)| JsonProjection {
                label: if index == 0 { "websocket.md" } else { "api.md" },
                json,
            })
            .collect::<Vec<_>>();
        let report = validate_operator_snapshot_evidence(&projections, &retained_log);
        assert!(report.validation_errors.is_empty(), "{report:#?}");
    }

    #[test]
    fn phase34_operator_snapshot_runtime_source_guard() {
        // Arrange
        let publication = source_between(
            RUNTIME_SNAPSHOT_SOURCE,
            "fn publish_operator_snapshot",
            "fn collect_operator_snapshot_candidate",
        );
        let collection = source_between(
            RUNTIME_SNAPSHOT_SOURCE,
            "fn collect_operator_snapshot_candidate",
            "fn runtime_projection_for_api_views",
        );
        let retention = source_between(
            RUNTIME_SNAPSHOT_SOURCE,
            "fn retain_completed_operator_snapshot",
            "fn mutate_command_visible_state_with_result",
        );

        // Act / Assert
        assert_eq!(BOOT_EVIDENCE_SOURCE.matches("esp_random()").count(), 4);
        assert_eq!(
            BOOT_EVIDENCE_SOURCE.matches("static BOOT_SESSION:").count(),
            1
        );
        assert!(BOOT_EVIDENCE_SOURCE.contains("operator_snapshot_boot_session"));
        assert!(BOOT_EVIDENCE_SOURCE.contains("BootSessionId::from_words(boot_session().0)"));

        assert_eq!(
            RUNTIME_SNAPSHOT_SOURCE
                .matches("static OPERATOR_SNAPSHOT_PUBLISHER:")
                .count(),
            1
        );
        assert_eq!(PUBLICATION_SOURCE.matches(".next_identity(").count(), 1);
        assert_eq!(
            RUNTIME_SNAPSHOT_SOURCE
                .matches("snapshot.operator_snapshot_identity = operator_snapshot_identity")
                .count(),
            1
        );
        assert!(RUNTIME_SNAPSHOT_SOURCE
            .contains("static OPERATOR_SNAPSHOT_PUBLISHER: OnceLock<OperatorSnapshotPublisher>"));
        let collect_adapter = publication
            .find("|| collect_operator_snapshot_candidate(drain_sample_marker)")
            .expect("unnumbered candidate collection adapter");
        let complete_adapter = publication
            .find("|candidate, identity|")
            .expect("identity completion adapter");
        let retain_adapter = publication
            .find("retain_completed_operator_snapshot(publication)")
            .expect("retained chronology adapter");
        let issue_adapter = publication
            .find("|publication| issue(publication.output)")
            .expect("external issuance adapter");
        assert!(collect_adapter < complete_adapter);
        assert!(complete_adapter < retain_adapter && retain_adapter < issue_adapter);
        assert!(!collection.contains("OperatorSnapshotIdentity"));
        assert!(!collection.contains("next_identity"));
        assert!(retention.contains("append_runtime_log_line(&publication.retained_marker)"));
        assert!(retention.contains("append_runtime_log_line(&publication.retained_runtime_health)"));
        assert!(
            API_IDENTITY_SOURCE.contains("operator_snapshot session={} revision={} redacted=true")
        );
        assert!(PUBLICATION_SOURCE
            .contains("reverse_collection_completion_publishes_direct_revisions_in_order"));

        for forbidden in [
            "esp_random",
            "SystemTime",
            "firmware_commit",
            "app_elf_sha256",
            "mac_addr",
            "fixture_only",
        ] {
            assert!(
                !publication.contains(forbidden),
                "publication contains forbidden fallback {forbidden}"
            );
        }
    }

    fn identity_json(session: &str, revision: u64) -> String {
        serde_json::json!({
            "bootSession": session,
            "operatorSnapshotRevision": revision,
        })
        .to_string()
    }

    fn marker(session: &str, revision: u64) -> String {
        format!("operator_snapshot session={session} revision={revision} redacted=true")
    }

    fn evidence_document(field: &str, session: &str, revision: u64) -> String {
        format!(
            "{field}: {}\noperator_snapshot_boot_session: {session}\noperator_snapshot_revision: {revision}\n",
            identity_json(session, revision)
        )
    }

    fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
        let start_index = source.find(start).expect("start marker should exist");
        let tail = &source[start_index..];
        let end_index = tail.find(end).expect("end marker should exist");
        &tail[..end_index]
    }
}
