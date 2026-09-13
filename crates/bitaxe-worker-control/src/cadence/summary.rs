use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CadencePhase {
    Idle,
    Usb,
    Mining,
}

impl CadencePhase {
    pub(super) const fn index(self) -> usize {
        match self {
            Self::Idle => 0,
            Self::Usb => 1,
            Self::Mining => 2,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CadenceState {
    Empty,
    Armed,
    Capturing,
    Complete,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CadenceArmReceipt {
    pub schema: &'static str,
    pub phase: CadencePhase,
    pub armed_at_us: u64,
    pub generation: u32,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CadenceSummary {
    pub phase: CadencePhase,
    pub state: CadenceState,
    pub armed_at_us: u64,
    pub started_at_us: u64,
    pub ended_at_us: u64,
    pub generation: u32,
    pub interval_count: u32,
    pub max_probe_count: u32,
    pub first_max_probe_at_us: u64,
    pub last_max_probe_at_us: u64,
    pub interval_buckets: [u32; 4],
    pub maximum_interval_us: u64,
    pub maximum_execution_us: u64,
    pub maximum_live_us: u64,
    pub maximum_logs_us: u64,
    pub maximum_prune_us: u64,
    pub cpu_mismatch_count: u32,
    pub priority_mismatch_count: u32,
    pub subscriber_mismatch_count: u32,
    pub projection_count: u32,
    pub unchanged_count: u32,
    pub no_subscriber_count: u32,
    pub projection_failures: u32,
    pub serialization_failures: u32,
    pub queue_failures: u32,
    pub send_failures: u32,
    pub sends_queued: u32,
    pub sends_completed: u32,
    pub pending_sends: u32,
    pub clock_failures: u32,
    pub overflow: bool,
    pub passed: bool,
}

impl CadenceSummary {
    pub(super) const fn empty(phase: CadencePhase) -> Self {
        Self {
            phase,
            state: CadenceState::Empty,
            armed_at_us: 0,
            started_at_us: 0,
            ended_at_us: 0,
            generation: 0,
            interval_count: 0,
            max_probe_count: 0,
            first_max_probe_at_us: 0,
            last_max_probe_at_us: 0,
            interval_buckets: [0; 4],
            maximum_interval_us: 0,
            maximum_execution_us: 0,
            maximum_live_us: 0,
            maximum_logs_us: 0,
            maximum_prune_us: 0,
            cpu_mismatch_count: 0,
            priority_mismatch_count: 0,
            subscriber_mismatch_count: 0,
            projection_count: 0,
            unchanged_count: 0,
            no_subscriber_count: 0,
            projection_failures: 0,
            serialization_failures: 0,
            queue_failures: 0,
            send_failures: 0,
            sends_queued: 0,
            sends_completed: 0,
            pending_sends: 0,
            clock_failures: 0,
            overflow: false,
            passed: false,
        }
    }

    pub(super) fn refresh_passed(&mut self, dropped: u32) {
        self.passed = self.state == CadenceState::Complete
            && self.interval_count >= 60
            && (self.phase != CadencePhase::Usb || self.max_probe_count == 12)
            && (u64::from(self.interval_buckets[0]) + u64::from(self.interval_buckets[1])) * 100
                >= u64::from(self.interval_count) * 95
            && self.maximum_interval_us <= 1_500_000
            && self.maximum_execution_us <= 500_000
            && self.cpu_mismatch_count == 0
            && self.priority_mismatch_count == 0
            && self.subscriber_mismatch_count == 0
            && self.projection_failures == 0
            && self.serialization_failures == 0
            && self.queue_failures == 0
            && self.send_failures == 0
            && self.pending_sends == 0
            && self.clock_failures == 0
            && !self.overflow
            && dropped == 0;
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CadenceSnapshot {
    pub schema: &'static str,
    pub snapshot_available: bool,
    /// Saturated loss indication: zero means none; one means at least one observation was lost.
    pub dropped_observations: u32,
    pub storage_bytes: usize,
    pub phases: [CadenceSummary; 3],
}

#[cfg(test)]
mod tests {
    use super::*;

    fn complete() -> CadenceSummary {
        let mut summary = CadenceSummary::empty(CadencePhase::Idle);
        summary.state = CadenceState::Complete;
        summary.interval_count = 100;
        summary.interval_buckets = [0, 95, 5, 0];
        summary.maximum_interval_us = 1_500_000;
        summary.maximum_execution_us = 500_000;
        summary
    }

    #[test]
    fn exact_prospective_limits_pass() {
        // Arrange
        let mut summary = complete();
        // Act
        summary.refresh_passed(0);
        // Assert
        assert!(summary.passed);
    }

    #[test]
    fn percentile_below_ninety_five_is_rejected() {
        // Arrange
        let mut summary = complete();
        summary.interval_buckets = [0, 94, 6, 0];
        // Act
        summary.refresh_passed(0);
        // Assert
        assert!(!summary.passed);
    }

    #[test]
    fn one_microsecond_above_execution_limit_is_rejected() {
        // Arrange
        let mut summary = complete();
        summary.maximum_execution_us = 500_001;
        // Act
        summary.refresh_passed(0);
        // Assert
        assert!(!summary.passed);
    }

    #[test]
    fn one_microsecond_above_interval_limit_is_rejected() {
        // Arrange
        let mut summary = complete();
        summary.maximum_interval_us = 1_500_001;
        // Act
        summary.refresh_passed(0);
        // Assert
        assert!(!summary.passed);
    }
}
