mod source_guard;

use std::sync::{mpsc, Arc, Mutex};
use std::thread;

#[path = "../../../../firmware/bitaxe/src/operator_snapshot_publication.rs"]
mod operator_snapshot_publication_shell;

use operator_snapshot_publication_shell::OperatorSnapshotPublisher;

use super::*;

const SESSION: &str = "0123456789abcdef0011223344556677";
const OTHER_SESSION: &str = "fedcba9876543210ffeeddccbbaa9988";
const API_IDENTITY_SOURCE: &str =
    include_str!("../../../../crates/bitaxe-api/src/operator_snapshot.rs");
const PUBLICATION_SOURCE: &str =
    include_str!("../../../../firmware/bitaxe/src/operator_snapshot_publication.rs");
const BOOT_EVIDENCE_SOURCE: &str = include_str!("../../../../firmware/bitaxe/src/boot_evidence.rs");
const RUNTIME_SNAPSHOT_SOURCE: &str =
    include_str!("../../../../firmware/bitaxe/src/runtime_snapshot.rs");
const OPERATOR_SNAPSHOT_RETENTION_SOURCE: &str =
    include_str!("../../../../firmware/bitaxe/src/operator_snapshot_retention.rs");
const LOG_BUFFER_SOURCE: &str = include_str!("../../../../firmware/bitaxe/src/log_buffer.rs");

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
    assert!(publication
        .payload
        .windows(32)
        .any(|window| { window == publication.identity.boot_session().to_string().as_bytes() }));
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
    assert!(publication
        .payload
        .windows(32)
        .any(|window| { window == publication.identity.boot_session().to_string().as_bytes() }));
    issued
        .lock()
        .expect("issued history must be available")
        .push(IssuedPayload::LiveWebSocket(publication.payload));
    Ok(())
}

#[test]
fn operator_snapshot_publication_reverse_completion_preserves_direct_chronology() {
    // Arrange
    let harness = PublicationRaceHarness::new();

    // Act
    let outcome = harness.run_reverse_completion();

    // Assert
    assert_reverse_completion_chronology(&outcome);
}

struct PublicationRaceHarness {
    publisher: Arc<OperatorSnapshotPublisher>,
    retained: Arc<Mutex<Vec<(String, String)>>>,
    issued: Arc<Mutex<Vec<IssuedPayload>>>,
    completed: Arc<Mutex<Vec<NamedCandidate>>>,
    session: BootSessionId,
}

impl PublicationRaceHarness {
    fn new() -> Self {
        Self {
            publisher: Arc::new(OperatorSnapshotPublisher::new()),
            retained: Arc::new(Mutex::new(Vec::new())),
            issued: Arc::new(Mutex::new(Vec::new())),
            completed: Arc::new(Mutex::new(Vec::new())),
            session: SESSION
                .parse::<BootSessionId>()
                .expect("test boot session must be valid"),
        }
    }

    fn run_reverse_completion(self) -> PublicationRaceOutcome {
        let (capture1_entered_tx, capture1_entered_rx) = mpsc::channel();
        let (release_capture1_tx, release_capture1_rx) = mpsc::channel();
        let capture1 = self.spawn_capture1(capture1_entered_tx, release_capture1_rx);
        capture1_entered_rx
            .recv()
            .expect("capture 1 must enter collection");
        let capture2 = self.spawn_capture2();
        capture2.join().expect("capture 2 thread must not panic");
        release_capture1_tx
            .send(())
            .expect("capture 1 collection must be releasable");
        capture1.join().expect("capture 1 thread must not panic");
        let completed = self
            .completed
            .lock()
            .expect("completion history must be available")
            .clone();
        let retained = self
            .retained
            .lock()
            .expect("retained history must be available")
            .clone();
        let issued = self
            .issued
            .lock()
            .expect("issued history must be available")
            .clone();
        PublicationRaceOutcome {
            completed,
            retained,
            issued,
        }
    }

    fn spawn_capture1(
        &self,
        entered: mpsc::Sender<()>,
        release: mpsc::Receiver<()>,
    ) -> thread::JoinHandle<()> {
        let publisher = Arc::clone(&self.publisher);
        let retained = Arc::clone(&self.retained);
        let issued = Arc::clone(&self.issued);
        let completed = Arc::clone(&self.completed);
        let session = self.session;
        thread::spawn(move || {
            publisher
                .publish(
                    session,
                    || {
                        entered
                            .send(())
                            .expect("capture 1 collection entry must be observable");
                        release.recv().expect("capture 1 release must arrive");
                        NamedCandidate::HttpSystemInfoCapture1
                    },
                    |candidate, identity| {
                        completed
                            .lock()
                            .expect("completion history must be available")
                            .push(candidate);
                        complete_evidence_publication(candidate, identity)
                    },
                    |publication| {
                        retained
                            .lock()
                            .expect("retained history must be available")
                            .push((
                                publication.retained_marker.clone(),
                                publication.retained_runtime_health.clone(),
                            ));
                        Ok::<(), &'static str>(())
                    },
                    |publication| issue_http_response(&issued, publication),
                )
                .expect("capture 1 publication must succeed");
        })
    }

    fn spawn_capture2(&self) -> thread::JoinHandle<()> {
        let publisher = Arc::clone(&self.publisher);
        let retained = Arc::clone(&self.retained);
        let issued = Arc::clone(&self.issued);
        let completed = Arc::clone(&self.completed);
        let session = self.session;
        thread::spawn(move || {
            publisher
                .publish(
                    session,
                    || NamedCandidate::LiveWebSocketCapture2,
                    |candidate, identity| {
                        completed
                            .lock()
                            .expect("completion history must be available")
                            .push(candidate);
                        complete_evidence_publication(candidate, identity)
                    },
                    |publication| {
                        retained
                            .lock()
                            .expect("retained history must be available")
                            .push((
                                publication.retained_marker.clone(),
                                publication.retained_runtime_health.clone(),
                            ));
                        Ok::<(), &'static str>(())
                    },
                    |publication| issue_live_websocket_frame(&issued, publication),
                )
                .expect("capture 2 publication must succeed");
        })
    }
}

struct PublicationRaceOutcome {
    completed: Vec<NamedCandidate>,
    retained: Vec<(String, String)>,
    issued: Vec<IssuedPayload>,
}

fn assert_reverse_completion_chronology(outcome: &PublicationRaceOutcome) {
    assert_eq!(
        outcome.completed,
        [
            NamedCandidate::LiveWebSocketCapture2,
            NamedCandidate::HttpSystemInfoCapture1,
        ]
    );
    let retained_identities = outcome
        .retained
        .iter()
        .map(|(marker, _health)| {
            let (session, revision) = maybe_parse_retained_marker_fields(marker)
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
    for ((session, revision), (_, runtime_health)) in
        retained_identities.iter().zip(&outcome.retained)
    {
        assert!(runtime_health.contains(&format!("boot_session={session}")));
        assert!(runtime_health.contains(&format!("operator_snapshot_revision={revision}")));
        assert!(runtime_health.contains("checkpoint_health=healthy"));
    }

    assert!(matches!(outcome.issued[0], IssuedPayload::LiveWebSocket(_)));
    assert!(matches!(outcome.issued[1], IssuedPayload::Http(_)));
    let issued_json = outcome
        .issued
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
    let retained_log = outcome
        .retained
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
    source_guard::verify_runtime_sources();
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
