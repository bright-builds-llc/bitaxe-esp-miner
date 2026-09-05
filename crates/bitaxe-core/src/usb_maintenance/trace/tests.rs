use super::*;
use crate::usb_worker_diagnostics::WorkerDiagnosticReplay;

fn ready_state() -> TracedUsbMaintenanceState {
    let mut state = TracedUsbMaintenanceState::default();
    state.observe(
        MaintenanceEvent::LineState {
            dtr: true,
            rts: false,
        },
        10,
    );
    state.observe(MaintenanceEvent::LineCoding { bit_rate: 1_200 }, 20);
    state.observe(MaintenanceEvent::SafeStopComplete, 30);
    state
}

#[test]
fn identical_ready_callback_disarm_is_distinct_from_expiry() {
    // Arrange
    let mut state = ready_state();

    // Act
    state.observe(MaintenanceEvent::LineCoding { bit_rate: 1_200 }, 40);
    let marker = state.maybe_trace_marker(3).expect("disarmed trace");

    // Assert
    assert!(marker.contains(
        "event=coding_1200 before=ready after=idle action=none expired=false remaining_ms=4970"
    ));
}

#[test]
fn late_commit_identifies_deadline_expiry() {
    // Arrange
    let mut state = ready_state();

    // Act
    state.observe(MaintenanceEvent::LineCoding { bit_rate: 115_200 }, 5_011);
    let marker = state.maybe_trace_marker(3).expect("expired trace");

    // Assert
    assert!(marker.contains(
        "event=coding_115200 before=ready after=idle action=none expired=true remaining_ms=0"
    ));
}

#[test]
fn commit_enqueue_failure_enables_diagnostics_without_changing_committed_protocol_state() {
    // Arrange
    let mut state = ready_state();
    state.observe(MaintenanceEvent::LineCoding { bit_rate: 115_200 }, 40);
    assert!(!state.diagnostics_allowed());

    // Act
    state.record_effect(
        MaintenanceTraceEffect::CommitEnqueue,
        MaintenanceTraceOutcome::PartialWrite,
        41,
    );

    // Assert
    assert!(state.diagnostics_allowed());
    let marker = state.maybe_trace_marker(4).expect("failure trace");
    assert!(marker.contains("event=commit_enqueue before=committed after=committed"));
    assert!(marker.contains("outcome=partial_write"));
}

#[test]
fn ordinary_observer_survives_arm_expiry_and_emits_a_finite_report() {
    // Arrange
    let mut state = TracedUsbMaintenanceState::default();
    let mut replay = WorkerDiagnosticReplay::default();
    state.observe(MaintenanceEvent::LineCoding { bit_rate: 115_200 }, 0);
    replay.line_coding(115_200, 0);
    state.observe(
        MaintenanceEvent::LineState {
            dtr: true,
            rts: false,
        },
        1,
    );
    replay.line_state(true, 1);

    // Act / Assert
    assert_eq!(replay.maybe_due_slot(2, state.diagnostics_allowed()), None);
    state.expire(5_001);
    assert_eq!(
        replay.maybe_due_slot(5_101, state.diagnostics_allowed()),
        Some(0)
    );
    assert!(state
        .maybe_trace_marker(2)
        .expect("expiry trace")
        .contains("event=expiry"));
}

#[test]
fn ready_and_committed_success_cannot_emit_trace() {
    // Arrange
    let mut state = ready_state();

    // Act / Assert
    assert_eq!(state.maybe_trace_marker(0), None);
    state.record_effect(
        MaintenanceTraceEffect::ReadyEnqueue,
        MaintenanceTraceOutcome::Ok,
        31,
    );
    assert!(!state.diagnostics_allowed());
    state.observe(MaintenanceEvent::LineCoding { bit_rate: 115_200 }, 40);
    state.record_effect(
        MaintenanceTraceEffect::CommitEnqueue,
        MaintenanceTraceOutcome::Ok,
        41,
    );
    assert!(!state.diagnostics_allowed());
}

#[test]
fn trace_ring_keeps_only_sixteen_closed_records_without_growing() {
    // Arrange
    let mut state = TracedUsbMaintenanceState::default();

    // Act
    for now in 1..=30 {
        state.record_effect(
            MaintenanceTraceEffect::QueueLoss,
            MaintenanceTraceOutcome::None,
            now,
        );
    }

    // Assert
    assert!(state
        .maybe_trace_marker(0)
        .expect("oldest")
        .contains("seq=15 "));
    assert!(state
        .maybe_trace_marker(15)
        .expect("newest")
        .contains("seq=30 "));
    assert_eq!(state.maybe_trace_marker(16), None);
    assert!(std::mem::size_of::<TracedUsbMaintenanceState>() <= 512);
}

#[test]
fn failed_attempt_trace_survives_a_fresh_ordinary_observer() {
    // Arrange
    let mut state = ready_state();
    state.observe(MaintenanceEvent::LineCoding { bit_rate: 1_200 }, 40);
    let mut replay = WorkerDiagnosticReplay::default();

    // Act
    state.observe(MaintenanceEvent::LineCoding { bit_rate: 115_200 }, 100);
    replay.line_coding(115_200, 100);
    state.observe(
        MaintenanceEvent::LineState {
            dtr: true,
            rts: false,
        },
        101,
    );
    replay.line_state(true, 101);
    assert_eq!(
        replay.maybe_due_slot(102, state.diagnostics_allowed()),
        None
    );
    state.expire(5_101);

    // Assert
    assert_eq!(
        replay.maybe_due_slot(5_201, state.diagnostics_allowed()),
        Some(0)
    );
    assert!(state
        .maybe_trace_marker(3)
        .expect("prior failure retained")
        .contains("before=ready after=idle"));
}
