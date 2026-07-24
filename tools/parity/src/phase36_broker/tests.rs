use super::*;
use crate::phase36_evidence::effects::ValidatedIndependentEffectInterval;

fn digest(seed: char) -> String {
    std::iter::repeat_n(seed, 64).collect()
}

fn capability_scope() -> Phase36CapabilityScope {
    Phase36CapabilityScope::new(
        32,
        digest('1'),
        digest('2'),
        digest('3'),
        digest('4'),
        digest('5'),
    )
    .expect("fixture scope should be valid")
}

fn capability_and_presentation() -> (Phase36BrokerCapability, Phase36CapabilityPresentation) {
    let capability =
        Phase36BrokerCapability::issue(capability_scope(), digest('6'), 10_000, 20_000)
            .expect("fixture capability should issue");
    let presentation = capability.presentation();
    (capability, presentation)
}

#[test]
fn phase36_broker_capability_is_single_use() {
    // Arrange
    let (capability, presentation) = capability_and_presentation();
    let mut guard = Phase36CapabilityGuard::new(capability);

    // Act
    let first = guard.admit(&presentation, 11_000);
    let replay = guard.admit(&presentation, 11_001);

    // Assert
    assert!(first.is_ok());
    assert_eq!(replay, Err(Phase36CapabilityError::Replay));
}

#[test]
fn phase36_broker_capability_rejects_expiry() {
    // Arrange
    let (capability, presentation) = capability_and_presentation();
    let mut guard = Phase36CapabilityGuard::new(capability);

    // Act
    let result = guard.admit(&presentation, 20_001);

    // Assert
    assert_eq!(result, Err(Phase36CapabilityError::Expired));
}

#[test]
fn phase36_broker_capability_rejects_wrong_peer() {
    // Arrange
    let (capability, mut presentation) = capability_and_presentation();
    presentation.peer_identity_digest = digest('a');
    let mut guard = Phase36CapabilityGuard::new(capability);

    // Act
    let result = guard.admit(&presentation, 11_000);

    // Assert
    assert_eq!(result, Err(Phase36CapabilityError::WrongPeer));
}

#[test]
fn phase36_broker_capability_rejects_wrong_attempt() {
    // Arrange
    let (capability, mut presentation) = capability_and_presentation();
    presentation.attempt_ordinal += 1;
    let mut guard = Phase36CapabilityGuard::new(capability);

    // Act
    let result = guard.admit(&presentation, 11_000);

    // Assert
    assert_eq!(result, Err(Phase36CapabilityError::WrongAttempt));
}

#[test]
fn phase36_broker_capability_rejects_wrong_source() {
    // Arrange
    let (capability, mut presentation) = capability_and_presentation();
    presentation.source_identity_digest = digest('a');
    let mut guard = Phase36CapabilityGuard::new(capability);

    // Act
    let result = guard.admit(&presentation, 11_000);

    // Assert
    assert_eq!(result, Err(Phase36CapabilityError::WrongSource));
}

#[test]
fn phase36_broker_capability_rejects_wrong_evaluator() {
    // Arrange
    let (capability, mut presentation) = capability_and_presentation();
    presentation.evaluator_identity_digest = digest('a');
    let mut guard = Phase36CapabilityGuard::new(capability);

    // Act
    let result = guard.admit(&presentation, 11_000);

    // Assert
    assert_eq!(result, Err(Phase36CapabilityError::WrongEvaluator));
}

#[test]
fn phase36_broker_capability_rejects_wrong_package() {
    // Arrange
    let (capability, mut presentation) = capability_and_presentation();
    presentation.package_identity_digest = digest('a');
    let mut guard = Phase36CapabilityGuard::new(capability);

    // Act
    let result = guard.admit(&presentation, 11_000);

    // Assert
    assert_eq!(result, Err(Phase36CapabilityError::WrongPackage));
}

#[test]
fn phase36_broker_capability_rejects_wrong_protected_root() {
    // Arrange
    let (capability, mut presentation) = capability_and_presentation();
    presentation.protected_root_identity_digest = digest('a');
    let mut guard = Phase36CapabilityGuard::new(capability);

    // Act
    let result = guard.admit(&presentation, 11_000);

    // Assert
    assert_eq!(result, Err(Phase36CapabilityError::WrongProtectedRoot));
}

fn apply_successful_operation(
    state: &mut Phase36LedgerState,
    operation: Phase36AllowedOperation,
    timestamp: &mut u64,
) {
    for transition in [
        Phase36LedgerTransition::Authorized,
        Phase36LedgerTransition::Invoked,
        Phase36LedgerTransition::Completed,
        Phase36LedgerTransition::Closed,
    ] {
        *timestamp += 1;
        let record = Phase36LedgerRecord::next(state, operation, transition, *timestamp)
            .expect("fixture record should construct");
        state
            .apply(&record)
            .expect("fixture transition should apply");
    }
}

