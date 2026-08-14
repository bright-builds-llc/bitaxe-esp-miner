use std::time::{Duration, Instant};

const SAFE_STOP_DEADLINE: Duration = Duration::from_secs(130);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum PauseJoinDecision {
    Wait,
    Ready,
    TimedOut,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct PauseJoinState {
    deadline: Instant,
    logical_pause_confirmed: bool,
}

impl PauseJoinState {
    pub(super) fn new(now: Instant) -> Self {
        Self {
            deadline: now + SAFE_STOP_DEADLINE,
            logical_pause_confirmed: false,
        }
    }

    pub(super) fn expired(self, now: Instant) -> bool {
        now >= self.deadline
    }

    pub(super) fn observe(
        &mut self,
        logical_pause_confirmed: bool,
        serial_safe_stop_confirmed: bool,
        now: Instant,
    ) -> PauseJoinDecision {
        self.logical_pause_confirmed |= logical_pause_confirmed;
        if self.expired(now) {
            return PauseJoinDecision::TimedOut;
        }
        if self.logical_pause_confirmed && serial_safe_stop_confirmed {
            return PauseJoinDecision::Ready;
        }
        PauseJoinDecision::Wait
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use super::{PauseJoinDecision, PauseJoinState};

    #[test]
    fn logical_pause_alone_cannot_release_resume() {
        // Arrange
        let now = Instant::now();
        let mut join = PauseJoinState::new(now);

        // Act
        let decision = join.observe(true, false, now);

        // Assert
        assert_eq!(decision, PauseJoinDecision::Wait);
    }

    #[test]
    fn either_observation_order_releases_resume_after_both_facts_join() {
        // Arrange
        let now = Instant::now();
        let mut serial_first = PauseJoinState::new(now);
        let mut logical_first = PauseJoinState::new(now);

        // Act
        let serial_wait = serial_first.observe(false, true, now);
        let serial_joined = serial_first.observe(true, true, now);
        let logical_wait = logical_first.observe(true, false, now);
        let logical_joined = logical_first.observe(false, true, now);

        // Assert
        assert_eq!(serial_wait, PauseJoinDecision::Wait);
        assert_eq!(logical_wait, PauseJoinDecision::Wait);
        assert_eq!(serial_joined, PauseJoinDecision::Ready);
        assert_eq!(logical_joined, PauseJoinDecision::Ready);
    }

    #[test]
    fn exact_deadline_fails_closed_even_when_both_facts_are_present() {
        // Arrange
        let now = Instant::now();
        let mut join = PauseJoinState::new(now);
        let deadline = now + Duration::from_secs(130);

        // Act
        let decision = join.observe(true, true, deadline);

        // Assert
        assert_eq!(decision, PauseJoinDecision::TimedOut);
    }
}
