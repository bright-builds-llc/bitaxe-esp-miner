use super::*;

#[test]
fn programmatic_pause_joins_serial_first_and_http_later() {
    // Arrange
    let (_temp, root) = private_root();
    let target = trusted_target();
    let mut sample = SystemInfoWire::from_snapshot(&ApiSnapshot::safe_ultra_205());
    sample.mining_paused = false;
    sample.mining_activity = "active".to_owned();
    sample.block_found = 7;
    let mut tracker = CommandStatusTracker::default();
    let started = std::time::Instant::now();
    let mut phase = CommandPhase::ProgrammaticPause(PauseJoinState::new(started));
    let mut generations = CommandGenerations {
        pause: 1,
        ..CommandGenerations::default()
    };
    let mut evidence = CommandEffectsEvidence::new();
    evidence.pause_request_count = 1;
    let mut maybe_block_count = Some(7);
    let mut maybe_failure = None;
    let mut serial = SharedSerialState {
        resumable_pause_safe_stop_confirmed: true,
        ..SharedSerialState::default()
    };
    let websocket = CommandTransitionWitness::default();
    let idle_http = StrictHttpClient::new("http://127.0.0.1:9").expect("idle HTTP client");

    // Act: the one-shot serial fact arrives before the HTTP status catches up.
    advance_programmatic_commands(
        &idle_http,
        &target,
        &root,
        &sample,
        &command_status(&tracker, &sample),
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
    tracker.record_command(CommandStatusEffect::Pause);
    sample.mining_paused = true;
    sample.mining_activity = "paused".to_owned();
    serial.resumable_pause_safe_stop_confirmed = false;
    let (dismiss_http, dismiss_server) = successful_command_server();
    advance_programmatic_commands(
        &dismiss_http,
        &target,
        &root,
        &sample,
        &command_status(&tracker, &sample),
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

    // Assert
    assert!(evidence.pause_confirmed);
    assert_eq!(evidence.dismiss_request_count, 1);
    assert_eq!(maybe_failure, None);
    assert_eq!(
        dismiss_server.join().expect("dismiss server"),
        "POST /api/system/blockFound/dismiss HTTP/1.1"
    );
}

fn command_status(
    tracker: &CommandStatusTracker,
    sample: &SystemInfoWire,
) -> bitaxe_api::CommandStatusWire {
    tracker.snapshot(
        "0".repeat(32)
            .parse::<BootSessionId>()
            .expect("boot session"),
        1_000,
        CommandStatusFacts {
            mining_paused: sample.mining_paused,
            mining_activity: &sample.mining_activity,
            identify_active: false,
            block_found: sample.block_found,
            block_notification_visible: sample.show_new_block,
        },
    )
}
