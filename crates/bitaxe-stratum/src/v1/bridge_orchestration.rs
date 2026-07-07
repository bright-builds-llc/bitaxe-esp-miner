//! Pure upstream-parity ASIC bridge orchestration decisions.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/tasks/create_jobs_task.c:60,84-86,183-187`
//!   (timed-dequeue regeneration cadence with fresh extranonce2)
//! - `reference/esp-miner/main/tasks/asic_result_task.c:24-36`
//!   (timeout = continue, never fatal)
//!
//! The shell interprets [`BridgeStep`] values; all priority and cadence
//! decisions live here so the J2c dispatch-starvation regression stays
//! encoded as executable host tests.

use std::time::Instant;

/// Upper bound for a single result poll slice in milliseconds.
///
/// UART RX bytes buffer in the driver ring between slices, so short
/// slices lose nothing while keeping dispatch reachable every iteration.
pub const MAX_POLL_SLICE_MS: u32 = 100;

/// Upstream-parity bridge step decisions.
///
/// There is intentionally no fatal/blocked variant: upstream's result
/// task never exits its loop on timeout
/// (`reference/esp-miner/main/tasks/asic_result_task.c:24-36`), so an
/// illegal "stop mining on timeout" state is unrepresentable.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BridgeStep {
    /// Dispatch queued pool work now (always beats polling — J2c fix).
    Dispatch,
    /// Regenerate work with a fresh extranonce2 and dispatch it.
    Regenerate,
    /// Poll the result path for at most `slice_ms` (<= [`MAX_POLL_SLICE_MS`]).
    Poll { slice_ms: u32 },
    /// Nothing to do this iteration.
    Idle,
}

/// Pure decision core for the live ASIC bridge pump.
///
/// Every time-sensitive method takes `now` as a parameter so tests
/// inject a deterministic clock.
#[derive(Debug)]
pub struct BridgeOrchestrator {
    pending_dispatch: bool,
    listener_armed: bool,
    maybe_last_dispatch_at: Option<Instant>,
    job_interval_ms: u32,
    timeout_streak: u64,
}

impl BridgeOrchestrator {
    #[must_use]
    pub fn new(job_interval_ms: u32) -> Self {
        Self {
            pending_dispatch: false,
            listener_armed: false,
            maybe_last_dispatch_at: None,
            job_interval_ms,
            timeout_streak: 0,
        }
    }

    /// Fresh pool work was queued for dispatch.
    pub fn note_work_queued(&mut self) {
        self.pending_dispatch = true;
    }

    /// The continuous result listener is armed.
    pub fn note_listener_armed(&mut self) {
        self.listener_armed = true;
    }

    /// Queued work was dispatched to the chip at `now`.
    pub fn note_dispatched(&mut self, now: Instant) {
        self.pending_dispatch = false;
        self.maybe_last_dispatch_at = Some(now);
    }

    /// A result poll slice elapsed without data; returns the streak count.
    ///
    /// Timeouts are telemetry, never fatal
    /// (`reference/esp-miner/main/tasks/asic_result_task.c:24-36`).
    pub fn note_poll_timeout(&mut self) -> u64 {
        self.timeout_streak += 1;
        self.timeout_streak
    }

    /// A result was read and correlated; the cadence continues.
    pub fn note_result_received(&mut self) {
        self.timeout_streak = 0;
    }

    /// The pool session was invalidated; clears the redispatch timer.
    ///
    /// The listener stays armed — session churn must not stop polling.
    pub fn invalidate_session(&mut self) {
        self.pending_dispatch = false;
        self.maybe_last_dispatch_at = None;
        self.timeout_streak = 0;
    }

    /// Decide the next bridge step at `now`.
    ///
    /// Priority: Dispatch > Regenerate > Poll > Idle. Dispatch is checked
    /// before any poll state — this ordering is the J2c fix.
    ///
    /// Regeneration cadence mirrors upstream's timed dequeue
    /// (`reference/esp-miner/main/tasks/create_jobs_task.c:60,84-86,183-187`).
    #[must_use]
    pub fn next_step(&self, now: Instant) -> BridgeStep {
        if self.pending_dispatch {
            return BridgeStep::Dispatch;
        }

        let Some(last_dispatch_at) = self.maybe_last_dispatch_at else {
            return BridgeStep::Idle;
        };

        let elapsed_ms = now.saturating_duration_since(last_dispatch_at).as_millis();
        if elapsed_ms >= u128::from(self.job_interval_ms) {
            return BridgeStep::Regenerate;
        }

        if self.listener_armed {
            return BridgeStep::Poll {
                slice_ms: MAX_POLL_SLICE_MS,
            };
        }

        BridgeStep::Idle
    }
}

/// Redaction-safe marker for a cadence-driven work regeneration.
#[must_use]
pub fn job_redispatched_marker(counter: u64) -> String {
    format!("asic_bridge=job_redispatched extranonce2_counter={counter}")
}

