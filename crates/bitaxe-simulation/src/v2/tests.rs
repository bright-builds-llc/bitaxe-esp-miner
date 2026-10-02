use super::{
    run_encrypted_share_on_board, run_encrypted_share_profile, scenario::run_with_fault, Fault,
};
use crate::{CheckStatus, SimulationError};
use bitaxe_virtual_board::{BoardConfig, VirtualBoard};

#[test]
fn encrypted_genesis_share_has_independent_target_and_actual_ack() {
    // Arrange
    let seed = 91;
    // Act
    let (facts, checks) =
        run_encrypted_share_profile(seed).expect("generic production encrypted profile");
    // Assert
    assert_eq!(
        facts.header_sha256d,
        "6fe28c0ab6f1b372c1a6a246ae63f74f931e8365e15a089c68d6190000000000"
    );
    assert_eq!(facts.acknowledged_shares, 1);
    assert!(facts.independently_target_valid);
    assert!(facts.resources_released);
    assert_eq!(facts.encrypted_frames_delivered, 9);
    assert_eq!(checks[0].status, CheckStatus::Passed);
    assert_eq!(checks[1].status, CheckStatus::Unsupported);
}
#[test]
fn wrong_responder_authority_cannot_authenticate() {
    // Arrange / Act
    let result = run_with_fault(92, Fault::WrongAuthority);
    // Assert
    assert!(matches!(
        result,
        Err(SimulationError::Boundary("noise_authentication"))
    ));
}
#[test]
fn corrupted_encrypted_submission_is_not_a_fixture_share() {
    // Arrange / Act
    let result = run_with_fault(93, Fault::TamperedEncryptedSubmission);
    // Assert
    assert!(matches!(
        result,
        Err(SimulationError::Boundary("server_payload_authentication"))
    ));
}
#[test]
fn replayed_encrypted_submission_is_rejected_by_nonce_state() {
    // Arrange / Act
    let result = run_with_fault(94, Fault::ReplayedEncryptedSubmission);
    // Assert
    assert!(matches!(
        result,
        Err(SimulationError::Boundary("server_header_authentication"))
    ));
}
#[test]
fn partial_encrypted_submission_cannot_be_completed_by_an_ack() {
    // Arrange / Act
    let result = run_with_fault(95, Fault::TruncatedEncryptedSubmission);
    // Assert
    assert!(matches!(
        result,
        Err(SimulationError::Boundary("server_payload_length"))
    ));
}
#[test]
fn valid_encryption_does_not_override_wrong_submission_job() {
    // Arrange / Act
    let result = run_with_fault(96, Fault::InvalidPlainSubmission);
    // Assert
    assert!(matches!(
        result,
        Err(SimulationError::Boundary("independent_submission"))
    ));
}
#[test]
fn wrong_channel_ack_cannot_credit_current_submission() {
    // Arrange / Act
    let result = run_with_fault(97, Fault::WrongAcknowledgementChannel);
    // Assert
    assert!(matches!(
        result,
        Err(SimulationError::Boundary("independent_ack_correlation"))
    ));
}
#[test]
fn inflated_ack_cannot_turn_one_submission_into_two() {
    // Arrange / Act
    let result = run_with_fault(98, Fault::InflatedAcknowledgement);
    // Assert
    assert!(matches!(
        result,
        Err(SimulationError::Boundary("independent_ack_correlation"))
    ));
}
#[test]
fn corrupted_encrypted_ack_is_not_a_completion_observation() {
    // Arrange / Act
    let result = run_with_fault(99, Fault::TamperedEncryptedAcknowledgement);
    // Assert
    assert!(matches!(
        result,
        Err(SimulationError::Boundary("client_authentication"))
    ));
}
#[test]
fn ack_without_completed_write_cannot_credit_submission() {
    // Arrange / Act
    let result = run_with_fault(100, Fault::IncompleteSubmissionWrite);
    // Assert
    assert!(matches!(
        result,
        Err(SimulationError::Boundary("independent_ack_correlation"))
    ));
}
#[test]
fn seeded_encrypted_profile_preserves_semantic_results() {
    // Arrange / Act
    let first = run_encrypted_share_profile(101).expect("genericprofile");
    let second = run_encrypted_share_profile(101).expect("genericprofile");
    // Assert
    assert_eq!(
        serde_json::to_value(&first).expect("facts"),
        serde_json::to_value(&second).expect("facts")
    );
}
#[test]
fn controller_profile_rejects_unprepared_board() {
    // Arrange
    let mut board = VirtualBoard::new(BoardConfig::default()).expect("board");
    // Act
    let result = run_encrypted_share_on_board(102, &mut board, |_| Ok(()));
    // Assert
    assert!(matches!(
        result,
        Err(SimulationError::Boundary("controller_board_not_prepared"))
    ));
}
#[test]
fn controller_revocation_prevents_handshake_and_dispatch() {
    // Arrange
    let mut board = VirtualBoard::new(BoardConfig::default()).expect("board");
    board.peripherals.power_enabled = true;
    board.peripherals.reset_asserted = false;
    board.asic.frequency_mhz = 485;
    // Act
    let result = run_encrypted_share_on_board(103, &mut board, |_| {
        Err(SimulationError::Boundary("revoked"))
    });
    // Assert
    assert!(matches!(result, Err(SimulationError::Boundary("revoked"))));
    assert_eq!(board.asic.dispatched_jobs, 0);
}

