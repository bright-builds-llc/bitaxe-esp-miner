use super::*;

fn correlation(epoch: u32) -> SerialTraceCorrelation {
    SerialTraceCorrelation {
        epoch,
        request_sequence: 7,
    }
}

#[test]
fn successful_output_survives_fresh_hello_without_becoming_new_session_authority() {
    // Arrange
    let trace = SerialTrace::new();
    trace.record(
        correlation(1),
        SerialTraceStage::WriterCompleted,
        20,
        900,
        900,
    );
    // Act
    trace.record(
        SerialTraceCorrelation {
            epoch: 2,
            request_sequence: 0,
        },
        SerialTraceStage::Hello,
        3000,
        0,
        0,
    );
    let snapshot = trace.snapshot();
    // Assert
    let previous = snapshot.previous.expect("prior epoch retained");
    assert_eq!(previous.events.len(), 1);
    assert_eq!(previous.events[0].stage, SerialTraceStage::WriterCompleted);
    assert_eq!(previous.epoch, 1);
    assert_eq!(snapshot.current.epoch, 2);
    assert_eq!(snapshot.current.next_event_ordinal, 2);
}

#[test]
fn overwritten_window_is_bounded_and_reports_the_missing_prefix() {
    // Arrange
    let trace = SerialTrace::new();
    // Act
    for time in 0..65 {
        trace.record(
            correlation(1),
            SerialTraceStage::WriterQueued,
            time,
            100,
            100,
        );
    }
    let snapshot = trace.snapshot();
    // Assert
    assert_eq!(snapshot.current.events.len(), 64);
    assert_eq!(snapshot.current.first_event_ordinal, 2);
    assert_eq!(snapshot.current.next_event_ordinal, 66);
    assert_eq!(snapshot.current.overwritten_events, 1);
}

#[test]
fn a_busy_export_never_blocks_capture_and_reports_the_dropped_event() {
    // Arrange
    let trace = SerialTrace::new();
    let held = trace.buffer.try_lock().expect("unowned fixture");
    // Act
    trace.record(correlation(1), SerialTraceStage::Validated, 1, 200, 0);
    let unavailable = trace.snapshot();
    drop(held);
    let snapshot = trace.snapshot();
    // Assert
    assert!(!unavailable.snapshot_available);
    assert!(unavailable.current.events.is_empty());
    assert_eq!(snapshot.dropped_events, 1);
    assert!(snapshot.current.events.is_empty());
    assert_eq!(snapshot.current.first_event_ordinal, 1);
}

#[test]
fn recorder_contention_cannot_block_or_veto_admitted_dispatch() {
    // Arrange
    let trace = SerialTrace::new();
    let held = trace.buffer.try_lock().expect("held observer storage");
    // Act
    let admitted = trace.admit_dispatch(correlation(1), 1, 1, 10, 300);
    drop(held);
    // Assert
    assert!(admitted);
    assert_eq!(trace.snapshot().dropped_events, 1);
}

#[test]
fn invalid_counts_are_reported_as_loss() {
    // Arrange
    let trace = SerialTrace::new();
    // Act
    trace.record(correlation(1), SerialTraceStage::WriterQueued, 1, 100, 101);
    // Assert
    let snapshot = trace.snapshot();
    assert_eq!(snapshot.dropped_events, 1);
    assert!(snapshot.current.events.is_empty());
}

#[test]
fn unrepresentable_time_is_reported_as_loss() {
    // Arrange
    let trace = SerialTrace::new();
    // Act
    trace.record(
        correlation(1),
        SerialTraceStage::WriterQueued,
        MAXIMUM_SAFE_TIMESTAMP + 1,
        100,
        100,
    );
    // Assert
    let snapshot = trace.snapshot();
    assert_eq!(snapshot.dropped_events, 1);
    assert!(snapshot.current.events.is_empty());
}

#[test]
fn ordinal_exhaustion_never_wraps_and_export_never_clears_history() {
    // Arrange
    let trace = SerialTrace::new();
    trace.record(correlation(1), SerialTraceStage::Hello, 0, 0, 0);
    {
        let mut held = trace.buffer.try_lock().expect("fixture");
        let current = held.current;
        held.slots[current].next_ordinal = u32::MAX - 1;
    }
    // Act
    trace.record(correlation(1), SerialTraceStage::Validated, 1, 100, 0);
    trace.record(correlation(1), SerialTraceStage::Enqueued, 2, 100, 0);
    let snapshot = trace.snapshot();
    // Assert
    assert_eq!(snapshot.current.events.len(), 2);
    assert_eq!(snapshot.current.next_event_ordinal, u32::MAX);
    assert_eq!(snapshot.dropped_events, 1);
    assert_eq!(trace.snapshot().current.events[1].ordinal, u32::MAX - 1);
}

