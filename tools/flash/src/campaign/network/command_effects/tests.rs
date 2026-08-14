use std::fs;

use camino::Utf8PathBuf;

use super::{
    arm_identify_transaction, arm_ready_after_pause, automated_phase_expired,
    consume_checkpoint_response, consume_cleared_signal, consume_ready_signal,
    rendered_checkpoint_expired, respond_identify_checkpoint, take_recovery_pause_request,
    write_required_checkpoint, CheckpointResponse, CommandEffectsEvidence, CommandPhase,
    IdentifyCheckpointKind, IdentifyCheckpointOutcome,
};
use crate::set_private_directory_mode;

fn private_root() -> (tempfile::TempDir, Utf8PathBuf) {
    let temp = tempfile::tempdir().expect("tempdir");
    let root = Utf8PathBuf::from_path_buf(temp.path().join("attempt")).expect("utf8 attempt path");
    fs::create_dir(&root).expect("create attempt");
    set_private_directory_mode(&root).expect("private attempt");
    (temp, root)
}

#[test]
fn confirmation_is_consumed_once() {
    // Arrange
    let (_temp, root) = private_root();
    write_required_checkpoint(&root, IdentifyCheckpointKind::Rendered)
        .expect("required checkpoint");

    // Act
    respond_identify_checkpoint(
        &root,
        IdentifyCheckpointKind::Rendered,
        IdentifyCheckpointOutcome::Confirmed,
    )
    .expect("confirmation");
    let accepted = consume_checkpoint_response(&root, IdentifyCheckpointKind::Rendered)
        .expect("consume confirmation");

    // Assert
    assert_eq!(accepted, CheckpointResponse::Confirmed);
    assert!(
        consume_checkpoint_response(&root, IdentifyCheckpointKind::Rendered)
            .expect("second checkpoint is absent")
            == CheckpointResponse::Pending
    );
    assert!(respond_identify_checkpoint(
        &root,
        IdentifyCheckpointKind::Rendered,
        IdentifyCheckpointOutcome::Confirmed,
    )
    .is_err());
}

#[test]
fn identify_transaction_arms_readiness_without_issuing_a_request() {
    // Arrange
    let (_temp, root) = private_root();
    let evidence = CommandEffectsEvidence::new();

    // Act
    let phase = arm_identify_transaction(&root, &evidence).expect("arm transaction");

    // Assert
    assert_eq!(phase, CommandPhase::IdentifyReady);
    assert_eq!(evidence.identify_request_count, 0);
    assert!(!evidence.identify_operator_ready_confirmed);
    assert!(root.join("identify-ready.required.json").is_file());
    assert!(!root.join("identify-rendered.required.json").exists());
}

#[test]
fn safe_stopped_pause_arms_ready_without_resume_or_identify() {
    // Arrange
    let (_temp, root) = private_root();
    let mut evidence = CommandEffectsEvidence::new();
    evidence.pause_request_count = 1;

    // Act
    let phase = arm_ready_after_pause(&root, &mut evidence).expect("arm paused readiness");

    // Assert
    assert_eq!(phase, CommandPhase::IdentifyReady);
    assert!(evidence.pause_confirmed);
    assert_eq!(evidence.resume_request_count, 0);
    assert_eq!(evidence.identify_request_count, 0);
    assert!(root.join("identify-ready.required.json").is_file());
}

#[test]
fn ready_signal_is_consumed_without_releasing_pause() {
    // Arrange
    let (_temp, root) = private_root();
    let mut evidence = CommandEffectsEvidence::new();
    evidence.pause_request_count = 1;
    arm_ready_after_pause(&root, &mut evidence).expect("arm paused readiness");

    // Act
    let absent = consume_ready_signal(&root, &mut evidence).expect("absent signal");
    respond_identify_checkpoint(
        &root,
        IdentifyCheckpointKind::Ready,
        IdentifyCheckpointOutcome::Confirmed,
    )
    .expect("ready signal");
    let present = consume_ready_signal(&root, &mut evidence).expect("consume signal");

    // Assert
    assert_eq!(absent, CheckpointResponse::Pending);
    assert_eq!(present, CheckpointResponse::Confirmed);
    assert!(evidence.identify_operator_ready_confirmed);
    assert_eq!(evidence.resume_request_count, 0);
    assert_eq!(evidence.identify_request_count, 0);
    assert!(root.join("identify-ready.consumed.json").is_file());
}

