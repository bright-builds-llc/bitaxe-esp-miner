use super::*;
use crate::CampaignTerminalCategory;

#[test]
fn pause_convergence_increment_is_preserved_across_dismissal() {
    // Arrange
    let (_temp, root) = private_root();
    let (http, server) = successful_command_server();
    let mut sample = SystemInfoWire::from_snapshot(&ApiSnapshot::safe_ultra_205());
    sample.mining_paused = true;
    sample.mining_activity = "paused".to_owned();
    sample.block_found = 8;
    sample.show_new_block = true;
    let started = std::time::Instant::now();
    let mut phase = CommandPhase::Pause(PauseJoinState::new(started));
    let mut evidence = CommandEffectsEvidence::new();
    evidence.pause_request_count = 1;
    let mut maybe_block_count = Some(7);
    let mut maybe_failure = None;
    let serial = SharedSerialState {
        resumable_pause_safe_stop_confirmed: true,
        ..SharedSerialState::default()
    };

    // Act
    advance_commands(
        &http,
        &trusted_target(),
        &root,
        &sample,
        &serial,
        started,
        CommandProgress {
            phase: &mut phase,
            maybe_block_count: &mut maybe_block_count,
            evidence: &mut evidence,
            maybe_failure: &mut maybe_failure,
        },
    );
    sample.show_new_block = false;
    advance_commands(
        &http,
        &trusted_target(),
        &root,
        &sample,
        &serial,
        started + std::time::Duration::from_millis(1),
        CommandProgress {
            phase: &mut phase,
            maybe_block_count: &mut maybe_block_count,
            evidence: &mut evidence,
            maybe_failure: &mut maybe_failure,
        },
    );

    // Assert
    assert_eq!(phase, CommandPhase::IdentifyReady);
    assert_eq!(maybe_block_count, Some(8));
    assert!(evidence.block_count_preserved);
    assert_eq!(maybe_failure, None);
    assert_eq!(
        server.join().expect("command server thread"),
        "POST /api/system/blockFound/dismiss HTTP/1.1"
    );
}

#[test]
fn zero_paused_count_fails_before_dismissal_request() {
    // Arrange
    let (_temp, root) = private_root();
    let http = StrictHttpClient::new("http://127.0.0.1:9").expect("HTTP client");
    let mut sample = SystemInfoWire::from_snapshot(&ApiSnapshot::safe_ultra_205());
    sample.mining_paused = true;
    sample.mining_activity = "paused".to_owned();
    sample.block_found = 0;
    let started = std::time::Instant::now();
    let mut phase = CommandPhase::Pause(PauseJoinState::new(started));
    let mut evidence = CommandEffectsEvidence::new();
    evidence.pause_request_count = 1;
    let mut maybe_block_count = Some(1);
    let mut maybe_failure = None;
    let serial = SharedSerialState {
        resumable_pause_safe_stop_confirmed: true,
        ..SharedSerialState::default()
    };

    // Act
    advance_commands(
        &http,
        &trusted_target(),
        &root,
        &sample,
        &serial,
        started,
        CommandProgress {
            phase: &mut phase,
            maybe_block_count: &mut maybe_block_count,
            evidence: &mut evidence,
            maybe_failure: &mut maybe_failure,
        },
    );

    // Assert
    assert!(matches!(phase, CommandPhase::Pause(_)));
    assert_eq!(maybe_block_count, Some(1));
    assert_eq!(evidence.dismiss_request_count, 0);
    assert_eq!(
        maybe_failure,
        Some(CampaignTerminalCategory::NetworkCorrelationFailed)
    );
}

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
