use super::*;
use std::sync::{Arc, Barrier};

#[test]
fn observations_are_first_wins_and_all_numeric_values_are_retained_exactly() {
    // Arrange
    let cache = BootDiagnosticCache::new();
    // Act
    assert_eq!(
        cache.record_checkpoint(Checkpoint::UsbInstall, 10, 11, 98304),
        RecordOutcome::Recorded
    );
    assert_eq!(
        cache.record_checkpoint(Checkpoint::UsbInstall, 99, 98, 42),
        RecordOutcome::AlreadyRecorded
    );
    // Assert: these historical source measurements are separate heap queries, not a synthetic coherent pair.
    assert_eq!(cache.maybe_checkpoint_marker(Checkpoint::UsbInstall).as_deref(), Some("usb_memory_checkpoint stage=usb_install free_bytes=10 largest_block_bytes=11 reserve_bytes=98304 redacted=true"));
}

#[test]
fn all_checkpoint_and_failure_wire_labels_remain_exact_and_closed() {
    // Arrange
    let cache = BootDiagnosticCache::new();
    // Act / Assert
    for checkpoint in [
        Checkpoint::UsbInstall,
        Checkpoint::UsbInstalled,
        Checkpoint::WifiDriverPrepare,
        Checkpoint::WifiDriverPrepared,
        Checkpoint::WorkerOwnerPrepare,
        Checkpoint::StatisticsStart,
        Checkpoint::StatisticsStarted,
    ] {
        assert_eq!(
            Checkpoint::maybe_from_label(checkpoint.label()),
            Some(checkpoint)
        );
        assert_eq!(
            cache.record_checkpoint(checkpoint, 0, 0, u32::MAX as usize),
            RecordOutcome::Recorded
        );
        assert_eq!(cache.maybe_checkpoint_marker(checkpoint), Some(format!("usb_memory_checkpoint stage={} free_bytes=0 largest_block_bytes=0 reserve_bytes=4294967295 redacted=true", checkpoint.label())));
    }
    for invalid in [
        "",
        "usb_install extra=private",
        "stage=usb_install",
        "private-network",
        "usb_install\n",
    ] {
        assert_eq!(Checkpoint::maybe_from_label(invalid), None);
    }
    for failure in [
        StartFailure::OwnerSpawn,
        StartFailure::UsbInstall,
        StartFailure::ControlOwner,
    ] {
        let cache = BootDiagnosticCache::new();
        assert!(cache.record_failure(failure));
        assert_eq!(
            cache.maybe_failure_marker(),
            Some(format!(
                "bwg_worker_start_failure category=startup_failed detail={} redacted=true",
                failure.label()
            ))
        );
    }
}

#[test]
fn first_failure_cannot_be_replaced() {
    // Arrange
    let cache = BootDiagnosticCache::new();
    // Act
    assert!(cache.record_failure(StartFailure::UsbInstall));
    assert!(!cache.record_failure(StartFailure::OwnerSpawn));
    // Assert
    assert_eq!(
        cache.failure(),
        Observation::Available(StartFailure::UsbInstall)
    );
}

#[test]
fn unavailable_partial_and_corrupt_states_never_emit_a_marker() {
    // Arrange
    let cache = BootDiagnosticCache::new();
    let slot = &cache.checkpoints[Checkpoint::UsbInstall as usize];
    // Act / Assert
    assert_eq!(
        cache.checkpoint(Checkpoint::UsbInstall),
        Observation::Unavailable
    );
    assert!(cache
        .maybe_checkpoint_marker(Checkpoint::UsbInstall)
        .is_none());
    slot.state.store(WRITING, Ordering::Relaxed);
    slot.free.store(123, Ordering::Relaxed);
    assert_eq!(
        cache.checkpoint(Checkpoint::UsbInstall),
        Observation::InProgress
    );
    assert!(cache
        .maybe_checkpoint_marker(Checkpoint::UsbInstall)
        .is_none());
    slot.state.store(99, Ordering::Relaxed);
    cache.failure.store(99, Ordering::Relaxed);
    assert_eq!(
        cache.checkpoint(Checkpoint::UsbInstall),
        Observation::Corrupt
    );
    assert_eq!(cache.failure(), Observation::Corrupt);
    assert!(cache
        .maybe_checkpoint_marker(Checkpoint::UsbInstall)
        .is_none());
    assert!(cache.maybe_failure_marker().is_none());
}

#[cfg(target_pointer_width = "64")]
#[test]
fn out_of_range_input_is_explicitly_corrupt_and_never_truncated() {
    // Arrange
    let cache = BootDiagnosticCache::new();
    // Act
    let outcome = cache.record_checkpoint(Checkpoint::UsbInstall, u32::MAX as usize + 1, 0, 98304);
    // Assert
    assert_eq!(outcome, RecordOutcome::InvalidObservation);
    assert_eq!(
        cache.checkpoint(Checkpoint::UsbInstall),
        Observation::Corrupt
    );
    assert!(cache
        .maybe_checkpoint_marker(Checkpoint::UsbInstall)
        .is_none());
    assert_eq!(
        cache.record_checkpoint(Checkpoint::UsbInstall, 1, 1, 1),
        RecordOutcome::AlreadyRecorded
    );
}

#[test]
fn racing_writers_publish_one_coherent_observation() {
    // Arrange
    let cache = Arc::new(BootDiagnosticCache::new());
    let barrier = Arc::new(Barrier::new(3));
    let workers: Vec<_> = [11, 22]
        .into_iter()
        .map(|value| {
            let cache = Arc::clone(&cache);
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                cache.record_checkpoint(Checkpoint::UsbInstall, value, value + 1, value + 2)
            })
        })
        .collect();
    // Act
    barrier.wait();
    let outcomes: Vec<_> = workers
        .into_iter()
        .map(|worker| worker.join().expect("writer exits"))
        .collect();
    // Assert
    assert_eq!(
        outcomes
            .iter()
            .filter(|&&o| o == RecordOutcome::Recorded)
            .count(),
        1
    );
    let Observation::Available(value) = cache.checkpoint(Checkpoint::UsbInstall) else {
        panic!("one complete observation");
    };
    assert!(value.free_bytes == 11 || value.free_bytes == 22);
    assert_eq!(value.largest_block_bytes, value.free_bytes + 1);
    assert_eq!(value.reserve_bytes, value.free_bytes + 2);
}

#[test]
fn cache_static_storage_is_exactly_116_bytes() {
    assert_eq!(std::mem::size_of::<BootDiagnosticCache>(), 116);
}

#[test]
fn error_context_classification_never_retains_private_text() {
    // Arrange / Act / Assert
    for (context, expected) in [
        ("owner_spawn: private-fixture", StartFailure::OwnerSpawn),
        ("usb_install: private-fixture", StartFailure::UsbInstall),
        ("unrecognized private-fixture", StartFailure::ControlOwner),
    ] {
        let cache = BootDiagnosticCache::new();
        assert_eq!(StartFailure::from_context(context), expected);
        assert!(cache.record_failure(StartFailure::from_context(context)));
        assert!(!cache
            .maybe_failure_marker()
            .expect("closed failure")
            .contains("private-fixture"));
    }
}
