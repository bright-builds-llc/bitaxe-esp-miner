use super::*;

fn tick(recorder: &CadenceRecorder, start: u64, work: u64) {
    recorder.iteration(CadenceIteration {
        started_at_us: start,
        live_finished_at_us: start + work,
        logs_finished_at_us: start + work,
        finished_at_us: start + work,
        cpu: 0,
        priority: 5,
        live_stages: LiveStageMeasurements {
            durations_us: [0; LIVE_STAGE_COUNT],
            complete: true,
        },
    });
}
fn seeded() -> CadenceRecorder {
    let recorder = CadenceRecorder::new();
    recorder.subscribers_changed(1);
    tick(&recorder, 1, 10);
    recorder
}
fn capture(recorder: &CadenceRecorder, phase: CadencePhase, start: u64) {
    recorder.maybe_arm(phase, start, 7).expect("valid arm");
    if phase == CadencePhase::Mining {
        recorder.successful_dispatch(7, start);
    }
    if phase == CadencePhase::Usb {
        recorder.max_probe_prepared(65536, 65536, start);
    }
    for count in 1..=121 {
        let now = start + count * 500_000;
        if phase == CadencePhase::Usb && count % 10 == 0 && count < 120 {
            recorder.max_probe_prepared(65536, 65536, now);
        }
        tick(recorder, now, 20);
    }
}

#[test]
fn bounded_three_phase_storage_and_complete_capture() {
    // Arrange
    let recorder = seeded();
    // Act
    capture(&recorder, CadencePhase::Idle, 100);
    capture(&recorder, CadencePhase::Usb, 60_500_101);
    capture(&recorder, CadencePhase::Mining, 121_000_102);
    let snapshot = recorder.snapshot();
    // Assert
    assert!(snapshot.storage_bytes + 4 <= 2048);
    assert!(snapshot.phases.iter().all(|phase| phase.passed));
    assert_eq!(snapshot.phases[2].generation, 7);
}

#[test]
fn boundary_stall_is_included_and_rejects_capture() {
    // Arrange
    let recorder = seeded();
    recorder.maybe_arm(CadencePhase::Idle, 100, 0).expect("arm");
    for count in 1..=119 {
        tick(&recorder, 100 + count * 500_000, 20);
    }
    // Act
    tick(&recorder, 62_000_000, 20);
    let phase = recorder.snapshot().phases[0];
    // Assert
    assert_eq!(phase.state, CadenceState::Complete);
    assert_eq!(phase.maximum_interval_us, 2_499_900);
    assert!(!phase.passed);
}

#[test]
fn queue_completion_is_required_even_after_capture_boundary() {
    // Arrange
    let recorder = seeded();
    recorder.maybe_arm(CadencePhase::Idle, 100, 0).expect("arm");
    let token = recorder.maybe_begin_send().expect("active send");
    for count in 1..=120 {
        tick(&recorder, 100 + count * 500_000, 20);
    }
    // Act
    let pending = recorder.snapshot().phases[0];
    recorder.send_completed(token, true);
    let completed = recorder.snapshot().phases[0];
    // Assert
    assert!(!pending.passed);
    assert_eq!(pending.pending_sends, 1);
    assert!(completed.passed);
    assert_eq!(completed.sends_completed, completed.sends_queued);
}

#[test]
fn asynchronous_failure_remains_bound_to_original_phase() {
    // Arrange
    let recorder = seeded();
    recorder.maybe_arm(CadencePhase::Idle, 100, 0).expect("arm");
    let token = recorder.maybe_begin_send().expect("active send");
    for count in 1..=120 {
        tick(&recorder, 100 + count * 500_000, 20);
    }
    // Act
    recorder.send_completed(token, false);
    // Assert
    let phase = recorder.snapshot().phases[0];
    assert_eq!(phase.send_failures, 1);
    assert!(!phase.passed);
}

#[test]
fn queue_failure_is_distinct_from_completed_send() {
    // Arrange
    let recorder = seeded();
    recorder.maybe_arm(CadencePhase::Idle, 100, 0).expect("arm");
    let token = recorder.maybe_begin_send().expect("active send");
    // Act
    recorder.queue_failed(token);
    // Assert
    let phase = recorder.snapshot().phases[0];
    assert_eq!(phase.queue_failures, 1);
    assert_eq!(phase.pending_sends, 0);
    assert_eq!(phase.sends_queued, 0);
    assert_eq!(phase.sends_completed, 0);
}

