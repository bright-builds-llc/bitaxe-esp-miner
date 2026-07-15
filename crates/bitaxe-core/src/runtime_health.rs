//! Pure, observation-only runtime-health derivation.

/// Maximum serialized supervisor checkpoint category length.
pub const CHECKPOINT_CATEGORY_MAX_ASCII_BYTES: usize = 32;
const STALE_INTERVAL_MULTIPLIER: u64 = 3;
const UNHEALTHY_INTERVAL_MULTIPLIER: u64 = 10;

/// Passive self-test lifecycle values exposed to operators.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PassiveSelfTestState {
    Idle,
    Blocked,
    Running,
    Passed,
    Failed,
    Canceled,
    Unavailable,
}

impl PassiveSelfTestState {
    /// Returns the exact serialized spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Blocked => "blocked",
            Self::Running => "running",
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Canceled => "canceled",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Whether a supervisor checkpoint can be observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupervisorAvailability {
    Available,
    Unavailable,
}

impl SupervisorAvailability {
    /// Returns the exact serialized spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Available => "available",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Age-derived health of the latest supervisor checkpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointHealth {
    Healthy,
    Stale,
    Unhealthy,
    Unavailable,
}

impl CheckpointHealth {
    /// Returns the exact serialized spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Stale => "stale",
            Self::Unhealthy => "unhealthy",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Independently observed ESP task-watchdog participation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskWatchdogParticipation {
    Participating,
    NotParticipating,
    Unavailable,
}

impl TaskWatchdogParticipation {
    /// Returns the exact serialized spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Participating => "participating",
            Self::NotParticipating => "not_participating",
            Self::Unavailable => "unavailable",
        }
    }
}

/// Invalid producer checkpoint input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CheckpointObservationError {
    EmptyCategory,
    CategoryTooLong,
    CategoryNotAscii,
    SequenceRegression,
    MonotonicTimeRegression,
    SameSequenceChanged,
}

/// Validated bounded checkpoint category.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointCategory(String);

impl CheckpointCategory {
    /// Parses a bounded non-empty ASCII checkpoint category.
    pub fn new(value: &str) -> Result<Self, CheckpointObservationError> {
        if value.is_empty() {
            return Err(CheckpointObservationError::EmptyCategory);
        }
        if !value.is_ascii() {
            return Err(CheckpointObservationError::CategoryNotAscii);
        }
        if value.len() > CHECKPOINT_CATEGORY_MAX_ASCII_BYTES {
            return Err(CheckpointObservationError::CategoryTooLong);
        }

        Ok(Self(value.to_owned()))
    }

    /// Returns the validated category text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// One producer-owned supervisor checkpoint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckpointObservation {
    category: CheckpointCategory,
    sequence: u64,
    observed_at_millis: u64,
}

impl CheckpointObservation {
    /// Creates one validated checkpoint observation.
    pub fn new(
        category: &str,
        sequence: u64,
        observed_at_millis: u64,
    ) -> Result<Self, CheckpointObservationError> {
        Ok(Self {
            category: CheckpointCategory::new(category)?,
            sequence,
            observed_at_millis,
        })
    }

    /// Returns the bounded category.
    #[must_use]
    pub fn category(&self) -> &str {
        self.category.as_str()
    }

    /// Returns the producer-owned sequence.
    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    /// Returns the producer-owned monotonic observation time.
    #[must_use]
    pub const fn observed_at_millis(&self) -> u64 {
        self.observed_at_millis
    }

    /// Validates that this observation follows a prior observation without
    /// regression or same-sequence mutation.
    pub fn validate_after(&self, previous: &Self) -> Result<(), CheckpointObservationError> {
        if self.sequence < previous.sequence {
            return Err(CheckpointObservationError::SequenceRegression);
        }
        if self.observed_at_millis < previous.observed_at_millis {
            return Err(CheckpointObservationError::MonotonicTimeRegression);
        }
        if self.sequence == previous.sequence && self != previous {
            return Err(CheckpointObservationError::SameSequenceChanged);
        }

        Ok(())
    }
}

/// One immutable runtime-health projection captured under an operator snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeHealthSnapshot {
    passive_self_test_state: PassiveSelfTestState,
    supervisor_availability: SupervisorAvailability,
    maybe_checkpoint_category: Option<String>,
    maybe_checkpoint_sequence: Option<u64>,
    maybe_checkpoint_age_millis: Option<u64>,
    checkpoint_health: CheckpointHealth,
    task_watchdog_participation: TaskWatchdogParticipation,
    maybe_task_watchdog_reason: Option<&'static str>,
}

