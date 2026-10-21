//! Firmware publishes `stopping` while a hardware safe stop is in progress. The
//! accumulator waits through it: it is neither active evidence nor a terminal stop.
use super::*;

fn stopping_sample(revision: u64, sequence: u64) -> SystemInfoWire {
    let mut sample = active_sample(revision, sequence);
    sample.mining_paused = true;
    sample.mining_activity = "stopping".to_owned();
    sample
}

#[test]
fn stopping_after_initial_activity_is_a_wait_not_lost_activity() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    accumulator.record_active_sample(NetworkTransport::Http, 1_000, 1_000, &active_sample(1, 1));

    // Act
    accumulator.record_active_sample(NetworkTransport::Http, 2_000, 2_000, &stopping_sample(2, 2));

    // Assert
    assert_eq!(accumulator.maybe_failure, None);
    assert_eq!(accumulator.http_success_count, 1);
}

#[test]
fn stopping_at_the_evidence_horizon_neither_confirms_nor_fails_the_terminal_state() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    accumulator.record_active_sample(NetworkTransport::Http, 1_000, 1_000, &active_sample(1, 1));

    // Act
    accumulator.record_active_sample(
        NetworkTransport::Http,
        EVIDENCE_DURATION_MILLIS,
        EVIDENCE_DURATION_MILLIS,
        &stopping_sample(2, 2),
    );

    // Assert
    assert_eq!(accumulator.maybe_failure, None);
    assert!(!accumulator.terminal_http_valid);
}

#[test]
fn stopping_terminal_sample_neither_confirms_nor_fails_the_terminal_state() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());

    // Act
    accumulator.record_terminal_sample(NetworkTransport::WebSocket, &stopping_sample(1, 1));

    // Assert
    assert_eq!(accumulator.maybe_failure, None);
    assert!(!accumulator.terminal_websocket_valid);
}

#[test]
fn the_paused_sample_after_a_stopping_wait_confirms_the_terminal_state() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    accumulator.record_terminal_sample(NetworkTransport::Http, &stopping_sample(1, 1));

    // Act
    accumulator.record_terminal_sample(NetworkTransport::Http, &terminal_sample(2, 2));

    // Assert
    assert_eq!(accumulator.maybe_failure, None);
    assert!(accumulator.terminal_http_valid);
}

#[test]
fn a_stopping_sample_from_another_boot_still_fails_identity() {
    // Arrange
    let mut accumulator = NetworkAccumulator::new(target());
    let mut sample = stopping_sample(1, 1);
    sample.boot_ordinal = 1;

    // Act
    accumulator.record_terminal_sample(NetworkTransport::Http, &sample);

    // Assert
    assert_eq!(
        accumulator.maybe_failure,
        Some(CampaignTerminalCategory::TerminalStateUnconfirmed)
    );
    assert!(!accumulator.terminal_http_valid);
}