#[test]
fn mining_ignores_wrong_generation_then_starts_after_success() {
    // Arrange
    let recorder = seeded();
    capture(&recorder, CadencePhase::Idle, 100);
    capture(&recorder, CadencePhase::Usb, 60_500_101);
    recorder
        .maybe_arm(CadencePhase::Mining, 121_000_102, 7)
        .expect("arm");
    // Act
    recorder.successful_dispatch(8, 122_000_000);
    let wrong = recorder.snapshot().phases[2];
    recorder.successful_dispatch(7, 123_000_000);
    let right = recorder.snapshot().phases[2];
    // Assert
    assert_eq!(wrong.state, CadenceState::Armed);
    assert_eq!(right.state, CadenceState::Capturing);
    assert_eq!(right.started_at_us, 123_000_000);
}

#[test]
fn frozen_capture_cannot_be_rearmed_or_erased() {
    // Arrange
    let recorder = seeded();
    capture(&recorder, CadencePhase::Idle, 100);
    // Act
    let result = recorder.maybe_arm(CadencePhase::Idle, 70_000_000, 0);
    // Assert
    assert!(result.is_none());
    assert_eq!(recorder.snapshot().phases[0].armed_at_us, 100);
}

#[test]
fn incomplete_capture_never_passes_even_after_host_time_has_elapsed() {
    // Arrange
    let recorder = seeded();
    recorder.maybe_arm(CadencePhase::Idle, 100, 0).expect("arm");
    // Act
    tick(&recorder, 500_100, 20);
    // Assert
    let phase = recorder.snapshot().phases[0];
    assert_eq!(phase.state, CadenceState::Capturing);
    assert!(!phase.passed);
}

#[test]
fn subscriber_disconnect_between_iterations_is_retained() {
    // Arrange
    let recorder = seeded();
    recorder.maybe_arm(CadencePhase::Idle, 100, 0).expect("arm");
    // Act
    recorder.subscribers_changed(0);
    recorder.subscribers_changed(1);
    // Assert
    assert_eq!(recorder.snapshot().phases[0].subscriber_mismatch_count, 1);
}

#[test]
fn clock_reversal_is_explicit() {
    // Arrange
    let recorder = seeded();
    recorder.maybe_arm(CadencePhase::Idle, 100, 0).expect("arm");
    // Act
    tick(&recorder, 0, 20);
    // Assert
    assert_eq!(recorder.snapshot().phases[0].clock_failures, 1);
}

#[test]
fn unchanged_and_absent_subscribers_are_not_serialization_failures() {
    // Arrange
    let recorder = seeded();
    recorder.maybe_arm(CadencePhase::Idle, 100, 0).expect("arm");
    // Act
    recorder.publication(CadencePublication::Unchanged);
    recorder.publication(CadencePublication::NoSubscribers);
    recorder.publication(CadencePublication::SerializationFailed);
    // Assert
    let phase = recorder.snapshot().phases[0];
    assert_eq!(phase.unchanged_count, 1);
    assert_eq!(phase.no_subscriber_count, 1);
    assert_eq!(phase.serialization_failures, 1);
}

struct FakeLoop {
    now: u64,
    durations: [u64; 3],
    cpu: u32,
    priority: u32,
}
impl CadenceLoopIo for FakeLoop {
    fn now_us(&self) -> u64 {
        self.now
    }
    fn cpu(&self) -> u32 {
        self.cpu
    }
    fn priority(&self) -> u32 {
        self.priority
    }
    fn live(&mut self) -> LiveStageMeasurements {
        self.now += self.durations[0];
        let mut durations_us = [0; LIVE_STAGE_COUNT];
        durations_us[0] = self.durations[0];
        LiveStageMeasurements {
            durations_us,
            complete: true,
        }
    }
    fn logs(&mut self) {
        self.now += self.durations[1];
    }
    fn prune(&mut self) {
        self.now += self.durations[2];
    }
}