#[test]
fn cleared_signal_is_the_only_transition_that_releases_resume() {
    // Arrange
    let (_temp, root) = private_root();
    let mut evidence = CommandEffectsEvidence::new();
    evidence.pause_confirmed = true;
    evidence.identify_operator_ready_confirmed = true;
    evidence.identify_request_count = 2;
    evidence.identify_rendered_confirmed = true;
    write_required_checkpoint(&root, IdentifyCheckpointKind::Cleared).expect("required checkpoint");

    // Act
    let absent = consume_cleared_signal(&root, &mut evidence).expect("absent signal");
    respond_identify_checkpoint(
        &root,
        IdentifyCheckpointKind::Cleared,
        IdentifyCheckpointOutcome::Confirmed,
    )
    .expect("cleared signal");
    let present = consume_cleared_signal(&root, &mut evidence).expect("consume signal");

    // Assert
    assert_eq!(absent, CheckpointResponse::Pending);
    assert_eq!(present, CheckpointResponse::Confirmed);
    assert!(evidence.identify_cleared_confirmed);
    assert_eq!(evidence.resume_request_count, 1);
    assert!(root.join("identify-cleared.consumed.json").is_file());
}

#[test]
fn malformed_confirmation_fails_closed() {
    // Arrange
    let (_temp, root) = private_root();
    write_required_checkpoint(&root, IdentifyCheckpointKind::Cleared).expect("required checkpoint");
    let response = root.join("identify-cleared.response.json");
    crate::write_private_new_bytes(
        &response,
        br#"{"schema":"wrong","checkpoint":"cleared","status":"confirmed"}"#,
    )
    .expect("malformed confirmation");

    // Act
    let result = consume_checkpoint_response(&root, IdentifyCheckpointKind::Cleared);

    // Assert
    assert!(result.is_err());
}

#[test]
fn declined_rendered_checkpoint_is_a_typed_response() {
    // Arrange
    let (_temp, root) = private_root();
    write_required_checkpoint(&root, IdentifyCheckpointKind::Rendered)
        .expect("required checkpoint");
    respond_identify_checkpoint(
        &root,
        IdentifyCheckpointKind::Rendered,
        IdentifyCheckpointOutcome::Declined,
    )
    .expect("declined response");

    // Act
    let result = consume_checkpoint_response(&root, IdentifyCheckpointKind::Rendered);

    // Assert
    assert_eq!(result, Ok(CheckpointResponse::Declined));
}

#[test]
fn rendered_checkpoint_expires_at_the_exact_effect_deadline() {
    // Arrange
    let started = std::time::Instant::now();
    let expires_at = started + std::time::Duration::from_secs(30);

    // Act
    let before =
        rendered_checkpoint_expired(expires_at - std::time::Duration::from_nanos(1), expires_at);
    let exact = rendered_checkpoint_expired(expires_at, expires_at);

    // Assert
    assert!(!before);
    assert!(exact);
}

#[test]
fn human_checkpoint_phases_have_no_elapsed_deadline() {
    // Arrange
    let started = std::time::Instant::now();
    let overnight = started + std::time::Duration::from_secs(24 * 60 * 60);

    // Act
    let ready_expired = automated_phase_expired(CommandPhase::IdentifyReady, started, overnight);
    let cleared_expired =
        automated_phase_expired(CommandPhase::IdentifyCleared, started, overnight);

    // Assert
    assert!(!ready_expired);
    assert!(!cleared_expired);
}

#[test]
fn automated_notification_phase_keeps_its_exact_deadline() {
    // Arrange
    let started = std::time::Instant::now();
    let exact = started + std::time::Duration::from_secs(600);

    // Act
    let before = automated_phase_expired(
        CommandPhase::Notification,
        started,
        exact - std::time::Duration::from_nanos(1),
    );
    let at_deadline = automated_phase_expired(CommandPhase::Notification, started, exact);

    // Assert
    assert!(!before);
    assert!(at_deadline);
}

#[test]
fn confirmation_without_a_required_checkpoint_is_rejected() {
    // Arrange
    let (_temp, root) = private_root();

    // Act
    let result = respond_identify_checkpoint(
        &root,
        IdentifyCheckpointKind::Rendered,
        IdentifyCheckpointOutcome::Confirmed,
    );

    // Assert
    assert!(result.is_err());
}

#[test]
fn recovery_pause_is_claimed_once_without_replacing_the_primary_failure() {
    // Arrange
    let primary = Some(super::CampaignTerminalCategory::CommandRequestFailed);
    let mut request_count = 0;

    // Act
    let first = take_recovery_pause_request(primary, &mut request_count);
    let second = take_recovery_pause_request(primary, &mut request_count);

    // Assert
    assert!(first);
    assert!(!second);
    assert_eq!(request_count, 1);
    assert_eq!(
        primary,
        Some(super::CampaignTerminalCategory::CommandRequestFailed)
    );
}
