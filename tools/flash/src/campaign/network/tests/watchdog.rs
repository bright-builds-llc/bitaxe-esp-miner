use super::super::watchdog::{sample_failure, WatchdogFailure};
use super::*;

fn window_watchdog_failure(
    http_checkpoint: [u64; 2],
    http_feed: [u64; 2],
    websocket_checkpoint: [u64; 2],
    websocket_feed: [u64; 2],
) -> WatchdogFailure {
    let mut accumulator = NetworkAccumulator::new(target());
    for sample_index in 0..2 {
        let revision = u64::try_from(sample_index + 1).expect("sample index fits");
        let sample = active_sample_with_watchdog_sequences(
            revision,
            http_checkpoint[sample_index],
            http_feed[sample_index],
        );
        accumulator.record_active_sample(NetworkTransport::Http, 1_000, 1_000, &sample);
        let sample = active_sample_with_watchdog_sequences(
            revision,
            websocket_checkpoint[sample_index],
            websocket_feed[sample_index],
        );
        accumulator.record_active_sample(NetworkTransport::WebSocket, 1_000, 1_000, &sample);
    }
    accumulator.close_elapsed_windows(WINDOW_MILLIS, &complete_serial());
    accumulator.watchdog_failure
}

#[test]
fn stale_watchdog_sample_is_rejected_before_window_credit() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    let mut sample = active_sample(1, 1);
    sample.runtime_health.maybe_task_watchdog_feed_age_millis = Some(2_001);

    // Act
    accumulator.record_active_sample(NetworkTransport::Http, 1_000, 1_000, &sample);

    // Assert
    assert_eq!(
        accumulator.maybe_failure,
        Some(CampaignTerminalCategory::WatchdogUnresponsive)
    );
    assert_eq!(
        accumulator.watchdog_failure,
        WatchdogFailure::WatchdogFeedStale
    );
}

#[test]
fn evaluator_watchdog_reasons_keep_one_production_shaped_failure() {
    // Arrange
    let cases = [
        (None, "unavailable", WatchdogFailure::WatchdogReasonMissing),
        (
            Some("unproved"),
            "unavailable",
            WatchdogFailure::WatchdogUnproved,
        ),
        (
            Some("invalid_observation"),
            "not_participating",
            WatchdogFailure::WatchdogInvalidObservation,
        ),
        (
            Some("subscription_failed"),
            "not_participating",
            WatchdogFailure::WatchdogSubscriptionFailed,
        ),
        (
            Some("feed_failed"),
            "not_participating",
            WatchdogFailure::WatchdogFeedFailed,
        ),
        (
            Some("unsubscription_failed"),
            "not_participating",
            WatchdogFailure::WatchdogUnsubscriptionFailed,
        ),
        (
            Some("unsubscribed"),
            "not_participating",
            WatchdogFailure::WatchdogUnsubscribed,
        ),
        (
            Some("feed_stale"),
            "not_participating",
            WatchdogFailure::WatchdogFeedStale,
        ),
        (
            Some("future_reason"),
            "not_participating",
            WatchdogFailure::WatchdogReasonUnknown,
        ),
        (
            Some("feed_fresh"),
            "not_participating",
            WatchdogFailure::WatchdogParticipationInconsistent,
        ),
    ];

    // Act / Assert
    for (maybe_reason, participation, expected) in cases {
        let mut sample = active_sample(1, 1);
        sample.runtime_health.maybe_task_watchdog_reason = maybe_reason.map(str::to_owned);
        sample.runtime_health.task_watchdog_participation = participation.to_owned();
        assert_eq!(sample_failure(&sample), expected);
    }
}

#[test]
fn every_remaining_watchdog_sample_predicate_has_one_closed_failure() {
    // Arrange
    let mut cases = Vec::new();
    let mut sample = active_sample(1, 1);
    sample.runtime_health.supervisor_availability = "unavailable".to_owned();
    cases.push((WatchdogFailure::SupervisorUnavailable, sample));
    let mut sample = active_sample(1, 1);
    sample.runtime_health.checkpoint_health = "stale".to_owned();
    cases.push((WatchdogFailure::CheckpointUnhealthy, sample));
    let mut sample = active_sample(1, 1);
    sample.runtime_health.maybe_checkpoint_sequence = None;
    cases.push((WatchdogFailure::CheckpointSequenceMissing, sample));
    let mut sample = active_sample(1, 1);
    sample.runtime_health.maybe_task_watchdog_feed_sequence = None;
    cases.push((WatchdogFailure::WatchdogFeedSequenceMissing, sample));
    let mut sample = active_sample(1, 1);
    sample.runtime_health.maybe_task_watchdog_feed_age_millis = None;
    cases.push((WatchdogFailure::WatchdogFeedAgeMissing, sample));
    let mut sample = active_sample(1, 1);
    sample.runtime_health.maybe_task_watchdog_feed_age_millis = Some(2_001);
    cases.push((WatchdogFailure::WatchdogFeedStale, sample));

    // Act / Assert
    for (expected, sample) in cases {
        assert_eq!(sample_failure(&sample), expected);
    }
    assert_eq!(sample_failure(&active_sample(1, 1)), WatchdogFailure::None);
}

