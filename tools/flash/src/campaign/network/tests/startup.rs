use super::*;

fn startup_sample(revision: u64, sequence: u64) -> SystemInfoWire {
    let mut sample = active_sample(revision, sequence);
    sample.mining_activity = "safe_blocked".to_owned();
    sample
}

#[test]
fn startup_transitions_are_uncredited_before_all_twenty_windows_complete() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    accumulator.record_active_sample(NetworkTransport::Http, 500, 500, &startup_sample(99, 99));
    accumulator.record_active_sample(
        NetworkTransport::WebSocket,
        750,
        750,
        &startup_sample(98, 98),
    );
    record_complete_windows(&mut accumulator);
    let terminal = terminal_sample(100, 100);
    accumulator.record_terminal_sample(NetworkTransport::Http, &terminal);
    accumulator.record_terminal_sample(NetworkTransport::WebSocket, &terminal);

    // Act
    let evidence = accumulator.finish(&complete_serial());

    // Assert
    assert_eq!(evidence.status, "accepted");
    assert_eq!(evidence.covered_window_count, REQUIRED_WINDOWS);
    assert_eq!(evidence.http_success_count, 40);
    assert_eq!(evidence.websocket_frame_count, 40);
    assert_eq!(evidence.http_startup_transition_count, 1);
    assert_eq!(evidence.websocket_startup_transition_count, 1);
    assert!(evidence.http_initial_active_observed);
    assert!(evidence.websocket_initial_active_observed);
    assert_eq!(evidence.maximum_http_gap_ms, WINDOW_MILLIS);
    assert_eq!(evidence.maximum_websocket_gap_ms, WINDOW_MILLIS);
}

#[test]
fn unresolved_http_startup_transition_fails_when_window_zero_closes() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    accumulator.record_active_sample(
        NetworkTransport::Http,
        29_999,
        29_999,
        &startup_sample(1, 1),
    );
    for (active_ms, sequence) in [(1_000, 1), (2_000, 2)] {
        accumulator.record_active_sample(
            NetworkTransport::WebSocket,
            active_ms,
            active_ms,
            &active_sample(sequence, sequence),
        );
    }
    let serial = complete_serial();

    // Act
    accumulator.close_elapsed_windows(29_999, &serial);
    let before_boundary = accumulator.maybe_failure;
    accumulator.record_active_sample(NetworkTransport::Http, 30_000, 30_000, &active_sample(2, 2));
    accumulator.close_elapsed_windows(30_000, &serial);

    // Assert
    assert_eq!(before_boundary, None);
    assert_eq!(
        accumulator.maybe_failure,
        Some(CampaignTerminalCategory::HttpWindowIncomplete)
    );
}

#[test]
fn unresolved_websocket_startup_transition_fails_when_window_zero_closes() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    for (active_ms, sequence) in [(1_000, 1), (2_000, 2)] {
        accumulator.record_active_sample(
            NetworkTransport::Http,
            active_ms,
            active_ms,
            &active_sample(sequence, sequence),
        );
    }
    accumulator.record_active_sample(
        NetworkTransport::WebSocket,
        29_999,
        29_999,
        &startup_sample(3, 3),
    );

    // Act
    accumulator.close_elapsed_windows(30_000, &complete_serial());

    // Assert
    assert_eq!(
        accumulator.maybe_failure,
        Some(CampaignTerminalCategory::WebsocketWindowIncomplete)
    );
}

#[test]
fn transports_establish_independently_and_later_state_regression_fails() {
    for transport in [NetworkTransport::Http, NetworkTransport::WebSocket] {
        // Arrange
        let mut accumulator = NetworkAccumulator::new(target());
        let other = match transport {
            NetworkTransport::Http => NetworkTransport::WebSocket,
            NetworkTransport::WebSocket => NetworkTransport::Http,
        };
        accumulator.record_active_sample(other, 500, 500, &startup_sample(1, 1));
        accumulator.record_active_sample(transport, 1_000, 1_000, &active_sample(2, 2));

        // Act
        accumulator.record_active_sample(transport, 2_000, 2_000, &startup_sample(3, 3));

        // Assert
        assert_eq!(
            accumulator.maybe_failure,
            Some(CampaignTerminalCategory::NetworkCorrelationFailed)
        );
    }
}

#[test]
fn invalid_startup_prerequisites_remain_fail_fast() {
    let mut wrong_identity = startup_sample(1, 1);
    wrong_identity.source_commit = "1".repeat(40);
    let mut unsafe_sample = startup_sample(1, 1);
    unsafe_sample.power = 16.0;
    let mut stale_watchdog = startup_sample(1, 1);
    stale_watchdog
        .runtime_health
        .maybe_task_watchdog_feed_age_millis = Some(2_001);
    let cases = [
        (
            wrong_identity,
            CampaignTerminalCategory::NetworkCorrelationFailed,
        ),
        (
            unsafe_sample,
            CampaignTerminalCategory::NetworkCorrelationFailed,
        ),
        (
            stale_watchdog,
            CampaignTerminalCategory::WatchdogUnresponsive,
        ),
    ];

    for (sample, expected) in cases {
        // Arrange
        let mut accumulator = NetworkAccumulator::new(target());

        // Act
        accumulator.record_active_sample(NetworkTransport::Http, 1_000, 1_000, &sample);

        // Assert
        assert_eq!(accumulator.maybe_failure, Some(expected));
        assert_eq!(accumulator.http_success_count, 0);
    }
}

#[test]
fn startup_transitions_do_not_request_recovery_pause() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());

    // Act
    accumulator.record_active_sample(NetworkTransport::Http, 1_000, 1_000, &startup_sample(1, 1));
    let transition_request = accumulator.take_recovery_pause_request();
    accumulator.record_active_sample(NetworkTransport::Http, 2_000, 2_000, &active_sample(2, 2));
    accumulator.record_active_sample(NetworkTransport::Http, 3_000, 3_000, &startup_sample(3, 3));
    let failure_request = accumulator.take_recovery_pause_request();
    let duplicate_request = accumulator.take_recovery_pause_request();

    // Assert
    assert!(!transition_request);
    assert!(failure_request);
    assert!(!duplicate_request);
    assert_eq!(accumulator.recovery_pause_request_count, 1);
}
