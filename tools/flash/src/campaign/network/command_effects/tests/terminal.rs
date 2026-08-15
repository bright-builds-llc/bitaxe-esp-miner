use super::*;
use crate::CampaignTerminalCategory;

#[test]
fn completed_commands_wait_for_firmware_lease_consumption() {
    // Arrange
    let started = std::time::Instant::now();
    let after_the_full_lease = started + std::time::Duration::from_secs(601);

    // Act
    let failure = automated_phase_failure(CommandPhase::Terminal, started, after_the_full_lease);

    // Assert
    assert_eq!(failure, None);
}

#[test]
fn consumed_terminal_keeps_exact_http_confirmation_deadline() {
    // Arrange
    let consumed_at = std::time::Instant::now();
    let deadline = consumed_at + TERMINAL_DEADLINE;

    // Act
    let before = terminal_confirmation_timed_out(
        Some(deadline),
        deadline - std::time::Duration::from_nanos(1),
    );
    let at_deadline = terminal_confirmation_timed_out(Some(deadline), deadline);

    // Assert
    assert!(!before);
    assert!(at_deadline);
}

#[test]
fn command_failure_reuses_proved_paused_safe_stop_without_another_request() {
    // Arrange
    let mut evidence = CommandEffectsEvidence::new();
    evidence.pause_confirmed = true;
    let serial = SharedSerialState {
        resumable_pause_safe_stop_confirmed: true,
        ..SharedSerialState::default()
    };

    // Act
    let reusable = may_reuse_confirmed_safe_stop(
        Some(CampaignTerminalCategory::CommandRequestFailed),
        &evidence,
        &serial,
    );

    // Assert
    assert!(reusable);
}

#[test]
fn unproved_or_resumed_state_still_requires_recovery() {
    // Arrange
    let mut evidence = CommandEffectsEvidence::new();
    evidence.pause_confirmed = true;
    evidence.resume_request_count = 1;
    let serial = SharedSerialState {
        resumable_pause_safe_stop_confirmed: true,
        ..SharedSerialState::default()
    };

    // Act
    let after_resume = may_reuse_confirmed_safe_stop(
        Some(CampaignTerminalCategory::CommandRequestFailed),
        &evidence,
        &serial,
    );
    evidence.resume_request_count = 0;
    let without_serial_proof = may_reuse_confirmed_safe_stop(
        Some(CampaignTerminalCategory::CommandRequestFailed),
        &evidence,
        &SharedSerialState::default(),
    );

    // Assert
    assert!(!after_resume);
    assert!(!without_serial_proof);
}

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