impl RuntimeHealthSnapshot {
    /// Derives health from already-observed state and monotonic timestamps.
    ///
    /// Phase 34 has no direct proof of task-watchdog participation, so that
    /// independent fact is always unavailable with reason `unproved`.
    #[must_use]
    pub fn evaluate(
        passive_self_test_state: PassiveSelfTestState,
        maybe_previous_checkpoint: Option<&CheckpointObservation>,
        maybe_latest_checkpoint: Option<&CheckpointObservation>,
        current_monotonic_millis: u64,
        publish_interval_millis: u64,
    ) -> Self {
        let Some(latest_checkpoint) = maybe_latest_checkpoint else {
            return Self::unavailable(passive_self_test_state);
        };

        if let Some(previous_checkpoint) = maybe_previous_checkpoint {
            if latest_checkpoint
                .validate_after(previous_checkpoint)
                .is_err()
            {
                return Self::unavailable(passive_self_test_state);
            }
        }

        let Some(checkpoint_age_millis) =
            current_monotonic_millis.checked_sub(latest_checkpoint.observed_at_millis())
        else {
            return Self::unavailable(passive_self_test_state);
        };
        let Some(stale_after_millis) =
            publish_interval_millis.checked_mul(STALE_INTERVAL_MULTIPLIER)
        else {
            return Self::unavailable(passive_self_test_state);
        };
        let Some(unhealthy_after_millis) =
            publish_interval_millis.checked_mul(UNHEALTHY_INTERVAL_MULTIPLIER)
        else {
            return Self::unavailable(passive_self_test_state);
        };

        let checkpoint_health = if checkpoint_age_millis <= stale_after_millis {
            CheckpointHealth::Healthy
        } else if checkpoint_age_millis <= unhealthy_after_millis {
            CheckpointHealth::Stale
        } else {
            CheckpointHealth::Unhealthy
        };

        Self {
            passive_self_test_state,
            supervisor_availability: SupervisorAvailability::Available,
            maybe_checkpoint_category: Some(latest_checkpoint.category().to_owned()),
            maybe_checkpoint_sequence: Some(latest_checkpoint.sequence()),
            maybe_checkpoint_age_millis: Some(checkpoint_age_millis),
            checkpoint_health,
            task_watchdog_participation: TaskWatchdogParticipation::Unavailable,
            maybe_task_watchdog_reason: Some("unproved"),
        }
    }

    /// Returns an observation-only fixture with no authenticated health facts.
    #[must_use]
    pub const fn fixture_unavailable() -> Self {
        Self::unavailable(PassiveSelfTestState::Unavailable)
    }

    const fn unavailable(passive_self_test_state: PassiveSelfTestState) -> Self {
        Self {
            passive_self_test_state,
            supervisor_availability: SupervisorAvailability::Unavailable,
            maybe_checkpoint_category: None,
            maybe_checkpoint_sequence: None,
            maybe_checkpoint_age_millis: None,
            checkpoint_health: CheckpointHealth::Unavailable,
            task_watchdog_participation: TaskWatchdogParticipation::Unavailable,
            maybe_task_watchdog_reason: Some("unproved"),
        }
    }

    #[must_use]
    pub const fn passive_self_test_state(&self) -> PassiveSelfTestState {
        self.passive_self_test_state
    }

    #[must_use]
    pub const fn supervisor_availability(&self) -> SupervisorAvailability {
        self.supervisor_availability
    }

    #[must_use]
    pub fn maybe_checkpoint_category(&self) -> Option<&str> {
        self.maybe_checkpoint_category.as_deref()
    }

    #[must_use]
    pub const fn maybe_checkpoint_sequence(&self) -> Option<u64> {
        self.maybe_checkpoint_sequence
    }

    #[must_use]
    pub const fn maybe_checkpoint_age_millis(&self) -> Option<u64> {
        self.maybe_checkpoint_age_millis
    }

    #[must_use]
    pub const fn checkpoint_health(&self) -> CheckpointHealth {
        self.checkpoint_health
    }

    #[must_use]
    pub const fn task_watchdog_participation(&self) -> TaskWatchdogParticipation {
        self.task_watchdog_participation
    }

    #[must_use]
    pub const fn maybe_task_watchdog_reason(&self) -> Option<&'static str> {
        self.maybe_task_watchdog_reason
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn checkpoint(sequence: u64, observed_at_millis: u64) -> CheckpointObservation {
        CheckpointObservation::new("telemetry", sequence, observed_at_millis)
            .expect("test checkpoint should be valid")
    }

