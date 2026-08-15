use std::time::{Duration, Instant};

const RECOVERY_SAFE_STOP_DEADLINE: Duration = Duration::from_secs(130);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum RecoveryJoinDecision {
    Wait,
    Ready,
    TimedOut,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct RecoveryPauseJoinState {
    deadline: Instant,
    baseline_serial_observation_count: u64,
    api_pause_confirmed: bool,
    serial_safe_stop_confirmed: bool,
}

impl RecoveryPauseJoinState {
    pub(super) fn new(now: Instant, baseline_serial_observation_count: u64) -> Self {
        Self {
            deadline: now + RECOVERY_SAFE_STOP_DEADLINE,
            baseline_serial_observation_count,
            api_pause_confirmed: false,
            serial_safe_stop_confirmed: false,
        }
    }

    pub(super) fn observe(
        &mut self,
        api_pause_confirmed: bool,
        serial_observation_count: u64,
        now: Instant,
    ) -> RecoveryJoinDecision {
        self.api_pause_confirmed |= api_pause_confirmed;
        self.serial_safe_stop_confirmed |=
            serial_observation_count > self.baseline_serial_observation_count;
        if now >= self.deadline {
            return RecoveryJoinDecision::TimedOut;
        }
        if self.api_pause_confirmed && self.serial_safe_stop_confirmed {
            return RecoveryJoinDecision::Ready;
        }
        RecoveryJoinDecision::Wait
    }

    pub(super) const fn api_pause_confirmed(self) -> bool {
        self.api_pause_confirmed
    }

    pub(super) const fn serial_safe_stop_confirmed(self) -> bool {
        self.serial_safe_stop_confirmed
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::{RecoveryJoinDecision, RecoveryPauseJoinState};

    #[test]
    fn recovery_requires_post_request_api_and_serial_facts() {
        // Arrange
        let now = Instant::now();
        let mut join = RecoveryPauseJoinState::new(now, 4);

        // Act
        let stale_serial = join.observe(true, 4, now);
        let joined = join.observe(false, 5, now);

        // Assert
        assert_eq!(stale_serial, RecoveryJoinDecision::Wait);
        assert_eq!(joined, RecoveryJoinDecision::Ready);
        assert!(join.api_pause_confirmed());
        assert!(join.serial_safe_stop_confirmed());
    }

    #[test]
    fn recovery_deadline_fails_closed_with_both_facts() {
        // Arrange
        let now = Instant::now();
        let mut join = RecoveryPauseJoinState::new(now, 1);

        // Act
        let decision = join.observe(true, 2, now + Duration::from_secs(130));

        // Assert
        assert_eq!(decision, RecoveryJoinDecision::TimedOut);
    }
}
