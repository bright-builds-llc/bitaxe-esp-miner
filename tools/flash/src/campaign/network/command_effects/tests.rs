use std::fs;

use camino::Utf8PathBuf;

use super::{
    arm_identify_transaction, arm_ready_after_pause, confirm_identify_checkpoint,
    consume_confirmation, consume_ready_signal, take_recovery_pause_request,
    write_required_checkpoint, CommandEffectsEvidence, CommandPhase, IdentifyCheckpointKind,
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
    confirm_identify_checkpoint(&root, IdentifyCheckpointKind::Rendered).expect("confirmation");
    let accepted = consume_confirmation(&root, IdentifyCheckpointKind::Rendered)
        .expect("consume confirmation");

    // Assert
    assert!(accepted);
    assert!(
        !consume_confirmation(&root, IdentifyCheckpointKind::Rendered)
            .expect("second checkpoint is absent")
    );
    assert!(confirm_identify_checkpoint(&root, IdentifyCheckpointKind::Rendered).is_err());
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
fn ready_signal_is_the_only_transition_that_releases_resume() {
    // Arrange
    let (_temp, root) = private_root();
    let mut evidence = CommandEffectsEvidence::new();
    evidence.pause_request_count = 1;
    arm_ready_after_pause(&root, &mut evidence).expect("arm paused readiness");

    // Act
    let absent = consume_ready_signal(&root, &mut evidence).expect("absent signal");
    confirm_identify_checkpoint(&root, IdentifyCheckpointKind::Ready).expect("ready signal");
    let present = consume_ready_signal(&root, &mut evidence).expect("consume signal");

    // Assert
    assert!(!absent);
    assert!(present);
    assert!(evidence.identify_operator_ready_confirmed);
    assert_eq!(evidence.resume_request_count, 1);
    assert_eq!(evidence.identify_request_count, 0);
    assert!(root.join("identify-ready.consumed.json").is_file());
}

#[test]
fn malformed_confirmation_fails_closed() {
    // Arrange
    let (_temp, root) = private_root();
    write_required_checkpoint(&root, IdentifyCheckpointKind::Cleared).expect("required checkpoint");
    let confirmed = root.join("identify-cleared.confirmed.json");
    crate::write_private_new_bytes(
        &confirmed,
        br#"{"schema":"wrong","checkpoint":"cleared","status":"confirmed"}"#,
    )
    .expect("malformed confirmation");

    // Act
    let result = consume_confirmation(&root, IdentifyCheckpointKind::Cleared);

    // Assert
    assert!(result.is_err());
}

#[test]
fn confirmation_without_a_required_checkpoint_is_rejected() {
    // Arrange
    let (_temp, root) = private_root();

    // Act
    let result = confirm_identify_checkpoint(&root, IdentifyCheckpointKind::Rendered);

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
