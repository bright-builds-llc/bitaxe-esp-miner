use super::*;
use crate::CampaignTerminalCategory;

#[test]
fn active_reactivation_after_paused_dismissal_advances_directly_to_terminal() {
    // Arrange
    let (_temp, root) = private_root();
    let http = StrictHttpClient::new("http://127.0.0.1:9").expect("HTTP client");
    let mut sample = SystemInfoWire::from_snapshot(&ApiSnapshot::safe_ultra_205());
    sample.mining_paused = false;
    sample.mining_activity = "active".to_owned();
    let mut phase = CommandPhase::ResumeActive;
    let mut evidence = CommandEffectsEvidence::new();
    evidence.pause_confirmed = true;
    evidence.dismiss_request_count = 1;
    evidence.dismiss_confirmed = true;
    evidence.block_count_preserved = true;
    evidence.resume_request_count = 1;
    evidence.resume_intent_confirmed = true;
    let mut maybe_block_count = Some(1);
    let mut maybe_failure = None;

    // Act
    advance_commands(
        &http,
        &trusted_target(),
        &root,
        &sample,
        &SharedSerialState::default(),
        std::time::Instant::now(),
        CommandProgress {
            phase: &mut phase,
            maybe_block_count: &mut maybe_block_count,
            evidence: &mut evidence,
            maybe_failure: &mut maybe_failure,
        },
    );

    // Assert
    assert_eq!(phase, CommandPhase::Terminal);
    assert!(evidence.resume_confirmed);
    assert!(evidence.active_after_resume);
    assert_eq!(evidence.dismiss_request_count, 1);
    assert_eq!(maybe_failure, None);
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
fn replay_is_rejected_for_non_rendered_checkpoints() {
    // Arrange
    let (_temp, root) = private_root();
    write_required_checkpoint(&root, IdentifyCheckpointKind::Ready).expect("required checkpoint");

    // Act
    let result = respond_identify_checkpoint(
        &root,
        IdentifyCheckpointKind::Ready,
        IdentifyCheckpointOutcome::Replay,
    );

    // Assert
    assert!(result.is_err());
    assert!(!root.join("identify-ready.response.json").exists());
}

#[test]
fn recovery_pause_is_claimed_once_without_replacing_the_primary_failure() {
    // Arrange
    let primary = Some(CampaignTerminalCategory::CommandRequestFailed);
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
        Some(CampaignTerminalCategory::CommandRequestFailed)
    );
}
