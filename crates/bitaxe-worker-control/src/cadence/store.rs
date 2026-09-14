use super::outcomes::SendOutcomes;
use super::*;
use std::cell::UnsafeCell;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

struct Storage {
    phases: [CadenceSummary; 3],
    maybe_previous_start_us: Option<u64>,
}

impl Storage {
    const fn new() -> Self {
        Self {
            phases: [
                CadenceSummary::empty(CadencePhase::Idle),
                CadenceSummary::empty(CadencePhase::Usb),
                CadenceSummary::empty(CadencePhase::Mining),
            ],
            maybe_previous_start_us: None,
        }
    }

    fn maybe_capturing(&mut self) -> Option<&mut CadenceSummary> {
        self.phases
            .iter_mut()
            .find(|phase| phase.state == CadenceState::Capturing)
    }
}

/// One-attempt atomic ownership; neither capture nor export can wait for another owner.
pub struct CadenceRecorder {
    owned: AtomicBool,
    enabled: AtomicBool,
    active_phase: AtomicU32,
    outcomes: [SendOutcomes; 3],
    dropped: AtomicU32,
    subscribers: AtomicU32,
    storage: UnsafeCell<Storage>,
}

// SAFETY: all storage accesses are made under one acquired CAS and released by an
// uncloneable guard. Storage contains only fixed owned values; references never escape.
unsafe impl Sync for CadenceRecorder {}

struct Ownership<'a>(&'a AtomicBool);
impl Drop for Ownership<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

impl Default for CadenceRecorder {
    fn default() -> Self {
        Self::new()
    }
}

