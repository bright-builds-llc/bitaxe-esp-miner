use super::*;
use std::time::Instant;

use crate::campaign::CampaignTerminalCategory;

fn command_status(
    tracker: &CommandStatusTracker,
    sample: &SystemInfoWire,
    identify_active: bool,
) -> bitaxe_api::CommandStatusWire {
    tracker.snapshot(
        "0".repeat(32)
            .parse::<BootSessionId>()
            .expect("boot session"),
        1_000,
        CommandStatusFacts {
            mining_paused: sample.mining_paused,
            mining_activity: &sample.mining_activity,
            identify_active,
            block_found: sample.block_found,
            block_notification_visible: sample.show_new_block,
        },
    )
}

#[test]
fn programmatic_campaign_proves_all_command_effects_without_operator_input() {
    // Arrange
    let (_temp, root) = private_root();
    let target = trusted_target();
    let mut sample = SystemInfoWire::from_snapshot(&ApiSnapshot::safe_ultra_205());
    sample.mining_paused = false;
    sample.mining_activity = "active".to_owned();
    sample.show_new_block = true;
    sample.block_found = 7;
    let mut tracker = CommandStatusTracker::default();
    tracker.record_display_availability(true, 0);
    let mut phase = CommandPhase::Notification;
    let mut generations = CommandGenerations::default();
    let mut evidence = CommandEffectsEvidence::new();
    let mut maybe_block_count = None;
    let mut maybe_failure = None;
    let mut serial = SharedSerialState::default();
    let websocket = CommandTransitionWitness::default();
    let started = std::time::Instant::now();

    // Act: pause once, then prove the HTTP generation and native safe-stop witness.
    let (pause_http, pause_server) = successful_command_server();
    advance_programmatic_commands(
        &pause_http,
        &target,
        &root,
        &sample,
        &command_status(&tracker, &sample, false),
        &serial,
        &websocket,
        true,
        started,
        &mut generations,
        CommandProgress {
            phase: &mut phase,
            maybe_block_count: &mut maybe_block_count,
            evidence: &mut evidence,
            maybe_failure: &mut maybe_failure,
        },
    );
    assert_eq!(
        pause_server.join().expect("pause server"),
        "POST /api/system/pause HTTP/1.1"
    );
    tracker.record_command(CommandStatusEffect::Pause);
    sample.mining_paused = true;
    sample.mining_activity = "paused".to_owned();
    serial.resumable_pause_safe_stop_confirmed = true;
    serial.command_transitions.pause_generation = 1;

    let (dismiss_http, dismiss_server) = successful_command_server();
    advance_programmatic_commands(
        &dismiss_http,
        &target,
        &root,
        &sample,
        &command_status(&tracker, &sample, false),
        &serial,
        &websocket,
        true,
        started,
        &mut generations,
        CommandProgress {
            phase: &mut phase,
            maybe_block_count: &mut maybe_block_count,
            evidence: &mut evidence,
            maybe_failure: &mut maybe_failure,
        },
    );
    assert_eq!(
        dismiss_server.join().expect("dismiss server"),
        "POST /api/system/blockFound/dismiss HTTP/1.1"
    );
    tracker.record_command(CommandStatusEffect::BlockFoundDismiss);
    sample.show_new_block = false;
    serial.command_transitions.dismiss_generation = 1;
    let idle_http = StrictHttpClient::new("http://127.0.0.1:9").expect("idle HTTP client");
    advance_programmatic_commands(
        &idle_http,
        &target,
        &root,
        &sample,
        &command_status(&tracker, &sample, false),
        &serial,
        &websocket,
        true,
        started,
        &mut generations,
        CommandProgress {
            phase: &mut phase,
            maybe_block_count: &mut maybe_block_count,
            evidence: &mut evidence,
            maybe_failure: &mut maybe_failure,
        },
    );

    // Act: apply IDENTIFY once, prove the flushed frame, and prove its natural clearing.
    let (identify_http, identify_server) = successful_command_server();
    advance_programmatic_commands(
        &identify_http,
        &target,
        &root,
        &sample,
        &command_status(&tracker, &sample, false),
        &serial,
        &websocket,
        true,
        started,
        &mut generations,
        CommandProgress {
            phase: &mut phase,
            maybe_block_count: &mut maybe_block_count,
            evidence: &mut evidence,
            maybe_failure: &mut maybe_failure,
        },
    );
    assert_eq!(
        identify_server.join().expect("identify server"),
        "POST /api/system/identify HTTP/1.1"
    );
    tracker.record_command(CommandStatusEffect::IdentifyEnable {
        expires_at_uptime_ms: 30_000,
    });
    tracker.record_display(
        DisplayFrameKind::Identify,
        DisplayRenderOutcome::Rendered,
        2_000,
    );
    serial.command_transitions.identify_generation = 1;
    serial.command_transitions.display_identify_generation = 1;
    advance_programmatic_commands(
        &idle_http,
        &target,
        &root,
        &sample,
        &command_status(&tracker, &sample, true),
        &serial,
        &websocket,
        true,
        started,
        &mut generations,
        CommandProgress {
            phase: &mut phase,
            maybe_block_count: &mut maybe_block_count,
            evidence: &mut evidence,
            maybe_failure: &mut maybe_failure,
        },
    );
    tracker.record_display(
        DisplayFrameKind::NonIdentify,
        DisplayRenderOutcome::Rendered,
        31_000,
    );
    serial.command_transitions.display_non_identify_generation = 1;

    let (resume_http, resume_server) = successful_command_server();
    advance_programmatic_commands(
        &resume_http,
        &target,
        &root,
        &sample,
        &command_status(&tracker, &sample, false),
        &serial,
        &websocket,
        true,
        started,
        &mut generations,
        CommandProgress {
            phase: &mut phase,
            maybe_block_count: &mut maybe_block_count,
            evidence: &mut evidence,
            maybe_failure: &mut maybe_failure,
        },
    );
    assert_eq!(
        resume_server.join().expect("resume server"),
        "POST /api/system/resume HTTP/1.1"
    );
    tracker.record_command(CommandStatusEffect::Resume);
    sample.mining_paused = false;
    sample.mining_activity = "starting".to_owned();
    serial.command_transitions.resume_generation = 1;
    advance_programmatic_commands(
        &idle_http,
        &target,
        &root,
        &sample,
        &command_status(&tracker, &sample, false),
        &serial,
        &websocket,
        true,
        started,
        &mut generations,
        CommandProgress {
            phase: &mut phase,
            maybe_block_count: &mut maybe_block_count,
            evidence: &mut evidence,
            maybe_failure: &mut maybe_failure,
        },
    );
    sample.mining_activity = "active".to_owned();
    advance_programmatic_commands(
        &idle_http,
        &target,
        &root,
        &sample,
        &command_status(&tracker, &sample, false),
        &serial,
        &websocket,
        true,
        started,
        &mut generations,
        CommandProgress {
            phase: &mut phase,
            maybe_block_count: &mut maybe_block_count,
            evidence: &mut evidence,
            maybe_failure: &mut maybe_failure,
        },
    );
    evidence.terminal_http_valid = true;
    evidence.terminal_pool_persisted = true;

    // Assert
    assert_eq!(phase, CommandPhase::Terminal);
    assert_eq!(maybe_failure, None);
    assert!(evidence.complete());
    assert_eq!(evidence.identify_request_count, 1);
    assert!(evidence.serial_transition_witnesses_confirmed);
    assert!(!evidence.websocket_transition_witnesses_confirmed);
}