    #[test]
    fn passive_self_test_states_have_exact_serialized_spellings() {
        // Arrange
        let cases = [
            (PassiveSelfTestState::Idle, "idle"),
            (PassiveSelfTestState::Blocked, "blocked"),
            (PassiveSelfTestState::Running, "running"),
            (PassiveSelfTestState::Passed, "passed"),
            (PassiveSelfTestState::Failed, "failed"),
            (PassiveSelfTestState::Canceled, "canceled"),
            (PassiveSelfTestState::Unavailable, "unavailable"),
        ];

        // Act / Assert
        for (state, expected) in cases {
            assert_eq!(state.as_str(), expected);
        }
    }

    #[test]
    fn health_and_watchdog_vocabulary_has_exact_serialized_spellings() {
        // Arrange
        let supervisor = [
            (SupervisorAvailability::Available, "available"),
            (SupervisorAvailability::Unavailable, "unavailable"),
        ];
        let health = [
            (CheckpointHealth::Healthy, "healthy"),
            (CheckpointHealth::Stale, "stale"),
            (CheckpointHealth::Unhealthy, "unhealthy"),
            (CheckpointHealth::Unavailable, "unavailable"),
        ];
        let participation = [
            (TaskWatchdogParticipation::Participating, "participating"),
            (
                TaskWatchdogParticipation::NotParticipating,
                "not_participating",
            ),
            (TaskWatchdogParticipation::Unavailable, "unavailable"),
        ];

        // Act / Assert
        for (value, expected) in supervisor {
            assert_eq!(value.as_str(), expected);
        }
        for (value, expected) in health {
            assert_eq!(value.as_str(), expected);
        }
        for (value, expected) in participation {
            assert_eq!(value.as_str(), expected);
        }
    }

    #[test]
    fn checkpoint_category_rejects_empty_non_ascii_and_overlong_text() {
        // Arrange
        let overlong = "x".repeat(CHECKPOINT_CATEGORY_MAX_ASCII_BYTES + 1);

        // Act
        let empty = CheckpointCategory::new("");
        let non_ascii = CheckpointCategory::new("télémetry");
        let too_long = CheckpointCategory::new(&overlong);

        // Assert
        assert_eq!(empty, Err(CheckpointObservationError::EmptyCategory));
        assert_eq!(non_ascii, Err(CheckpointObservationError::CategoryNotAscii));
        assert_eq!(too_long, Err(CheckpointObservationError::CategoryTooLong));
    }

    #[test]
    fn checkpoint_transition_rejects_sequence_and_monotonic_time_regression() {
        // Arrange
        let previous = checkpoint(7, 1_000);
        let sequence_regression = checkpoint(6, 1_100);
        let time_regression = checkpoint(8, 999);

        // Act
        let sequence_result = sequence_regression.validate_after(&previous);
        let time_result = time_regression.validate_after(&previous);

        // Assert
        assert_eq!(
            sequence_result,
            Err(CheckpointObservationError::SequenceRegression)
        );
        assert_eq!(
            time_result,
            Err(CheckpointObservationError::MonotonicTimeRegression)
        );
    }

    #[test]
    fn checkpoint_transition_rejects_same_sequence_mutation() {
        // Arrange
        let previous = checkpoint(7, 1_000);
        let changed_timestamp = checkpoint(7, 1_001);

        // Act
        let result = changed_timestamp.validate_after(&previous);

        // Assert
        assert_eq!(result, Err(CheckpointObservationError::SameSequenceChanged));
    }

    #[test]
    fn missing_checkpoint_is_explicitly_unavailable() {
        // Arrange / Act
        let snapshot = RuntimeHealthSnapshot::evaluate(
            PassiveSelfTestState::Unavailable,
            None,
            None,
            10_000,
            500,
        );

        // Assert
        assert_eq!(
            snapshot.supervisor_availability(),
            SupervisorAvailability::Unavailable
        );
        assert_eq!(snapshot.checkpoint_health(), CheckpointHealth::Unavailable);
        assert_eq!(snapshot.maybe_checkpoint_sequence(), None);
    }