impl CadenceRecorder {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            owned: AtomicBool::new(false),
            enabled: AtomicBool::new(false),
            active_phase: AtomicU32::new(0),
            outcomes: [
                SendOutcomes::new(),
                SendOutcomes::new(),
                SendOutcomes::new(),
            ],
            dropped: AtomicU32::new(0),
            subscribers: AtomicU32::new(0),
            storage: UnsafeCell::new(Storage::new()),
        }
    }

    fn maybe_access<T>(&self, operation: impl FnOnce(&mut Storage) -> T) -> Option<T> {
        if self
            .owned
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_err()
        {
            if self.enabled.load(Ordering::Relaxed) {
                // A single saturated terminal loss bit avoids a retry loop on the recording path.
                self.dropped.store(1, Ordering::Relaxed);
                let phase = self.active_phase.load(Ordering::Acquire);
                if let Some(outcomes) = self.outcomes.get(phase as usize) {
                    outcomes.drop_observation();
                }
            }
            return None;
        }
        let _ownership = Ownership(&self.owned);
        // SAFETY: unique acquired ownership lives until operation returns or unwinds.
        Some(operation(unsafe { &mut *self.storage.get() }))
    }

    /// Arms each phase once per boot in order; no command can erase previously captured evidence.
    pub fn maybe_arm(
        &self,
        phase: CadencePhase,
        at_us: u64,
        generation: u32,
    ) -> Option<CadenceArmReceipt> {
        self.maybe_access(|storage| {
            let index = phase.index();
            if at_us == 0
                || (phase == CadencePhase::Mining && generation == 0)
                || storage.phases[index].state != CadenceState::Empty
                || storage.phases[..index]
                    .iter()
                    .enumerate()
                    .any(|(index, prior)| {
                        prior.state != CadenceState::Complete
                            || self.outcomes[index].maybe_terminal_count()
                                != Some(prior.sends_queued)
                    })
                || storage
                    .maybe_previous_start_us
                    .is_none_or(|previous| previous > at_us)
            {
                return None;
            }
            let summary = &mut storage.phases[index];
            summary.armed_at_us = at_us;
            summary.generation = if phase == CadencePhase::Mining {
                generation
            } else {
                0
            };
            if phase == CadencePhase::Mining {
                summary.state = CadenceState::Armed;
            } else {
                summary.started_at_us = at_us;
                summary.state = CadenceState::Capturing;
            }
            if self.subscribers.load(Ordering::Acquire) != 1 {
                summary.subscriber_mismatch_count = 1;
            }
            // Losing the first dispatch while armed must invalidate the capture;
            // a later dispatch cannot silently replace its start boundary.
            self.active_phase.store(index as u32, Ordering::Release);
            self.enabled.store(true, Ordering::Release);
            Some(CadenceArmReceipt {
                schema: "worker-telemetry-cadence-arm-v1",
                phase,
                armed_at_us: at_us,
                generation: summary.generation,
            })
        })
        .flatten()
    }

    /// Called only after the ASIC write succeeds for the admitted generation.
    pub fn successful_dispatch(&self, generation: u32, at_us: u64) {
        self.maybe_access(|storage| {
            let summary = &mut storage.phases[2];
            if summary.state != CadenceState::Armed || summary.generation != generation {
                return;
            }
            if at_us < summary.armed_at_us {
                increment(&mut summary.clock_failures, &mut summary.overflow);
                return;
            }
            summary.started_at_us = at_us;
            summary.state = CadenceState::Capturing;
            self.enabled.store(true, Ordering::Release);
            if self.subscribers.load(Ordering::Acquire) != 1 {
                increment(
                    &mut summary.subscriber_mismatch_count,
                    &mut summary.overflow,
                );
            }
        });
    }

    /// Owner-side membership transitions also catch a disconnect between loop samples.
    pub fn subscribers_changed(&self, count: u32) {
        self.subscribers.store(count, Ordering::Release);
        self.maybe_access(|storage| {
            if let Some(summary) = storage.maybe_capturing() {
                if count != 1 {
                    increment(
                        &mut summary.subscriber_mismatch_count,
                        &mut summary.overflow,
                    );
                }
            }
        });
    }

    #[must_use]
    pub fn subscriber_count(&self) -> u32 {
        self.subscribers.load(Ordering::Acquire)
    }

    /// Records an entire real iteration, including the interval spanning each capture boundary.
    pub fn iteration(&self, iteration: CadenceIteration) {
        self.maybe_access(|storage| {
            let maybe_previous = storage
                .maybe_previous_start_us
                .replace(iteration.started_at_us);
            let Some(summary) = storage.maybe_capturing() else {
                return;
            };
            if self.subscribers.load(Ordering::Acquire) != 1 {
                increment(
                    &mut summary.subscriber_mismatch_count,
                    &mut summary.overflow,
                );
            }
            if iteration.cpu != 0 {
                increment(&mut summary.cpu_mismatch_count, &mut summary.overflow);
            }
            if iteration.priority != 5 {
                increment(&mut summary.priority_mismatch_count, &mut summary.overflow);
            }
            let valid = maybe_previous.is_some_and(|previous| previous < iteration.started_at_us)
                && iteration.started_at_us <= iteration.live_finished_at_us
                && iteration.live_finished_at_us <= iteration.logs_finished_at_us
                && iteration.logs_finished_at_us <= iteration.finished_at_us;
            if !valid {
                increment(&mut summary.clock_failures, &mut summary.overflow);
                return;
            }
            let previous = maybe_previous.expect("validated previous iteration exists");
            let interval = iteration.started_at_us - previous;
            increment(&mut summary.interval_count, &mut summary.overflow);
            let bucket = if interval <= 500_000 {
                0
            } else if interval <= 750_000 {
                1
            } else if interval <= 1_500_000 {
                2
            } else {
                3
            };
            increment(&mut summary.interval_buckets[bucket], &mut summary.overflow);
            summary.maximum_interval_us = summary.maximum_interval_us.max(interval);
            summary.maximum_execution_us = summary
                .maximum_execution_us
                .max(iteration.finished_at_us - iteration.started_at_us);
            summary.maximum_live_us = summary
                .maximum_live_us
                .max(iteration.live_finished_at_us - iteration.started_at_us);
            summary.maximum_logs_us = summary
                .maximum_logs_us
                .max(iteration.logs_finished_at_us - iteration.live_finished_at_us);
            summary.maximum_prune_us = summary
                .maximum_prune_us
                .max(iteration.finished_at_us - iteration.logs_finished_at_us);
            if iteration
                .started_at_us
                .saturating_sub(summary.started_at_us)
                >= CAPTURE_DURATION_US
            {
                summary.ended_at_us = iteration.finished_at_us;
                summary.state = CadenceState::Complete;
                self.enabled.store(false, Ordering::Release);
            }
        });
    }

    /// Witnesses an exact maximum request and prepared response inside the USB load window.
    /// This proves Controller preparation only; the paired host receipt must prove delivery.
    pub fn max_probe_prepared(&self, request_bytes: usize, response_bytes: usize, at_us: u64) {
        if request_bytes != crate::serial::MAXIMUM_CONTROL_PAYLOAD_BYTES
            || response_bytes != crate::serial::MAXIMUM_CONTROL_PAYLOAD_BYTES
        {
            return;
        }
        self.maybe_access(|storage| {
            let summary = &mut storage.phases[CadencePhase::Usb.index()];
            if summary.state != CadenceState::Capturing
                || at_us < summary.started_at_us
                || at_us - summary.started_at_us >= CAPTURE_DURATION_US
            {
                return;
            }
            if summary.max_probe_count == 0 {
                summary.first_max_probe_at_us = at_us;
            }
            if at_us < summary.last_max_probe_at_us {
                increment(&mut summary.clock_failures, &mut summary.overflow);
                return;
            }
            summary.last_max_probe_at_us = at_us;
            increment(&mut summary.max_probe_count, &mut summary.overflow);
        });
    }

    pub fn publication(&self, outcome: CadencePublication) {
        self.maybe_access(|storage| {
            let Some(summary) = storage.maybe_capturing() else {
                return;
            };
            let counter = match outcome {
                CadencePublication::Projected => &mut summary.projection_count,
                CadencePublication::Unchanged => &mut summary.unchanged_count,
                CadencePublication::NoSubscribers => &mut summary.no_subscriber_count,
                CadencePublication::ProjectionFailed => &mut summary.projection_failures,
                CadencePublication::SerializationFailed => &mut summary.serialization_failures,
            };
            increment(counter, &mut summary.overflow);
        });
    }

    /// Registers pending completion before enqueue, so an immediately running callback is safe.
    pub fn maybe_begin_send(&self) -> Option<CadenceSendToken> {
        self.maybe_access(|storage| {
            let summary = storage.maybe_capturing()?;
            increment(&mut summary.sends_queued, &mut summary.overflow);
            Some(CadenceSendToken(summary.phase.index()))
        })
        .flatten()
    }

    pub fn queue_failed(&self, token: CadenceSendToken) {
        self.outcomes[token.0].queue_failed();
    }

    /// The exact original phase receives every terminal callback without the timing-store guard.
    pub fn send_completed(&self, token: CadenceSendToken, successful: bool) {
        self.outcomes[token.0].completed(successful);
    }

    #[must_use]
    pub fn snapshot(&self) -> CadenceSnapshot {
        let maybe_phases = self.maybe_access(|storage| storage.phases);
        let dropped = self.dropped.load(Ordering::Acquire);
        let mut phases = maybe_phases.unwrap_or_else(|| Storage::new().phases);
        for (index, phase) in phases.iter_mut().enumerate() {
            self.outcomes[index].project(phase);
            phase.refresh_passed(self.outcomes[index].dropped());
        }
        CadenceSnapshot {
            schema: "worker-telemetry-cadence-v1",
            snapshot_available: maybe_phases.is_some(),
            dropped_observations: dropped,
            storage_bytes: std::mem::size_of::<Self>(),
            phases,
        }
    }
}

