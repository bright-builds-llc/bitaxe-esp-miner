use std::time::Duration;

use bitaxe_api::{ApiSnapshot, OperatorSnapshotRevision, SystemInfoWire};
use serde_json::json;

use super::{advances, apply_live_frame, ReconnectBackoff};

#[test]
fn reconnect_backoff_is_one_two_four_then_five_seconds() {
    // Arrange
    let mut backoff = ReconnectBackoff::new();

    // Act
    let delays = [(); 6].map(|()| backoff.take_delay());

    // Assert
    assert_eq!(
        delays,
        [
            Duration::from_secs(1),
            Duration::from_secs(2),
            Duration::from_secs(4),
            Duration::from_secs(5),
            Duration::from_secs(5),
            Duration::from_secs(5),
        ]
    );
}

#[test]
fn successful_connection_resets_reconnect_backoff() {
    // Arrange
    let mut backoff = ReconnectBackoff::new();
    assert_eq!(backoff.take_delay(), Duration::from_secs(1));
    assert_eq!(backoff.take_delay(), Duration::from_secs(2));

    // Act
    backoff.reset();

    // Assert
    assert_eq!(backoff.take_delay(), Duration::from_secs(1));
}

#[test]
fn full_connect_frame_and_nested_diff_reconstruct_one_coherent_snapshot() {
    // Arrange
    let mut full = SystemInfoWire::from_snapshot(&ApiSnapshot::safe_ultra_205());
    full.operator_snapshot_revision = OperatorSnapshotRevision::new(1).expect("nonzero revision");
    full.runtime_health.maybe_task_watchdog_feed_sequence = Some(10);
    let full_frame = serde_json::to_vec(&json!({
        "event": "update",
        "data": full,
    }))
    .expect("full frame");
    let diff_frame = serde_json::to_vec(&json!({
        "event": "update",
        "data": {
            "operatorSnapshotRevision": 2,
            "runtimeHealth": {
                "taskWatchdogFeedSequence": 11,
            },
        },
    }))
    .expect("diff frame");
    let mut projection = None;

    // Act
    let first = apply_live_frame(&full_frame, &mut projection).expect("full snapshot");
    let second = apply_live_frame(&diff_frame, &mut projection).expect("merged snapshot");

    // Assert
    assert_eq!(first.operator_snapshot_revision.get(), 1);
    assert_eq!(second.operator_snapshot_revision.get(), 2);
    assert_eq!(
        second.runtime_health.maybe_task_watchdog_feed_sequence,
        Some(11)
    );
}

#[test]
fn partial_first_frame_is_rejected_without_a_projection() {
    // Arrange
    let frame = serde_json::to_vec(&json!({
        "event": "update",
        "data": { "operatorSnapshotRevision": 2 },
    }))
    .expect("partial frame");
    let mut projection = None;

    // Act
    let maybe_sample = apply_live_frame(&frame, &mut projection);

    // Assert
    assert!(maybe_sample.is_none());
}

#[test]
fn a_sequence_advances_only_when_it_appears_or_grows() {
    // Arrange / Act
    let cases = [
        advances(None, Some(1)),
        advances(Some(1), Some(2)),
        advances(Some(2), Some(2)),
        advances(None, None),
    ];
    // Assert
    assert_eq!(cases, [true, true, false, false]);
}