#[test]
fn pause_join_uses_claim_specific_http_generation_and_safe_stop_without_log_quorum() {
    // Arrange: reproduce attempt-028's exact proved boundary. The device has
    // applied pause and emitted the serial safe-stop fact, while neither log
    // channel has supplied the optional transition marker.
    let (_temp, root) = private_root();
    let target = trusted_target();
    let mut sample = SystemInfoWire::from_snapshot(&ApiSnapshot::safe_ultra_205());
    sample.mining_paused = true;
    sample.mining_activity = "paused".to_owned();
    sample.show_new_block = true;
    sample.block_found = 7;
    let mut tracker = CommandStatusTracker::default();
    tracker.record_command(CommandStatusEffect::Pause);
    let mut phase = CommandPhase::ProgrammaticPause(PauseJoinState::new(Instant::now()));
    let mut generations = CommandGenerations {
        pause: 1,
        ..CommandGenerations::default()
    };
    let mut evidence = CommandEffectsEvidence::new();
    evidence.pause_request_count = 1;
    let mut maybe_block_count = Some(7);
    let mut maybe_failure = None;
    let serial = SharedSerialState {
        resumable_pause_safe_stop_confirmed: true,
        ..SharedSerialState::default()
    };
    let websocket = CommandTransitionWitness::default();
    let idle_http = StrictHttpClient::new("http://127.0.0.1:9").expect("idle HTTP client");

    // Act
    advance_programmatic_commands(
        &idle_http,
        &target,
        &root,
        &sample,
        &command_status(&tracker, &sample, false),
        &serial,
        &websocket,
        true,
        Instant::now(),
        &mut generations,
        CommandProgress {
            phase: &mut phase,
            maybe_block_count: &mut maybe_block_count,
            evidence: &mut evidence,
            maybe_failure: &mut maybe_failure,
        },
    );

    // Assert: the claim-specific pause quorum is complete before the next
    // request; an unavailable optional log marker must not erase that proof.
    assert!(evidence.pause_confirmed);
    assert_eq!(evidence.dismiss_request_count, 1);
    assert_eq!(
        maybe_failure,
        Some(CampaignTerminalCategory::CommandRequestFailed)
    );
}

