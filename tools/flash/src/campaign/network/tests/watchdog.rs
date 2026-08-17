use super::super::watchdog::{sample_failure, WatchdogFailure};
use super::*;

const CAMPAIGN_WATCHDOG_SOURCE: &str = include_str!("../watchdog.rs");

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
fn producer_classified_fresh_sample_after_legacy_boundary_is_accepted() {
    // Arrange
    for feed_age_millis in [2_001, 5_000] {
        let mut accumulator = NetworkAccumulator::new(target());
        let mut sample = active_sample(1, 1);
        sample.runtime_health.maybe_task_watchdog_feed_age_millis = Some(feed_age_millis);

        // Act
        accumulator.record_active_sample(NetworkTransport::Http, 1_000, 1_000, &sample);

        // Assert
        assert_eq!(accumulator.maybe_failure, None);
        assert_eq!(accumulator.watchdog_failure, WatchdogFailure::None);
    }
}

#[test]
fn campaign_watchdog_has_no_numeric_feed_age_policy() {
    // Arrange / Act
    let feed_age_lines = CAMPAIGN_WATCHDOG_SOURCE
        .lines()
        .filter(|line| line.contains("feed_age_millis"))
        .collect::<Vec<_>>();

    // Assert
    assert!(CAMPAIGN_WATCHDOG_SOURCE.contains("maybe_task_watchdog_feed_age_millis.is_none()"));
    for line in feed_age_lines {
        assert!(
            !line.contains('<') && !line.contains('>'),
            "numeric policy found: {line}"
        );
    }
}

#[test]
fn producer_classified_stale_sample_is_rejected() {
    // Arrange
    let mut sample = active_sample(1, 1);
    sample.runtime_health.maybe_task_watchdog_reason = Some("feed_stale".to_owned());
    sample.runtime_health.task_watchdog_participation = "not_participating".to_owned();
    sample.runtime_health.maybe_task_watchdog_feed_age_millis = Some(5_001);

    // Act
    let failure = sample_failure(&sample);

    // Assert
    assert_eq!(failure, WatchdogFailure::WatchdogFeedStale);
}

#[test]
fn coherent_store_read_failures_have_distinct_watchdog_categories() {
    // Arrange
    let cases = [
        (
            "retry_exhausted",
            "snapshot_retry_exhausted",
            WatchdogFailure::WatchdogSnapshotRetryExhausted,
        ),
        (
            "history_poisoned",
            "snapshot_history_poisoned",
            WatchdogFailure::WatchdogSnapshotHistoryPoisoned,
        ),
    ];

    // Act / Assert
    for (read_outcome, reason, expected) in cases {
        let mut sample = active_sample(1, 1);
        sample.runtime_health.task_watchdog_read_outcome = read_outcome.to_owned();
        sample.runtime_health.maybe_task_watchdog_reason = Some(reason.to_owned());
        sample.runtime_health.task_watchdog_participation = "not_participating".to_owned();
        sample.runtime_health.maybe_task_watchdog_feed_sequence = None;
        sample.runtime_health.maybe_task_watchdog_feed_age_millis = None;
        assert_eq!(sample_failure(&sample), expected);
    }
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
    stale.runtime_health.maybe_task_watchdog_reason = Some("feed_stale".to_owned());
    stale.runtime_health.task_watchdog_participation = "not_participating".to_owned();
    stale.runtime_health.maybe_task_watchdog_feed_age_millis = Some(5_001);
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
fn terminal_sample_cannot_mix_phase_and_wait_into_earliest_unproved_failure() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    let mut unproved = active_sample(1, 1);
    unproved.runtime_health.task_watchdog_read_outcome = "uninitialized".to_owned();
    unproved.runtime_health.maybe_task_watchdog_reason = Some("unproved".to_owned());
    unproved.runtime_health.task_watchdog_participation = "unavailable".to_owned();
    unproved.runtime_health.maybe_task_watchdog_feed_sequence = None;
    unproved.runtime_health.maybe_task_watchdog_feed_age_millis = None;
    unproved.runtime_health.task_watchdog_owner_phase = "unavailable".to_owned();
    unproved.runtime_health.task_watchdog_wait_state = "not_waiting".to_owned();
    accumulator.record_active_sample(NetworkTransport::Http, 1_000, 1_000, &unproved);
    let later_terminal = terminal_sample(2, 2);

    // Act
    accumulator.record_terminal_sample(NetworkTransport::Http, &later_terminal);
    let evidence = accumulator.finish(&complete_serial());

    // Assert
    assert_eq!(evidence.watchdog_failure, "watchdog_unproved");
    assert_eq!(evidence.watchdog_read_outcome, "uninitialized");
    assert_eq!(evidence.watchdog_owner_phase, "unavailable");
    assert_eq!(evidence.watchdog_wait_state, "not_waiting");
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
    sample.runtime_health.maybe_task_watchdog_reason = Some("feed_stale".to_owned());
    sample.runtime_health.task_watchdog_participation = "not_participating".to_owned();
    sample.runtime_health.maybe_task_watchdog_feed_age_millis = Some(5_001);
    accumulator.record_active_sample(NetworkTransport::Http, 1_000, 1_000, &sample);

    // Act
    let evidence = accumulator.finish(&SharedSerialState::default());
    let encoded = serde_json::to_string(&evidence).expect("evidence should serialize");

    // Assert
    assert_eq!(evidence.watchdog_failure, "watchdog_feed_stale");
    assert!(encoded.contains("\"watchdog_failure\":\"watchdog_feed_stale\""));
    assert!(!encoded.contains("5001"));
}

#[test]
fn unknown_watchdog_read_outcome_fails_closed_without_republishing_free_text() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    let mut sample = active_sample(1, 1);
    sample.runtime_health.task_watchdog_read_outcome = "private-read-42".to_owned();

    // Act
    accumulator.record_active_sample(NetworkTransport::Http, 1_000, 1_000, &sample);
    let evidence = accumulator.finish(&complete_serial());

    // Assert
    assert_eq!(evidence.watchdog_failure, "watchdog_read_outcome_unknown");
    assert_eq!(evidence.watchdog_read_outcome, "uninitialized");
    assert_eq!(evidence.watchdog_owner_phase, "unavailable");
    assert_eq!(evidence.watchdog_wait_state, "invalid_observation");
}