#[test]
fn successful_protocol_does_not_hide_rejected_resource_close() {
    // Arrange / Act
    let result = run_with_fault(104, Fault::RejectedClose);
    // Assert
    assert!(matches!(
        result,
        Err(SimulationError::Model(
            bitaxe_virtual_board::ModelError::Unavailable("transport close")
        ))
    ));
}
#[test]
fn rejected_cleanup_preserves_earliest_submission_failure_separately() {
    // Arrange / Act
    let result = run_with_fault(105, Fault::InvalidSubmissionAndRejectedClose);
    // Assert
    let Err(SimulationError::Cleanup { original, cleanup }) = result else {
        panic!("both failures required");
    };
    assert!(matches!(
        *original,
        SimulationError::Boundary("independent_submission")
    ));
    assert!(matches!(
        *cleanup,
        SimulationError::Model(bitaxe_virtual_board::ModelError::Unavailable(
            "transport close"
        ))
    ));
}

#[test]
fn dispatch_gate_rejection_occurs_after_channel_before_asic_work() {
    // Arrange
    let mut board = VirtualBoard::new(BoardConfig::default()).expect("board");
    board.peripherals.power_enabled = true;
    board.peripherals.reset_asserted = false;
    board.asic.frequency_mhz = 485;
    let mut calls = 0;
    // Act
    let result = super::run_encrypted_share_on_board_with_dispatch(
        106,
        &mut board,
        |_| Ok(()),
        |_| {
            calls += 1;
            Err(SimulationError::Boundary("dispatch_revoked"))
        },
    );
    // Assert
    assert!(matches!(
        result,
        Err(SimulationError::Boundary("dispatch_revoked"))
    ));
    assert_eq!(calls, 1);
    assert_eq!(board.asic.dispatched_jobs, 0);
}

#[test]
fn completion_observer_reports_actual_dispatch_submit_ack_order() {
    // Arrange
    let mut board = VirtualBoard::new(BoardConfig::default()).expect("board");
    board.peripherals.power_enabled = true;
    board.peripherals.reset_asserted = false;
    board.asic.frequency_mhz = 485;
    let mut observed = Vec::new();
    // Act
    let facts = super::run_encrypted_share_on_board_with_io(
        107,
        &mut board,
        |_| Ok(()),
        |_| Ok(()),
        |event, _| observed.push(event),
    )
    .expect("profile");
    // Assert
    assert_eq!(
        observed,
        vec![
            super::ProfileIoEvent::AsicWriteCompleted,
            super::ProfileIoEvent::ShareWriteCompleted,
            super::ProfileIoEvent::AcknowledgementValidated
        ]
    );
    assert_eq!(facts.acknowledged_shares, 1);
    assert_eq!(board.asic.dispatched_jobs, 1);
}

#[test]
fn completed_submission_write_remains_observed_when_fixture_mac_rejects() {
    // Arrange
    let mut board = VirtualBoard::new(BoardConfig::default()).expect("board");
    board.peripherals.power_enabled = true;
    board.peripherals.reset_asserted = false;
    board.asic.frequency_mhz = 485;
    let mut observed = Vec::new();
    // Act
    let result = super::scenario::run_exchange(
        108,
        Fault::TamperedEncryptedSubmission,
        &mut board,
        &mut |_| Ok(()),
        &mut |_| Ok(()),
        &mut |event, _| observed.push(event),
    );
    // Assert
    assert!(matches!(
        result,
        Err(SimulationError::Boundary("server_payload_authentication"))
    ));
    assert_eq!(
        observed,
        vec![
            super::ProfileIoEvent::AsicWriteCompleted,
            super::ProfileIoEvent::ShareWriteCompleted
        ]
    );
}
#[test]
fn partial_submission_never_emits_completed_write_observation() {
    // Arrange
    let mut board = VirtualBoard::new(BoardConfig::default()).expect("board");
    board.peripherals.power_enabled = true;
    board.peripherals.reset_asserted = false;
    board.asic.frequency_mhz = 485;
    let mut observed = Vec::new();
    // Act
    let result = super::scenario::run_exchange(
        109,
        Fault::TruncatedEncryptedSubmission,
        &mut board,
        &mut |_| Ok(()),
        &mut |_| Ok(()),
        &mut |event, _| observed.push(event),
    );
    // Assert
    assert!(matches!(
        result,
        Err(SimulationError::Boundary("server_payload_length"))
    ));
    assert_eq!(observed, vec![super::ProfileIoEvent::AsicWriteCompleted]);
}

static OBSERVED_HELPER_THREAD: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);

fn record_helper_thread() {
    if let Ok(mut observed) = OBSERVED_HELPER_THREAD.lock() {
        *observed = std::thread::current().name().map(str::to_owned);
    }
}

#[test]
fn handshake_crypto_runs_on_the_dedicated_helper_thread() {
    // Arrange
    super::exchange::set_handshake_stack_observer(record_helper_thread);
    // Act
    let exchange = super::exchange::Exchange::new(5, false);
    // Assert
    assert!(exchange.is_ok());
    let observed = OBSERVED_HELPER_THREAD
        .lock()
        .expect("observer state")
        .clone();
    assert_eq!(observed.as_deref(), Some("noise-handshake"));
}

#[test]
fn helper_failure_keeps_the_typed_authentication_boundary() {
    // Arrange / Act
    let result = super::exchange::Exchange::new(5, true);
    // Assert
    assert!(matches!(
        result,
        Err(SimulationError::Boundary("noise_authentication"))
    ));
}