#[test]
fn new_admission_and_export_traffic_cannot_erase_the_previous_epoch_tail() {
    // Arrange
    let trace = SerialTrace::new();
    trace.record(
        correlation(1),
        SerialTraceStage::WriterQueued,
        10,
        1200,
        512,
    );
    trace.record(correlation(2), SerialTraceStage::Hello, 20, 0, 0);
    // Act
    for time in 21..221 {
        trace.record(
            correlation(2),
            SerialTraceStage::WriterQueued,
            time,
            300,
            300,
        );
    }
    trace.record(
        correlation(1),
        SerialTraceStage::WriterAbandoned,
        221,
        1200,
        512,
    );
    let snapshot = trace.snapshot();
    // Assert
    let previous = snapshot.previous.expect("previous epoch");
    assert_eq!(previous.epoch, 1);
    assert_eq!(previous.events.len(), 2);
    assert_eq!(previous.events[1].stage, SerialTraceStage::WriterAbandoned);
    assert_eq!(previous.events[1].queued_bytes, 512);
    assert_eq!(previous.overwritten_events, 0);
    assert!(snapshot.current.overwritten_events > 0);
}

#[test]
fn accepted_queue_record_can_be_rejected_before_production_dispatch_after_replacement() {
    // Arrange
    use std::sync::mpsc;
    let trace = SerialTrace::new();
    let current = AtomicU32::new(1);
    let (sender, receiver) = mpsc::sync_channel(1);
    let request = correlation(1);
    trace.record(request, SerialTraceStage::Validated, 1, 300, 0);
    sender.send(request).expect("real queue accepts request");
    trace.record(request, SerialTraceStage::Enqueued, 2, 300, 0);
    // Act
    current.store(2, Ordering::Release);
    trace.record(correlation(2), SerialTraceStage::Hello, 3, 0, 0);
    let queued = receiver.recv().expect("queued request");
    let dispatched = trace.admit_dispatch(queued, 1, current.load(Ordering::Acquire), 4, 300);
    // Assert
    assert!(!dispatched);
    let previous = trace.snapshot().previous.expect("retained old request");
    assert_eq!(previous.events[2].stage, SerialTraceStage::DispatchRejected);
    assert_eq!(previous.events[2].request_sequence, 7);
    assert!(!previous
        .events
        .iter()
        .any(|event| event.stage == SerialTraceStage::ReplyCreated));
}

#[test]
fn maximum_retained_export_fits_one_control_payload_and_has_bounded_storage() {
    // Arrange
    let trace = SerialTrace::new();
    for epoch in [u32::MAX - 1, u32::MAX] {
        trace.record(correlation(epoch), SerialTraceStage::Hello, 0, 0, 0);
        {
            let mut guard = trace.buffer.try_lock().expect("fixture ownership");
            let current = guard.current;
            guard.slots[current].len = 0;
            guard.slots[current].next_ordinal = u32::MAX - 64;
        }
        for _ in 0..64 {
            trace.record(
                SerialTraceCorrelation {
                    epoch,
                    request_sequence: u32::MAX,
                },
                SerialTraceStage::ValidationRejected,
                MAXIMUM_SAFE_TIMESTAMP,
                66560,
                66560,
            );
        }
    }
    // Act
    let snapshot = trace.snapshot();
    let encoded = serde_json::to_vec(&snapshot).expect("typed snapshot");
    let storage_bytes = std::mem::size_of::<SerialTrace>();
    let event_copy_bytes = 128 * std::mem::size_of::<SerialTraceEvent>();
    // Assert
    assert_eq!(snapshot.current.events.len(), 64);
    assert_eq!(snapshot.previous.expect("previous").events.len(), 64);
    assert!(encoded.len() + 300 < crate::serial::MAXIMUM_CONTROL_PAYLOAD_BYTES);
    assert!(storage_bytes <= 4352);
    assert!(event_copy_bytes <= 4096);
}

#[test]
fn concurrent_capture_retains_coherent_events_and_accounts_for_every_drop() {
    // Arrange
    let trace = SerialTrace::new();
    // Act
    std::thread::scope(|scope| {
        for worker in 0..3 {
            let recorder = &trace;
            scope.spawn(move || {
                for at_ms in 0..1000 {
                    recorder.record(
                        SerialTraceCorrelation {
                            epoch: 1,
                            request_sequence: worker + 1,
                        },
                        SerialTraceStage::WriterQueued,
                        at_ms,
                        1000,
                        512,
                    );
                }
            });
        }
    });
    let snapshot = trace.snapshot();
    // Assert
    assert!(snapshot.snapshot_available);
    assert_eq!(
        snapshot.current.next_event_ordinal - 1 + snapshot.dropped_events,
        3000
    );
    for (index, event) in snapshot.current.events.iter().enumerate() {
        assert_eq!(
            event.ordinal,
            snapshot.current.first_event_ordinal + index as u32
        );
        assert!((1..=3).contains(&event.request_sequence));
        assert_eq!(event.queued_bytes, 512);
        assert_eq!(event.wire_bytes, 1000);
    }
}