fn successful_state() -> (Phase36LedgerState, u64) {
    let mut state = Phase36LedgerState::start(1_000).expect("fixture interval should start");
    let mut timestamp = 1_000;
    for operation in [
        Phase36AllowedOperation::ExactPackageAdmission,
        Phase36AllowedOperation::Board205DetectorProbe,
        Phase36AllowedOperation::ExactPackageFlash,
        Phase36AllowedOperation::PassiveSerialObservation,
        Phase36AllowedOperation::ReadOnlySystemInfo,
        Phase36AllowedOperation::ReadOnlyWebSocket,
        Phase36AllowedOperation::ReadOnlyRetainedFacts,
        Phase36AllowedOperation::Cleanup,
    ] {
        apply_successful_operation(&mut state, operation, &mut timestamp);
    }
    (state, timestamp)
}

#[test]
fn phase36_broker_ledger_seals_complete_passive_interval() {
    // Arrange
    let (state, timestamp) = successful_state();

    // Act
    let interval = state.seal(timestamp + 1);

    // Assert
    let interval = interval.expect("complete interval should seal");
    assert_eq!(interval.effect_count(), 8);
    assert_eq!(interval.first_failure(), None);
}

#[test]
fn phase36_broker_interval_constructs_validated_independent_effect_interval() {
    // Arrange
    let (state, timestamp) = successful_state();
    let interval = state
        .seal(timestamp + 1)
        .expect("complete interval should seal");

    // Act
    let validated = ValidatedIndependentEffectInterval::from(&interval);

    // Assert
    assert_eq!(validated.start_millis, interval.start_millis());
    assert_eq!(validated.end_millis, interval.end_millis());
    assert_eq!(validated.effect_count, interval.effect_count());
    assert_eq!(validated.ledger_digest, interval.ledger_digest());
}

#[test]
fn phase36_broker_ledger_rejects_missing_record() {
    // Arrange
    let mut state = Phase36LedgerState::start(1_000).expect("fixture interval should start");
    let authorized = Phase36LedgerRecord::next(
        &state,
        Phase36AllowedOperation::ExactPackageAdmission,
        Phase36LedgerTransition::Authorized,
        1_001,
    )
    .expect("authorization should construct");
    state
        .apply(&authorized)
        .expect("authorization should apply");
    let completed = Phase36LedgerRecord::next(
        &state,
        Phase36AllowedOperation::ExactPackageAdmission,
        Phase36LedgerTransition::Completed,
        1_002,
    );

    // Act and Assert
    assert_eq!(completed, Err(Phase36LedgerError::OutOfOrder));
}

#[test]
fn phase36_broker_ledger_rejects_duplicate_record() {
    // Arrange
    let mut state = Phase36LedgerState::start(1_000).expect("fixture interval should start");
    let record = Phase36LedgerRecord::next(
        &state,
        Phase36AllowedOperation::ExactPackageAdmission,
        Phase36LedgerTransition::Authorized,
        1_001,
    )
    .expect("authorization should construct");
    state.apply(&record).expect("authorization should apply");

    // Act
    let result = state.apply(&record);

    // Assert
    assert_eq!(result, Err(Phase36LedgerError::Duplicate));
}

#[test]
fn phase36_broker_ledger_rejects_reordered_record() {
    // Arrange
    let state = Phase36LedgerState::start(1_000).expect("fixture interval should start");
    let record = Phase36LedgerRecord::next(
        &state,
        Phase36AllowedOperation::Board205DetectorProbe,
        Phase36LedgerTransition::Authorized,
        1_001,
    );

    // Act and Assert
    assert_eq!(record, Err(Phase36LedgerError::OutOfOrder));
}

#[test]
fn phase36_broker_ledger_rejects_unknown_operation_document() {
    // Arrange
    let document = r#"{
        "sequence":1,
        "effect_id":1,
        "operation":"active_control",
        "transition":{"status":"authorized"},
        "monotonic_millis":1001,
        "previous_digest":"0000000000000000000000000000000000000000000000000000000000000000",
        "record_digest":"1111111111111111111111111111111111111111111111111111111111111111"
    }"#;

    // Act
    let result = serde_json::from_str::<Phase36LedgerRecord>(document);

    // Assert
    assert!(result.is_err());
}

#[test]
fn phase36_broker_ledger_rejects_unclosed_interval() {
    // Arrange
    let mut state = Phase36LedgerState::start(1_000).expect("fixture interval should start");
    let mut timestamp = 1_000;
    for transition in [
        Phase36LedgerTransition::Authorized,
        Phase36LedgerTransition::Invoked,
        Phase36LedgerTransition::Completed,
    ] {
        timestamp += 1;
        let record = Phase36LedgerRecord::next(
            &state,
            Phase36AllowedOperation::ExactPackageAdmission,
            transition,
            timestamp,
        )
        .expect("fixture record should construct");
        state
            .apply(&record)
            .expect("fixture transition should apply");
    }

    // Act
    let result = state.seal(timestamp + 1);

    // Assert
    assert_eq!(result, Err(Phase36LedgerError::Unclosed));
}