/// Redaction-safe marker for consecutive result poll timeouts.
#[must_use]
pub fn timeout_streak_marker(count: u64) -> String {
    format!("asic_bridge=timeout_streak count={count}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    const JOB_INTERVAL_MS: u32 = 2000;

    fn armed_orchestrator() -> BridgeOrchestrator {
        let mut orchestrator = BridgeOrchestrator::new(JOB_INTERVAL_MS);
        orchestrator.note_listener_armed();
        orchestrator
    }

    #[test]
    fn dispatch_beats_poll_when_work_queued_and_listener_armed() {
        // Arrange: the literal J2c regression — listener armed AND work queued.
        let mut orchestrator = armed_orchestrator();
        let t0 = Instant::now();
        orchestrator.note_dispatched(t0);
        orchestrator.note_work_queued();

        // Act
        let step = orchestrator.next_step(t0 + Duration::from_millis(50));

        // Assert
        assert_eq!(step, BridgeStep::Dispatch);
    }

    #[test]
    fn cadence_polls_before_interval_and_regenerates_at_interval() {
        // Arrange
        let mut orchestrator = armed_orchestrator();
        let t0 = Instant::now();
        orchestrator.note_work_queued();
        orchestrator.note_dispatched(t0);

        // Act
        let step_before_interval = orchestrator.next_step(t0 + Duration::from_millis(1999));
        let step_at_interval = orchestrator.next_step(t0 + Duration::from_millis(2000));

        // Assert
        assert!(matches!(step_before_interval, BridgeStep::Poll { .. }));
        assert_eq!(step_at_interval, BridgeStep::Regenerate);
    }

    #[test]
    fn poll_timeout_increments_streak_and_cadence_continues() {
        // Arrange
        let mut orchestrator = armed_orchestrator();
        let t0 = Instant::now();
        orchestrator.note_dispatched(t0);

        // Act
        let first_streak = orchestrator.note_poll_timeout();
        let second_streak = orchestrator.note_poll_timeout();
        let step_after_timeouts = orchestrator.next_step(t0 + Duration::from_millis(100));
        let step_at_interval = orchestrator.next_step(t0 + Duration::from_millis(2000));

        // Assert: streak counts up and no fatal variant exists in BridgeStep.
        assert_eq!(first_streak, 1);
        assert_eq!(second_streak, 2);
        assert!(matches!(step_after_timeouts, BridgeStep::Poll { .. }));
        assert_eq!(step_at_interval, BridgeStep::Regenerate);
    }

    #[test]
    fn invalidation_clears_redispatch_timer() {
        // Arrange
        let mut orchestrator = armed_orchestrator();
        let t0 = Instant::now();
        orchestrator.note_dispatched(t0);

        // Act
        orchestrator.invalidate_session();
        let step_long_after = orchestrator.next_step(t0 + Duration::from_secs(10));

        // Assert: no regeneration from a cleared timer; idle until new work.
        assert_ne!(step_long_after, BridgeStep::Regenerate);
        assert_eq!(step_long_after, BridgeStep::Idle);
    }

    #[test]
    fn result_received_keeps_mining_with_no_terminal_state() {
        // Arrange
        let mut orchestrator = armed_orchestrator();
        let t0 = Instant::now();
        orchestrator.note_dispatched(t0);

        // Act: a correlated result (share classified) must not stop the cadence.
        orchestrator.note_result_received();
        let step_soon_after = orchestrator.next_step(t0 + Duration::from_millis(50));
        let step_at_interval = orchestrator.next_step(t0 + Duration::from_millis(2000));

        // Assert
        assert!(matches!(step_soon_after, BridgeStep::Poll { .. }));
        assert_eq!(step_at_interval, BridgeStep::Regenerate);
    }

    #[test]
    fn poll_slices_never_exceed_bound() {
        // Arrange
        let mut orchestrator = armed_orchestrator();
        let t0 = Instant::now();
        orchestrator.note_dispatched(t0);

        // Act
        let step = orchestrator.next_step(t0 + Duration::from_millis(500));

        // Assert
        let BridgeStep::Poll { slice_ms } = step else {
            panic!("expected a poll step, got {step:?}");
        };
        assert!(slice_ms <= MAX_POLL_SLICE_MS);
        assert!(slice_ms <= 100);
    }

    #[test]
    fn markers_are_exact_and_redaction_safe() {
        // Arrange
        let redispatched = job_redispatched_marker(3);
        let timeout = timeout_streak_marker(2);

        // Act
        let markers = [redispatched.as_str(), timeout.as_str()];

        // Assert: exact formats, counters only, no raw job/extranonce/hex material.
        assert_eq!(
            redispatched,
            "asic_bridge=job_redispatched extranonce2_counter=3"
        );
        assert_eq!(timeout, "asic_bridge=timeout_streak count=2");
        for marker in markers {
            assert!(!marker.contains("0x"));
            assert!(!marker.contains("extranonce2="));
            assert!(!marker.contains("job_id="));
        }
    }

    #[test]
    fn module_source_stays_pure_of_shell_types() {
        // Arrange: forbidden tokens built by concatenation so this test
        // cannot match its own source text.
        let source = include_str!("bridge_orchestration.rs");
        let forbidden_tokens = [
            ["esp", "_idf"].concat(),
            ["firm", "ware"].concat(),
            ["Uart", "Driver"].concat(),
        ];

        // Act & Assert
        for token in &forbidden_tokens {
            assert!(
                !source.contains(token.as_str()),
                "pure module must not reference shell token {token}"
            );
        }
    }
}