#[test]
fn production_iteration_measures_each_real_stage_and_total_processing() {
    // Arrange
    let recorder = seeded();
    recorder.maybe_arm(CadencePhase::Idle, 100, 0).expect("arm");
    let mut io = FakeLoop {
        now: 500_100,
        durations: [20, 700_000, 30],
        cpu: 0,
        priority: 5,
    };
    // Act
    run_iteration(&mut io, &recorder);
    // Assert
    let phase = recorder.snapshot().phases[0];
    assert_eq!(phase.maximum_live_us, 20);
    assert_eq!(phase.maximum_logs_us, 700_000);
    assert_eq!(phase.maximum_prune_us, 30);
    assert_eq!(phase.maximum_execution_us, 700_050);
}

#[test]
fn production_iteration_records_wrong_cpu_and_priority() {
    // Arrange
    let recorder = seeded();
    recorder.maybe_arm(CadencePhase::Idle, 100, 0).expect("arm");
    let mut io = FakeLoop {
        now: 500_100,
        durations: [20, 30, 40],
        cpu: 1,
        priority: 4,
    };
    // Act
    run_iteration(&mut io, &recorder);
    // Assert
    let phase = recorder.snapshot().phases[0];
    assert_eq!(phase.cpu_mismatch_count, 1);
    assert_eq!(phase.priority_mismatch_count, 1);
}

#[test]
fn fixed_interval_buckets_include_exact_edges() {
    // Arrange
    let recorder = seeded();
    recorder.maybe_arm(CadencePhase::Idle, 2, 0).expect("arm");
    let mut start = 1;
    // Act
    for interval in [500_000, 500_001, 750_000, 750_001, 1_500_000, 1_500_001] {
        start += interval;
        tick(&recorder, start, 20);
    }
    // Assert
    assert_eq!(recorder.snapshot().phases[0].interval_buckets, [1, 2, 2, 1]);
}

#[test]
fn max_probe_witnesses_bind_real_load_to_device_capture_window() {
    // Arrange
    let recorder = seeded();
    capture(&recorder, CadencePhase::Idle, 100);
    let start = 60_500_101;
    recorder
        .maybe_arm(CadencePhase::Usb, start, 0)
        .expect("arm");
    // Act
    for index in 0..12 {
        recorder.max_probe_prepared(65536, 65536, start + index * 5_000_000);
    }
    // Assert
    let phase = recorder.snapshot().phases[1];
    assert_eq!(phase.max_probe_count, 12);
    assert_eq!(phase.first_max_probe_at_us, start);
    assert_eq!(phase.last_max_probe_at_us, start + 55_000_000);
    assert_eq!(recorder.snapshot().phases[0].max_probe_count, 0);
}

#[test]
fn delayed_probes_after_device_capture_deadline_cannot_prove_usb_load() {
    // Arrange
    let recorder = seeded();
    capture(&recorder, CadencePhase::Idle, 100);
    let start = 60_500_101;
    recorder
        .maybe_arm(CadencePhase::Usb, start, 0)
        .expect("arm");
    for index in 1..120 {
        tick(&recorder, start + index * 500_000, 20);
    }
    // Act
    // The final boundary interval is still pending, but the 60-second load window is closed.
    for index in 0..12 {
        recorder.max_probe_prepared(65536, 65536, start + 60_000_000 + index);
    }
    tick(&recorder, start + 60_000_012, 20);
    // Assert
    let phase = recorder.snapshot().phases[1];
    assert_eq!(phase.state, CadenceState::Complete);
    assert_eq!(phase.max_probe_count, 0);
    assert!(!phase.passed);
}

#[test]
fn short_or_unmatched_probe_payloads_do_not_count_as_maximum_load() {
    // Arrange
    let recorder = seeded();
    capture(&recorder, CadencePhase::Idle, 100);
    let start = 60_500_101;
    recorder
        .maybe_arm(CadencePhase::Usb, start, 0)
        .expect("arm");
    // Act
    recorder.max_probe_prepared(65535, 65536, start + 1);
    recorder.max_probe_prepared(65536, 65535, start + 2);
    recorder.max_probe_prepared(65537, 65536, start + 3);
    // Assert
    assert_eq!(recorder.snapshot().phases[1].max_probe_count, 0);
}