fn increment(counter: &mut u32, overflow: &mut bool) {
    match counter.checked_add(1) {
        Some(value) => *counter = value,
        None => *overflow = true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn armed_idle() -> CadenceRecorder {
        let recorder = CadenceRecorder::new();
        recorder.subscribers_changed(1);
        recorder.iteration(CadenceIteration {
            started_at_us: 1,
            live_finished_at_us: 11,
            logs_finished_at_us: 11,
            finished_at_us: 11,
            cpu: 0,
            priority: 5,
        });
        recorder.maybe_arm(CadencePhase::Idle, 100, 0).expect("arm");
        recorder
    }

    #[test]
    fn successful_callbacks_survive_timing_writer_contention_without_false_pending() {
        // Arrange
        let recorder = armed_idle();
        let first = recorder.maybe_begin_send().expect("queued");
        let second = recorder.maybe_begin_send().expect("queued");
        for _ in 0..180 {
            let token = recorder.maybe_begin_send().expect("queued");
            recorder.send_completed(token, true);
        }
        assert_eq!(recorder.snapshot().phases[0].pending_sends, 2);
        // Act
        recorder
            .maybe_access(|_| {
                std::thread::scope(|scope| {
                    scope
                        .spawn(|| {
                            recorder.send_completed(first, true);
                            recorder.send_completed(second, true);
                        })
                        .join()
                        .expect("completion worker");
                });
            })
            .expect("timing writer owns capture storage");
        // Assert
        let snapshot = recorder.snapshot();
        assert_eq!(snapshot.dropped_observations, 0);
        assert_eq!(snapshot.phases[0].sends_queued, 182);
        assert_eq!(snapshot.phases[0].sends_completed, 182);
        assert_eq!(snapshot.phases[0].pending_sends, 0);
        assert_eq!(snapshot.phases[0].send_failures, 0);
    }

    #[test]
    fn failed_callback_is_retained_while_timing_writer_is_busy() {
        // Arrange
        let recorder = armed_idle();
        let token = recorder.maybe_begin_send().expect("queued");
        // Act
        recorder.owned.store(true, Ordering::Release);
        recorder.send_completed(token, false);
        recorder.owned.store(false, Ordering::Release);
        // Assert
        let snapshot = recorder.snapshot();
        assert_eq!(snapshot.dropped_observations, 0);
        assert_eq!(snapshot.phases[0].send_failures, 1);
        assert_eq!(snapshot.phases[0].sends_completed, 1);
        assert_eq!(snapshot.phases[0].pending_sends, 0);
        assert!(!snapshot.phases[0].passed);
    }

    #[test]
    fn queue_rejection_is_retained_while_timing_writer_is_busy() {
        // Arrange
        let recorder = armed_idle();
        let token = recorder.maybe_begin_send().expect("queued");
        // Act
        recorder.owned.store(true, Ordering::Release);
        recorder.queue_failed(token);
        recorder.owned.store(false, Ordering::Release);
        // Assert
        let snapshot = recorder.snapshot();
        assert_eq!(snapshot.dropped_observations, 0);
        assert_eq!(snapshot.phases[0].queue_failures, 1);
        assert_eq!(snapshot.phases[0].sends_queued, 0);
        assert_eq!(snapshot.phases[0].pending_sends, 0);
        assert_eq!(snapshot.phases[0].sends_completed, 0);
    }

    #[test]
    fn impossible_duplicate_terminal_outcomes_remain_explicit_failures() {
        // Arrange
        let recorder = armed_idle();
        let token = recorder.maybe_begin_send().expect("queued");
        // Act
        recorder.queue_failed(token);
        recorder.send_completed(token, true);
        // Assert
        let phase = recorder.snapshot().phases[0];
        assert!(phase.overflow);
        assert!(!phase.passed);
    }

    #[test]
    fn later_phase_loss_cannot_rewrite_a_frozen_prior_phase() {
        // Arrange
        let recorder = armed_idle();
        for interval in 1..=120 {
            let started_at_us = 100 + interval * 500_000;
            recorder.iteration(CadenceIteration {
                started_at_us,
                live_finished_at_us: started_at_us + 10,
                logs_finished_at_us: started_at_us + 10,
                finished_at_us: started_at_us + 10,
                cpu: 0,
                priority: 5,
            });
        }
        let idle = serde_json::to_value(recorder.snapshot().phases[0]).expect("summary");
        assert_eq!(idle["passed"], true);
        recorder
            .maybe_arm(CadencePhase::Usb, 61_000_000, 0)
            .expect("next phase");
        // Act
        recorder.owned.store(true, Ordering::Release);
        recorder.publication(CadencePublication::Projected);
        recorder.owned.store(false, Ordering::Release);
        // Assert
        let snapshot = recorder.snapshot();
        assert_eq!(snapshot.dropped_observations, 1);
        assert_eq!(
            serde_json::to_value(snapshot.phases[0]).expect("summary"),
            idle
        );
        assert!(!snapshot.phases[1].passed);
    }

    #[test]
    fn contended_recorder_reports_loss_without_waiting() {
        // Arrange
        let recorder = CadenceRecorder::new();
        recorder.enabled.store(true, Ordering::Release);
        recorder.owned.store(true, Ordering::Release);
        // Act
        recorder.publication(CadencePublication::Projected);
        recorder.owned.store(false, Ordering::Release);
        // Assert
        assert_eq!(recorder.snapshot().dropped_observations, 1);
    }

    #[test]
    fn contended_first_dispatch_cannot_be_replaced_by_a_later_passing_capture() {
        // Arrange
        let recorder = CadenceRecorder::new();
        recorder.subscribers_changed(1);
        recorder.maybe_access(|storage| {
            storage.phases[0].state = CadenceState::Complete;
            storage.phases[1].state = CadenceState::Complete;
            storage.maybe_previous_start_us = Some(1);
        });
        recorder
            .maybe_arm(CadencePhase::Mining, 100, 7)
            .expect("arm");

        // Act
        recorder.owned.store(true, Ordering::Release);
        recorder.successful_dispatch(7, 200);
        recorder.owned.store(false, Ordering::Release);
        recorder.successful_dispatch(7, 300);
        for interval in 1..=120 {
            let started_at_us = 300 + interval * 500_000;
            recorder.iteration(CadenceIteration {
                started_at_us,
                live_finished_at_us: started_at_us + 10,
                logs_finished_at_us: started_at_us + 10,
                finished_at_us: started_at_us + 10,
                cpu: 0,
                priority: 5,
            });
        }

        // Assert
        let snapshot = recorder.snapshot();
        let mining = snapshot.phases[2];
        assert_eq!(mining.state, CadenceState::Complete);
        assert_eq!(mining.started_at_us, 300);
        assert_eq!(mining.interval_count, 120);
        assert_eq!(snapshot.dropped_observations, 1);
        assert!(!mining.passed);
    }

    #[test]
    fn counter_overflow_is_terminal_and_does_not_wrap() {
        // Arrange
        let recorder = CadenceRecorder::new();
        recorder.maybe_access(|storage| {
            storage.phases[0].state = CadenceState::Capturing;
            storage.phases[0].projection_count = u32::MAX;
        });
        // Act
        recorder.publication(CadencePublication::Projected);
        // Assert
        let phase = recorder.snapshot().phases[0];
        assert!(phase.overflow);
        assert_eq!(phase.projection_count, u32::MAX);
    }
}