#[test]
fn reason_failure_precedes_participation_and_feed_detail_checks() {
    // Arrange
    let mut sample = active_sample(1, 1);
    sample.runtime_health.task_watchdog_participation = "not_participating".to_owned();
    sample.runtime_health.maybe_task_watchdog_reason = Some("feed_failed".to_owned());
    sample.runtime_health.maybe_task_watchdog_feed_sequence = None;
    sample.runtime_health.maybe_task_watchdog_feed_age_millis = None;

    // Act
    let failure = sample_failure(&sample);

    // Assert
    assert_eq!(failure, WatchdogFailure::WatchdogFeedFailed);
}

#[test]
fn every_transport_window_watchdog_boundary_has_one_closed_failure() {
    // Arrange
    let advancing = [1, 2];
    let stagnant = [7, 7];
    let cases = [
        (
            WatchdogFailure::HttpCheckpointNotAdvanced,
            stagnant,
            advancing,
            advancing,
            advancing,
        ),
        (
            WatchdogFailure::HttpFeedNotAdvanced,
            advancing,
            stagnant,
            advancing,
            advancing,
        ),
        (
            WatchdogFailure::WebsocketCheckpointNotAdvanced,
            advancing,
            advancing,
            stagnant,
            advancing,
        ),
        (
            WatchdogFailure::WebsocketFeedNotAdvanced,
            advancing,
            advancing,
            advancing,
            stagnant,
        ),
    ];

    // Act / Assert
    for (expected, http_checkpoint, http_feed, websocket_checkpoint, websocket_feed) in cases {
        assert_eq!(
            window_watchdog_failure(
                http_checkpoint,
                http_feed,
                websocket_checkpoint,
                websocket_feed,
            ),
            expected,
        );
    }
}

#[test]
fn later_watchdog_observations_preserve_the_earliest_watchdog_failure() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    let mut stale = active_sample(1, 1);
    stale.runtime_health.maybe_task_watchdog_feed_age_millis = Some(2_001);
    accumulator.record_active_sample(NetworkTransport::Http, 1_000, 1_000, &stale);
    let mut unavailable = terminal_sample(2, 2);
    unavailable.runtime_health.supervisor_availability = "unavailable".to_owned();

    // Act
    accumulator.record_terminal_sample(NetworkTransport::Http, &unavailable);

    // Assert
    assert_eq!(
        accumulator.watchdog_failure,
        WatchdogFailure::WatchdogFeedStale
    );
}

#[test]
fn later_watchdog_observation_does_not_reclassify_an_earlier_non_watchdog_failure() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    accumulator.fail(CampaignTerminalCategory::HttpWindowIncomplete);
    let mut unavailable = terminal_sample(2, 2);
    unavailable.runtime_health.supervisor_availability = "unavailable".to_owned();

    // Act
    accumulator.record_terminal_sample(NetworkTransport::Http, &unavailable);

    // Assert
    assert_eq!(
        accumulator.maybe_failure,
        Some(CampaignTerminalCategory::HttpWindowIncomplete)
    );
    assert_eq!(accumulator.watchdog_failure, WatchdogFailure::None);
}

#[test]
fn watchdog_and_checkpoint_sequences_must_advance_within_each_window() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    for (revision, active_ms) in [(1, 1_000), (2, 2_000)] {
        let sample = active_sample(revision, 7);
        accumulator.record_active_sample(NetworkTransport::Http, active_ms, active_ms, &sample);
        accumulator.record_active_sample(
            NetworkTransport::WebSocket,
            active_ms,
            active_ms,
            &sample,
        );
    }
    let serial = complete_serial();

    // Act
    accumulator.close_elapsed_windows(WINDOW_MILLIS, &serial);

    // Assert
    assert_eq!(
        accumulator.maybe_failure,
        Some(CampaignTerminalCategory::WatchdogUnresponsive)
    );
}

#[test]
fn failed_network_evidence_serializes_only_the_closed_watchdog_label() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    let mut sample = active_sample(1, 1);
    sample.runtime_health.maybe_task_watchdog_feed_age_millis = Some(2_001);
    accumulator.record_active_sample(NetworkTransport::Http, 1_000, 1_000, &sample);

    // Act
    let evidence = accumulator.finish(&SharedSerialState::default());
    let encoded = serde_json::to_string(&evidence).expect("evidence should serialize");

    // Assert
    assert_eq!(evidence.watchdog_failure, "watchdog_feed_stale");
    assert!(encoded.contains("\"watchdog_failure\":\"watchdog_feed_stale\""));
    assert!(!encoded.contains("2001"));
}