#[test]
fn phase36_broker_ledger_rejects_post_close_record() {
    // Arrange
    let (state, timestamp) = successful_state();
    let interval = state
        .seal(timestamp + 1)
        .expect("complete interval should seal");

    // Act
    let result = interval.record_after_close();

    // Assert
    assert_eq!(result, Err(Phase36LedgerError::PostClose));
}

#[test]
fn phase36_broker_ledger_preserves_earliest_failure_through_cleanup() {
    // Arrange
    let cases = [
        (
            Phase36AllowedOperation::ExactPackageAdmission,
            Phase36BrokerFailure::AdmissionFailed,
        ),
        (
            Phase36AllowedOperation::Board205DetectorProbe,
            Phase36BrokerFailure::DetectorFailed,
        ),
        (
            Phase36AllowedOperation::ExactPackageFlash,
            Phase36BrokerFailure::FlashFailed,
        ),
        (
            Phase36AllowedOperation::ReadOnlySystemInfo,
            Phase36BrokerFailure::CaptureFailed,
        ),
    ];

    // Act and Assert
    for (failed_operation, primary_failure) in cases {
        let mut state = Phase36LedgerState::start(1_000).expect("fixture interval should start");
        let mut timestamp = 1_000;
        for operation in Phase36AllowedOperation::SUCCESS_ORDER {
            if operation == failed_operation {
                for transition in [
                    Phase36LedgerTransition::Authorized,
                    Phase36LedgerTransition::Invoked,
                    Phase36LedgerTransition::Failed {
                        category: primary_failure,
                    },
                    Phase36LedgerTransition::Closed,
                ] {
                    timestamp += 1;
                    let record =
                        Phase36LedgerRecord::next(&state, operation, transition, timestamp)
                            .expect("failed operation record should construct");
                    state
                        .apply(&record)
                        .expect("failed operation record should apply");
                }
                break;
            }
            apply_successful_operation(&mut state, operation, &mut timestamp);
        }
        apply_successful_operation(
            &mut state,
            Phase36AllowedOperation::TypedRecovery,
            &mut timestamp,
        );
        for transition in [
            Phase36LedgerTransition::Authorized,
            Phase36LedgerTransition::Invoked,
            Phase36LedgerTransition::Failed {
                category: Phase36BrokerFailure::CleanupFailed,
            },
            Phase36LedgerTransition::Closed,
        ] {
            timestamp += 1;
            let record = Phase36LedgerRecord::next(
                &state,
                Phase36AllowedOperation::Cleanup,
                transition,
                timestamp,
            )
            .expect("cleanup record should construct");
            state.apply(&record).expect("cleanup record should apply");
        }
        let interval = state
            .seal(timestamp + 1)
            .expect("failed but complete interval should seal");
        assert_eq!(interval.first_failure(), Some(primary_failure));
        assert_eq!(
            interval.secondary_failure(),
            Some(Phase36BrokerFailure::CleanupFailed)
        );
    }
}

#[test]
fn phase36_broker_private_ledger_is_mode_0600_and_append_only() {
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::sync::atomic::{AtomicU64, Ordering};

    static NEXT_ROOT: AtomicU64 = AtomicU64::new(1);

    // Arrange
    let root = std::env::temp_dir().join(format!(
        "phase36-broker-test-{}-{}",
        std::process::id(),
        NEXT_ROOT.fetch_add(1, Ordering::Relaxed)
    ));
    fs::create_dir(&root).expect("private test root should create");
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700))
        .expect("private test root mode should set");
    let path = root.join("ledger.jsonl");
    let utf8_path = camino::Utf8PathBuf::from_path_buf(path.clone())
        .expect("temporary test path should be UTF-8");
    let state = Phase36LedgerState::start(1_000).expect("fixture interval should start");
    let record = Phase36LedgerRecord::next(
        &state,
        Phase36AllowedOperation::ExactPackageAdmission,
        Phase36LedgerTransition::Authorized,
        1_001,
    )
    .expect("authorization should construct");

    // Act
    let mut ledger = PrivateAppendOnlyLedger::create(&utf8_path)
        .expect("private append-only ledger should create");
    ledger
        .append(&record)
        .expect("private ledger append should succeed");
    ledger.seal().expect("private ledger should sync");
    drop(ledger);
    let metadata = fs::metadata(&path).expect("private ledger metadata should load");
    let contents = fs::read_to_string(&path).expect("private ledger should be readable by owner");

    // Assert
    assert_eq!(metadata.permissions().mode() & 0o777, 0o600);
    assert_eq!(contents.lines().count(), 1);
    assert!(contents.ends_with('\n'));

    fs::remove_dir_all(root).expect("private test root should clean up");
}