#[test]
fn duplicate_identify_generation_cannot_satisfy_the_original_claim() {
    // Arrange
    let (_temp, root) = private_root();
    let target = trusted_target();
    let mut sample = SystemInfoWire::from_snapshot(&ApiSnapshot::safe_ultra_205());
    sample.mining_paused = true;
    sample.mining_activity = "paused".to_owned();
    let mut tracker = CommandStatusTracker::default();
    tracker.record_display_availability(true, 0);
    tracker.record_command(CommandStatusEffect::IdentifyEnable {
        expires_at_uptime_ms: 30_000,
    });
    tracker.record_command(CommandStatusEffect::IdentifyEnable {
        expires_at_uptime_ms: 31_000,
    });
    tracker.record_display(
        DisplayFrameKind::Identify,
        DisplayRenderOutcome::Rendered,
        1_000,
    );
    let status = command_status(&tracker, &sample, true);
    let mut phase = CommandPhase::ProgrammaticIdentifyRendered;
    let mut generations = CommandGenerations {
        identify: 1,
        ..CommandGenerations::default()
    };
    let mut evidence = CommandEffectsEvidence::new();
    let mut maybe_block_count = None;
    let mut maybe_failure = None;
    let mut serial = SharedSerialState::default();
    serial.command_transitions.identify_generation = 2;
    serial.command_transitions.display_identify_generation = 2;
    let websocket = serial.command_transitions;
    let idle_http = StrictHttpClient::new("http://127.0.0.1:9").expect("idle HTTP client");

    // Act
    advance_programmatic_commands(
        &idle_http,
        &target,
        &root,
        &sample,
        &status,
        &serial,
        &websocket,
        true,
        std::time::Instant::now(),
        &mut generations,
        CommandProgress {
            phase: &mut phase,
            maybe_block_count: &mut maybe_block_count,
            evidence: &mut evidence,
            maybe_failure: &mut maybe_failure,
        },
    );

    // Assert
    assert_eq!(phase, CommandPhase::ProgrammaticIdentifyRendered);
    assert!(!evidence.identify_render_receipt_confirmed);
    assert_eq!(maybe_failure, None);
}
