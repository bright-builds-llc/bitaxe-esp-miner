use super::super::capture::{
    classify_candidate_files, classify_private_capture, inspect_candidate_file,
    write_synthetic_capture, BrokerCaptureDocument, CaptureObservationSource,
    Phase36PrivateCaptureBundle, RuntimeIdentityCaptureDocuments, SubstantiveCaptureDocuments,
    PHASE36_PRIVATE_CAPTURE_SCHEMA,
};
use super::runtime_identity;
use crate::phase36_broker::{
    Phase36AllowedOperation, Phase36LedgerRecord, Phase36LedgerState, Phase36LedgerTransition,
};
use camino::Utf8PathBuf;
use std::fs;
use std::os::unix::fs::{MetadataExt, PermissionsExt};

const SUBSTANCE: &str = include_str!("../../../fixtures/phase36/substance-eligible.json");

fn digest(seed: char) -> String {
    std::iter::repeat_n(seed, 64).collect()
}

fn substantive_documents() -> SubstantiveCaptureDocuments {
    let value: serde_json::Value =
        serde_json::from_str(SUBSTANCE).expect("substance fixture must parse");
    let json = serde_json::to_string(&value).expect("substance fixture must serialize");
    let session = value["bootSession"]
        .as_str()
        .expect("fixture session must be textual");
    let revision = value["operatorSnapshotRevision"]
        .as_u64()
        .expect("fixture revision must be numeric");
    let marker = format!("operator_snapshot session={session} revision={revision} redacted=true");
    SubstantiveCaptureDocuments {
        system_info_document: format!(
            "system_info_json: {json}\noperator_snapshot_boot_session: {session}\noperator_snapshot_revision: {revision}\n"
        ),
        websocket_document: format!(
            "live_websocket_json: {json}\noperator_snapshot_boot_session: {session}\noperator_snapshot_revision: {revision}\n"
        ),
        retained_document: format!("{marker}\nsubstantive_snapshot_json: {json}\n"),
    }
}

fn broker_document() -> BrokerCaptureDocument {
    let mut state = Phase36LedgerState::start(100).expect("interval should start");
    let mut records = Vec::new();
    let mut millis = 100;
    for operation in Phase36AllowedOperation::SUCCESS_ORDER {
        for transition in [
            Phase36LedgerTransition::Authorized,
            Phase36LedgerTransition::Invoked,
            Phase36LedgerTransition::Completed,
            Phase36LedgerTransition::Closed,
        ] {
            millis += 1;
            let record = Phase36LedgerRecord::next(&state, operation, transition, millis)
                .expect("eligible record should build");
            state.apply(&record).expect("eligible record should reduce");
            records.push(record);
        }
    }
    BrokerCaptureDocument {
        observation_source: CaptureObservationSource::IndependentBrokerLedger,
        capability_digest: digest('c'),
        package_digest: digest('7'),
        same_physical_device_observed: true,
        interval_start_millis: 100,
        interval_end_millis: millis + 1,
        records,
    }
}

fn bundle() -> Phase36PrivateCaptureBundle {
    let mut identity = runtime_identity::documents();
    for document in [&mut identity.ledger, &mut identity.private_result] {
        *document = document.replace("boot-b", "0123456789abcdef0011223344556677");
    }
    Phase36PrivateCaptureBundle {
        schema_version: PHASE36_PRIVATE_CAPTURE_SCHEMA.to_owned(),
        board_category: "205".to_owned(),
        substantive: substantive_documents(),
        runtime_identity: RuntimeIdentityCaptureDocuments {
            exact_package_document: identity.package,
            request_document: identity.request,
            event_ledger_document: identity.ledger,
            private_result_document: identity.private_result,
            public_projection_document: identity.public_projection,
            hardware_observation_document: None,
        },
        broker: broker_document(),
    }
}

#[test]
fn phase36_capture_derives_only_validated_shareable_candidate_facts() {
    // Arrange
    let bundle = bundle();
    let private_bytes = serde_json::to_vec(&bundle).expect("bundle should serialize");

    // Act
    let candidate =
        classify_private_capture(&private_bytes).expect("eligible capture should classify");

    // Assert
    assert_eq!(candidate.board_category, "205");
    assert_eq!(candidate.private_capture_digest.len(), 64);
    assert_eq!(
        candidate.runtime_identity.exact_package.package_digest,
        digest('7')
    );
    assert_eq!(candidate.effect_interval.effect_count, 8);
}