    #[test]
    fn exact_age_boundaries_derive_healthy_stale_and_unhealthy() {
        // Arrange
        let latest = checkpoint(8, 1_000);

        // Act
        let healthy = RuntimeHealthSnapshot::evaluate(
            PassiveSelfTestState::Idle,
            None,
            Some(&latest),
            2_500,
            500,
        );
        let stale_start = RuntimeHealthSnapshot::evaluate(
            PassiveSelfTestState::Idle,
            None,
            Some(&latest),
            2_501,
            500,
        );
        let stale_end = RuntimeHealthSnapshot::evaluate(
            PassiveSelfTestState::Idle,
            None,
            Some(&latest),
            6_000,
            500,
        );
        let unhealthy = RuntimeHealthSnapshot::evaluate(
            PassiveSelfTestState::Idle,
            None,
            Some(&latest),
            6_001,
            500,
        );

        // Assert
        assert_eq!(healthy.checkpoint_health(), CheckpointHealth::Healthy);
        assert_eq!(stale_start.checkpoint_health(), CheckpointHealth::Stale);
        assert_eq!(stale_end.checkpoint_health(), CheckpointHealth::Stale);
        assert_eq!(unhealthy.checkpoint_health(), CheckpointHealth::Unhealthy);
    }

    #[test]
    fn fixed_sequence_ages_from_healthy_to_stale_to_unhealthy() {
        // Arrange
        let latest = checkpoint(11, 2_000);

        // Act
        let snapshots = [2_100, 3_501, 7_001].map(|now| {
            RuntimeHealthSnapshot::evaluate(
                PassiveSelfTestState::Idle,
                None,
                Some(&latest),
                now,
                500,
            )
        });

        // Assert
        assert_eq!(snapshots[0].checkpoint_health(), CheckpointHealth::Healthy);
        assert_eq!(snapshots[1].checkpoint_health(), CheckpointHealth::Stale);
        assert_eq!(
            snapshots[2].checkpoint_health(),
            CheckpointHealth::Unhealthy
        );
        assert!(snapshots
            .iter()
            .all(|snapshot| snapshot.maybe_checkpoint_sequence() == Some(11)));
    }

    #[test]
    fn recovery_requires_a_sequence_advance() {
        // Arrange
        let previous = checkpoint(11, 2_000);
        let unchanged_sequence = checkpoint(11, 8_000);
        let advanced_sequence = checkpoint(12, 8_000);

        // Act
        let frozen = RuntimeHealthSnapshot::evaluate(
            PassiveSelfTestState::Idle,
            None,
            Some(&previous),
            7_001,
            500,
        );
        let synthetic_recovery = RuntimeHealthSnapshot::evaluate(
            PassiveSelfTestState::Idle,
            Some(&previous),
            Some(&unchanged_sequence),
            8_001,
            500,
        );
        let real_recovery = RuntimeHealthSnapshot::evaluate(
            PassiveSelfTestState::Idle,
            Some(&previous),
            Some(&advanced_sequence),
            8_001,
            500,
        );

        // Assert
        assert_eq!(frozen.checkpoint_health(), CheckpointHealth::Unhealthy);
        assert_eq!(
            synthetic_recovery.checkpoint_health(),
            CheckpointHealth::Unavailable
        );
        assert_eq!(real_recovery.checkpoint_health(), CheckpointHealth::Healthy);
    }

    #[test]
    fn invalid_time_or_threshold_arithmetic_is_unavailable() {
        // Arrange
        let latest = checkpoint(4, 100);

        // Act
        let time_regression = RuntimeHealthSnapshot::evaluate(
            PassiveSelfTestState::Unavailable,
            None,
            Some(&latest),
            99,
            500,
        );
        let threshold_overflow = RuntimeHealthSnapshot::evaluate(
            PassiveSelfTestState::Unavailable,
            None,
            Some(&latest),
            100,
            u64::MAX,
        );

        // Assert
        assert_eq!(
            time_regression.checkpoint_health(),
            CheckpointHealth::Unavailable
        );
        assert_eq!(
            threshold_overflow.checkpoint_health(),
            CheckpointHealth::Unavailable
        );
    }

    #[test]
    fn healthy_supervisor_does_not_imply_task_watchdog_participation() {
        // Arrange
        let latest = checkpoint(42, 10_000);

        // Act
        let snapshot = RuntimeHealthSnapshot::evaluate(
            PassiveSelfTestState::Unavailable,
            None,
            Some(&latest),
            10_100,
            500,
        );

        // Assert
        assert_eq!(
            snapshot.supervisor_availability(),
            SupervisorAvailability::Available
        );
        assert_eq!(snapshot.checkpoint_health(), CheckpointHealth::Healthy);
        assert_eq!(
            snapshot.task_watchdog_participation(),
            TaskWatchdogParticipation::Unavailable
        );
        assert_eq!(snapshot.maybe_task_watchdog_reason(), Some("unproved"));
    }
}
