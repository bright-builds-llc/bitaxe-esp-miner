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
        if state.recovery_required() {
            apply_successful_operation(
                &mut state,
                Phase36AllowedOperation::TypedRecovery,
                &mut timestamp,
            );
        }
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
fn phase36_broker_ledger_rejects_recovery_after_no_effect_failure() {
    // Arrange
    let mut state = Phase36LedgerState::start(1_000).expect("fixture interval should start");
    let mut timestamp = 1_000;
    for transition in [
        Phase36LedgerTransition::Authorized,
        Phase36LedgerTransition::Invoked,
        Phase36LedgerTransition::Failed {
            category: Phase36BrokerFailure::AdmissionFailed,
        },
        Phase36LedgerTransition::Closed,
    ] {
        timestamp += 1;
        let record = Phase36LedgerRecord::next(
            &state,
            Phase36AllowedOperation::ExactPackageAdmission,
            transition,
            timestamp,
        )
        .expect("failed admission record should construct");
        state
            .apply(&record)
            .expect("failed admission record should apply");
    }

    // Act
    let result = Phase36LedgerRecord::next(
        &state,
        Phase36AllowedOperation::TypedRecovery,
        Phase36LedgerTransition::Authorized,
        timestamp + 1,
    );

    apply_successful_operation(&mut state, Phase36AllowedOperation::Cleanup, &mut timestamp);
    let interval = state
        .seal(timestamp + 1)
        .expect("no-effect failure should seal after cleanup");

    // Assert
    assert_eq!(result, Err(Phase36LedgerError::OutOfOrder));
    assert_eq!(
        interval.recovery_disposition(),
        Phase36RecoveryDisposition::NotAuthorized
    );
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
    assert!(include_str!("ledger.rs").contains("libc::O_CLOEXEC"));

    fs::remove_dir_all(root).expect("private test root should clean up");
}

#[test]
fn phase36_broker_unix_frames_accept_coalesced_and_fragmented_delivery() {
    use std::io::Write;
    use std::os::unix::net::UnixStream;
    use std::thread;

    // Arrange
    let fixture: serde_json::Value =
        serde_json::from_str(include_str!("../../fixtures/phase36-broker/ipc-cases.json"))
            .expect("IPC fixture should parse");
    assert_eq!(fixture["schema_version"], "phase36-broker-ipc-cases-v1");
    let (mut receiver_stream, mut sender_stream) =
        UnixStream::pair().expect("Unix stream pair should create");
    let (_, presentation) = capability_and_presentation();
    let mut coalesced = Vec::new();
    write_broker_frame(&mut coalesced, 1, &presentation).expect("first frame should encode");
    write_broker_frame(&mut coalesced, 2, &presentation).expect("second frame should encode");
    let midpoint = coalesced.len() / 3;
    let writer = thread::spawn(move || {
        sender_stream
            .write_all(&coalesced[..midpoint])
            .expect("fragment should write");
        sender_stream
            .write_all(&coalesced[midpoint..])
            .expect("remaining coalesced frames should write");
    });
    let mut receiver = Phase36BrokerFrameReceiver::new();

    // Act
    let first = receiver
        .read_next::<Phase36CapabilityPresentation>(&mut receiver_stream)
        .expect("fragmented first frame should decode");
    let second = receiver
        .read_next::<Phase36CapabilityPresentation>(&mut receiver_stream)
        .expect("coalesced second frame should decode");
    writer.join().expect("writer should exit");

    // Assert
    assert_eq!(first, presentation);
    assert_eq!(second, presentation);
}

#[test]
fn phase36_broker_unix_frames_reject_short_oversized_duplicate_reordered_and_after_close() {
    use std::io::{Cursor, Write};
    use std::os::unix::net::UnixStream;

    // Arrange
    let (_, presentation) = capability_and_presentation();
    let mut short = Cursor::new([0_u8, 0, 0, 8, b'{', b'}']);
    let mut oversized = Cursor::new([0_u8, 1, 0, 1]);
    let mut duplicate_bytes = Vec::new();
    write_broker_frame(&mut duplicate_bytes, 1, &presentation)
        .expect("first duplicate fixture should encode");
    write_broker_frame(&mut duplicate_bytes, 1, &presentation)
        .expect("second duplicate fixture should encode");
    let mut duplicate = Cursor::new(duplicate_bytes);
    let mut reordered_bytes = Vec::new();
    write_broker_frame(&mut reordered_bytes, 2, &presentation)
        .expect("reordered fixture should encode");
    let mut reordered = Cursor::new(reordered_bytes);

    // Act
    let short_result =
        Phase36BrokerFrameReceiver::new().read_next::<Phase36CapabilityPresentation>(&mut short);
    let oversized_result = Phase36BrokerFrameReceiver::new()
        .read_next::<Phase36CapabilityPresentation>(&mut oversized);
    let mut duplicate_receiver = Phase36BrokerFrameReceiver::new();
    duplicate_receiver
        .read_next::<Phase36CapabilityPresentation>(&mut duplicate)
        .expect("first duplicate fixture frame should decode");
    let duplicate_result =
        duplicate_receiver.read_next::<Phase36CapabilityPresentation>(&mut duplicate);
    let reordered_result = Phase36BrokerFrameReceiver::new()
        .read_next::<Phase36CapabilityPresentation>(&mut reordered);
    let (mut closed_stream, mut peer) =
        UnixStream::pair().expect("closed Unix stream pair should create");
    peer.write_all(&[]).expect("empty write should succeed");
    drop(peer);
    let mut closed_receiver = Phase36BrokerFrameReceiver::new();
    closed_receiver.close();
    let after_close_result =
        closed_receiver.read_next::<Phase36CapabilityPresentation>(&mut closed_stream);

    // Assert
    assert_eq!(short_result, Err(Phase36BrokerIpcError::Truncated));
    assert_eq!(oversized_result, Err(Phase36BrokerIpcError::Oversized));
    assert_eq!(duplicate_result, Err(Phase36BrokerIpcError::Duplicate));
    assert_eq!(reordered_result, Err(Phase36BrokerIpcError::OutOfOrder));
    assert_eq!(after_close_result, Err(Phase36BrokerIpcError::AfterClose));
}
