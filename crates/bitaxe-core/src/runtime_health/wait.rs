/// Closed classification of the production owner's receive wait.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskWatchdogWaitState {
    NotWaiting,
    WithinDeadline,
    DeadlineOverrun,
    InvalidObservation,
}

impl TaskWatchdogWaitState {
    /// Returns the exact serialized spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::NotWaiting => "not_waiting",
            Self::WithinDeadline => "within_deadline",
            Self::DeadlineOverrun => "deadline_overrun",
            Self::InvalidObservation => "invalid_observation",
        }
    }
}

/// One lock-free producer observation used to derive the receive-wait state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskWatchdogWaitObservation {
    NotWaiting,
    WaitingUntil { deadline_millis_low: u32 },
    Invalid,
}

impl TaskWatchdogWaitObservation {
    /// Creates a waiting observation from a checked absolute deadline.
    #[must_use]
    pub const fn waiting_until(maybe_deadline_millis: Option<u64>) -> Self {
        match maybe_deadline_millis {
            Some(deadline_millis) => Self::WaitingUntil {
                deadline_millis_low: deadline_millis as u32,
            },
            None => Self::Invalid,
        }
    }

    /// Derives the closed wait state at the post-observation evaluation time.
    #[must_use]
    pub const fn state_at(self, current_monotonic_millis: u64) -> TaskWatchdogWaitState {
        match self {
            Self::NotWaiting => TaskWatchdogWaitState::NotWaiting,
            Self::Invalid => TaskWatchdogWaitState::InvalidObservation,
            Self::WaitingUntil {
                deadline_millis_low,
            } => {
                let current_millis_low = current_monotonic_millis as u32;
                let elapsed_past_deadline = current_millis_low.wrapping_sub(deadline_millis_low);
                if (elapsed_past_deadline as i32) <= 0 {
                    TaskWatchdogWaitState::WithinDeadline
                } else {
                    TaskWatchdogWaitState::DeadlineOverrun
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wait_state_has_exact_closed_spellings() {
        // Arrange
        let cases = [
            (TaskWatchdogWaitState::NotWaiting, "not_waiting"),
            (TaskWatchdogWaitState::WithinDeadline, "within_deadline"),
            (TaskWatchdogWaitState::DeadlineOverrun, "deadline_overrun"),
            (
                TaskWatchdogWaitState::InvalidObservation,
                "invalid_observation",
            ),
        ];

        // Act / Assert
        for (state, expected) in cases {
            assert_eq!(state.as_str(), expected);
        }
    }

    #[test]
    fn exact_deadline_is_within_and_one_millisecond_late_is_overrun() {
        // Arrange
        let observation = TaskWatchdogWaitObservation::waiting_until(Some(2_000));

        // Act / Assert
        assert_eq!(
            observation.state_at(2_000),
            TaskWatchdogWaitState::WithinDeadline
        );
        assert_eq!(
            observation.state_at(2_001),
            TaskWatchdogWaitState::DeadlineOverrun
        );
    }

    #[test]
    fn non_waiting_and_missing_or_overflowed_deadlines_remain_distinct() {
        // Arrange
        let missing = TaskWatchdogWaitObservation::waiting_until(None);
        let wrapped = TaskWatchdogWaitObservation::waiting_until(Some(1_u64 << 32));

        // Act / Assert
        assert_eq!(
            TaskWatchdogWaitObservation::NotWaiting.state_at(u64::MAX),
            TaskWatchdogWaitState::NotWaiting
        );
        assert_eq!(
            missing.state_at(1),
            TaskWatchdogWaitState::InvalidObservation
        );
        assert_eq!(
            wrapped.state_at(1_u64 << 32),
            TaskWatchdogWaitState::WithinDeadline
        );
    }

    #[test]
    fn production_shaped_scheduler_delay_crosses_only_the_wait_deadline() {
        // Arrange
        let entered_at_millis = 600_000_u64;
        let maybe_deadline_millis = entered_at_millis.checked_add(1_000);
        let observation = TaskWatchdogWaitObservation::waiting_until(maybe_deadline_millis);

        // Act
        let on_time = observation.state_at(601_000);
        let delayed = observation.state_at(605_001);

        // Assert
        assert_eq!(on_time, TaskWatchdogWaitState::WithinDeadline);
        assert_eq!(delayed, TaskWatchdogWaitState::DeadlineOverrun);
    }

    #[test]
    fn modulo_deadline_remains_ordered_across_u32_wrap() {
        // Arrange
        let deadline = u64::from(u32::MAX) + 500;
        let observation = TaskWatchdogWaitObservation::waiting_until(Some(deadline));

        // Act / Assert
        assert_eq!(
            observation.state_at(deadline),
            TaskWatchdogWaitState::WithinDeadline
        );
        assert_eq!(
            observation.state_at(deadline + 1),
            TaskWatchdogWaitState::DeadlineOverrun
        );
    }
}