#[test]
fn phase36_capture_rejects_runtime_boot_outside_snapshot_join() {
    // Arrange
    let mut bundle = bundle();
    bundle.substantive.system_info_document = bundle.substantive.system_info_document.replace(
        "0123456789abcdef0011223344556677",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    bundle.substantive.websocket_document = bundle.substantive.websocket_document.replace(
        "0123456789abcdef0011223344556677",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    bundle.substantive.retained_document = bundle.substantive.retained_document.replace(
        "0123456789abcdef0011223344556677",
        "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
    );
    let private_bytes = serde_json::to_vec(&bundle).expect("bundle should serialize");

    // Act
    let result = classify_private_capture(&private_bytes);

    // Assert
    assert!(result.is_err());
}

#[test]
fn phase36_capture_rejects_supervisor_authored_effect_claim() {
    // Arrange
    let mut bundle = bundle();
    bundle.broker.observation_source = CaptureObservationSource::SupervisorAttestation;
    let private_bytes = serde_json::to_vec(&bundle).expect("bundle should serialize");

    // Act
    let result = classify_private_capture(&private_bytes);

    // Assert
    assert!(result.is_err());
}

#[test]
fn phase36_capture_rejects_package_not_bound_to_broker() {
    // Arrange
    let mut bundle = bundle();
    bundle.broker.package_digest = digest('a');
    let private_bytes = serde_json::to_vec(&bundle).expect("bundle should serialize");

    // Act
    let result = classify_private_capture(&private_bytes);

    // Assert
    assert!(result.is_err());
}

#[test]
fn phase36_capture_binds_every_bundle_and_broker_ledger_field() {
    // Arrange
    type Mutation = (&'static str, fn(&mut serde_json::Value));
    let original = serde_json::to_value(bundle()).expect("bundle should become JSON");
    let mutations: [Mutation; 17] = [
        ("schema", |value| {
            value["schema_version"] = serde_json::json!("phase36-private-capture-v0");
        }),
        ("board", |value| {
            value["board_category"] = serde_json::json!("204");
        }),
        ("system info", |value| {
            value["substantive"]["system_info_document"] = serde_json::json!("invalid");
        }),
        ("websocket", |value| {
            value["substantive"]["websocket_document"] = serde_json::json!("invalid");
        }),
        ("retained", |value| {
            value["substantive"]["retained_document"] = serde_json::json!("invalid");
        }),
        ("runtime package", |value| {
            value["runtime_identity"]["exact_package_document"] = serde_json::json!("invalid");
        }),
        ("runtime request", |value| {
            value["runtime_identity"]["request_document"] = serde_json::json!("invalid");
        }),
        ("runtime ledger", |value| {
            value["runtime_identity"]["event_ledger_document"] = serde_json::json!("invalid");
        }),
        ("runtime result", |value| {
            value["runtime_identity"]["private_result_document"] = serde_json::json!("invalid");
        }),
        ("runtime projection", |value| {
            value["runtime_identity"]["public_projection_document"] = serde_json::json!("invalid");
        }),
        ("effect source", |value| {
            value["broker"]["observation_source"] = serde_json::json!("supervisor_attestation");
        }),
        ("capability", |value| {
            value["broker"]["capability_digest"] = serde_json::json!("invalid");
        }),
        ("package join", |value| {
            value["broker"]["package_digest"] = serde_json::json!(digest('a'));
        }),
        ("physical device", |value| {
            value["broker"]["same_physical_device_observed"] = serde_json::json!(false);
        }),
        ("interval start", |value| {
            value["broker"]["interval_start_millis"] = serde_json::json!(101);
        }),
        ("interval end", |value| {
            value["broker"]["interval_end_millis"] = serde_json::json!(132);
        }),
        ("ledger record", |value| {
            value["broker"]["records"][0]["record_digest"] = serde_json::json!(digest('f'));
        }),
    ];

    // Act and Assert
    for (name, mutate) in mutations {
        let mut changed = original.clone();
        mutate(&mut changed);
        let bytes = serde_json::to_vec(&changed).expect("mutated bundle should serialize");
        assert!(
            classify_private_capture(&bytes).is_err(),
            "mutation {name} was accepted"
        );
    }
}

#[test]
fn phase36_capture_keeps_private_and_candidate_bytes_stable_across_offline_classification() {
    // Arrange
    let root = Utf8PathBuf::from(format!(
        "{}/phase36-capture-{}",
        std::env::temp_dir().display(),
        std::process::id()
    ));
    if root.exists() {
        fs::remove_dir_all(&root).expect("stale test root should be removable");
    }
    fs::create_dir(&root).expect("test root should be created");
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
        .expect("test root should be private");
    let private = root.join("private.json");
    let candidate = root.join("candidate.json");
    let classification = root.join("classification.json");
    write_synthetic_capture(&private, &candidate, &digest('c'))
        .expect("synthetic capture should derive");
    let private_before = fs::read(&private).expect("private input should be readable");
    let candidate_before = fs::read(&candidate).expect("candidate should be readable");

    // Act
    let inspection = inspect_candidate_file(&candidate).expect("candidate should inspect");
    let output = classify_candidate_files(&private, &candidate, &classification)
        .expect("candidate should classify");

    // Assert
    assert_eq!(inspection.category, "candidate_eligible");
    assert_eq!(output.category, "classification_complete");
    assert_eq!(
        fs::read(&private).expect("private input should remain readable"),
        private_before
    );
    assert_eq!(
        fs::read(&candidate).expect("candidate should remain readable"),
        candidate_before
    );
    assert_eq!(
        fs::metadata(&classification)
            .expect("classification should exist")
            .mode()
            & 0o777,
        0o600
    );
    fs::remove_dir_all(root).expect("test root should be removable");
}