#[test]
fn next_phase_waits_for_late_completion_using_terminal_counters() {
    // Arrange
    let recorder = seeded();
    recorder.maybe_arm(CadencePhase::Idle, 100, 0).expect("arm");
    let token = recorder.maybe_begin_send().expect("queued");
    for index in 1..=120 {
        tick(&recorder, 100 + index * 500_000, 20);
    }
    assert!(recorder
        .maybe_arm(CadencePhase::Usb, 61_000_000, 0)
        .is_none());
    // Act
    recorder.send_completed(token, true);
    let next = recorder.maybe_arm(CadencePhase::Usb, 61_000_000, 0);
    // Assert
    assert!(next.is_some());
    assert_eq!(recorder.snapshot().phases[0].pending_sends, 0);
    assert!(recorder.snapshot().phases[0].passed);
}

#[test]
fn worst_interval_joins_the_previous_iterations_work_not_the_current_stages() {
    // Arrange
    let recorder = seeded();
    recorder.maybe_arm(CadencePhase::Idle, 100, 0).expect("arm");
    let mut slow = FakeLoop {
        now: 500_100,
        durations: [260_000, 10, 20],
        cpu: 0,
        priority: 5,
    };
    run_iteration(&mut slow, &recorder);
    let mut fast = FakeLoop {
        now: slow.now + 500_000,
        durations: [1_000, 10, 20],
        cpu: 0,
        priority: 5,
    };
    // Act
    run_iteration(&mut fast, &recorder);
    // Assert
    let phase = recorder.snapshot().phases[0];
    assert_eq!(phase.maximum_interval_us, 760_030);
    assert_eq!(phase.worst_interval.previous_execution_us, 260_030);
    assert_eq!(phase.worst_interval.previous_live_stages_us[0], 260_000);
    assert_eq!(phase.worst_interval.gap_us, 500_000);
    assert_eq!(phase.maximum_live_stages_us[0], 260_000);
}

#[test]
fn unprofiled_previous_boundary_does_not_invalidate_a_complete_current_iteration() {
    // Arrange
    let recorder = CadenceRecorder::new();
    recorder.iteration(CadenceIteration {
        started_at_us: 1,
        live_finished_at_us: 11,
        logs_finished_at_us: 11,
        finished_at_us: 11,
        cpu: 0,
        priority: 5,
        live_stages: LiveStageMeasurements {
            durations_us: [0; LIVE_STAGE_COUNT],
            complete: false,
        },
    });
    recorder.subscribers_changed(1);
    recorder.maybe_arm(CadencePhase::Idle, 100, 0).expect("arm");
    // Act
    tick(&recorder, 500_100, 20);
    // Assert
    let phase = recorder.snapshot().phases[0];
    assert_eq!(phase.clock_failures, 0);
    assert_eq!(phase.interval_count, 1);
    assert_eq!(phase.worst_interval.previous_execution_us, 10);
    assert_eq!(
        phase.worst_interval.previous_live_stages_us,
        [0; LIVE_STAGE_COUNT]
    );
}

#[test]
fn missing_current_profile_is_explicitly_invalid() {
    // Arrange
    let recorder = seeded();
    recorder.maybe_arm(CadencePhase::Idle, 100, 0).expect("arm");
    // Act
    recorder.iteration(CadenceIteration {
        started_at_us: 500_100,
        live_finished_at_us: 500_110,
        logs_finished_at_us: 500_110,
        finished_at_us: 500_110,
        cpu: 0,
        priority: 5,
        live_stages: LiveStageMeasurements {
            durations_us: [0; LIVE_STAGE_COUNT],
            complete: false,
        },
    });
    // Assert
    let phase = recorder.snapshot().phases[0];
    assert_eq!(phase.clock_failures, 1);
    assert!(!phase.passed);
}

#[test]
fn absent_subscribers_are_not_misreported_as_a_profile_clock_failure() {
    // Arrange
    let recorder = seeded();
    recorder.subscribers_changed(0);
    recorder.maybe_arm(CadencePhase::Idle, 100, 0).expect("arm");
    // Act
    recorder.publication(CadencePublication::NoSubscribers);
    tick(&recorder, 500_100, 20);
    // Assert
    let phase = recorder.snapshot().phases[0];
    assert_eq!(phase.no_subscriber_count, 1);
    assert!(phase.subscriber_mismatch_count > 0);
    assert_eq!(phase.clock_failures, 0);
    assert!(!phase.passed);
}
