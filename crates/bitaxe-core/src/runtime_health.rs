//! Pure, observation-only runtime-health derivation.

#[path = "runtime_health/wait.rs"]
mod wait;
pub use wait::{
    TaskWatchdogOwnerPhase, TaskWatchdogOwnerSubphase, TaskWatchdogReadOutcome,
    TaskWatchdogWaitObservation, TaskWatchdogWaitState,
};

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

/// One closed, producer-owned observation of ESP task-watchdog participation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskWatchdogObservation {
    /// The firmware adapter has not established an observation yet.
    Unavailable,
    /// ESP-IDF rejected subscription of the monitored owner.
    SubscriptionFailed,
    /// ESP-IDF rejected a feed after successful subscription.
    FeedFailed,
    /// ESP-IDF rejected owner cleanup after successful subscription.
    UnsubscriptionFailed,
    /// The monitored owner has exited and unsubscribed cleanly.
    Unsubscribed,
    /// One successful ESP-IDF feed owned by the monitored task.
    Fed {
        /// Monotonic successful feed sequence.
        sequence: u64,
        /// Firmware monotonic time at which the feed succeeded.
        observed_at_millis: u64,
    },
}

/// Monotonic observation time and the configured health-policy intervals.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeHealthTiming {
    current_monotonic_millis: u64,
    publish_interval_millis: u64,
    task_watchdog_timeout_millis: u64,
}

impl RuntimeHealthTiming {
    /// Constructs the timing policy supplied by the runtime boundary.
    #[must_use]
    pub const fn new(
        current_monotonic_millis: u64,
        publish_interval_millis: u64,
        task_watchdog_timeout_millis: u64,
    ) -> Self {
        Self {
            current_monotonic_millis,
            publish_interval_millis,
            task_watchdog_timeout_millis,
        }
    }
}

impl TaskWatchdogObservation {
    /// Constructs one successful feed observation.
    #[must_use]
    pub const fn fed(sequence: u64, observed_at_millis: u64) -> Self {
        Self::Fed {
            sequence,
            observed_at_millis,
        }
    }

    /// Validates monotonic feed evidence following a prior observation.
    #[must_use]
    pub const fn is_valid_after(self, previous: Self) -> bool {
        match (previous, self) {
            (
                Self::Fed {
                    sequence: previous_sequence,
                    observed_at_millis: previous_millis,
                },
                Self::Fed {
                    sequence,
                    observed_at_millis,
                },
            ) => {
                sequence >= previous_sequence
                    && observed_at_millis >= previous_millis
                    && (sequence != previous_sequence || observed_at_millis == previous_millis)
            }
            _ => true,
        }
    }
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
    maybe_task_watchdog_feed_sequence: Option<u64>,
    maybe_task_watchdog_feed_age_millis: Option<u64>,
    task_watchdog_read_outcome: TaskWatchdogReadOutcome,
    task_watchdog_owner_phase: TaskWatchdogOwnerPhase,
    task_watchdog_owner_subphase: TaskWatchdogOwnerSubphase,
    task_watchdog_wait_state: TaskWatchdogWaitState,
}

impl RuntimeHealthSnapshot {
    /// Derives health from already-observed state and monotonic timestamps.
    ///
    #[must_use]
    pub fn evaluate(
        passive_self_test_state: PassiveSelfTestState,
        maybe_previous_checkpoint: Option<&CheckpointObservation>,
        maybe_latest_checkpoint: Option<&CheckpointObservation>,
        maybe_previous_task_watchdog: Option<TaskWatchdogObservation>,
        maybe_latest_task_watchdog: Option<TaskWatchdogObservation>,
        timing: RuntimeHealthTiming,
    ) -> Self {
        let supervisor = evaluate_supervisor(
            maybe_previous_checkpoint,
            maybe_latest_checkpoint,
            timing.current_monotonic_millis,
            timing.publish_interval_millis,
        );
        let task_watchdog_read_outcome = if maybe_latest_task_watchdog.is_some() {
            TaskWatchdogReadOutcome::Stable
        } else {
            TaskWatchdogReadOutcome::Uninitialized
        };
        let watchdog = evaluate_task_watchdog(
            maybe_previous_task_watchdog,
            maybe_latest_task_watchdog,
            timing.current_monotonic_millis,
            timing.task_watchdog_timeout_millis,
        );

        Self {
            passive_self_test_state,
            supervisor_availability: supervisor.availability,
            maybe_checkpoint_category: supervisor.maybe_category,
            maybe_checkpoint_sequence: supervisor.maybe_sequence,
            maybe_checkpoint_age_millis: supervisor.maybe_age_millis,
            checkpoint_health: supervisor.health,
            task_watchdog_participation: watchdog.participation,
            maybe_task_watchdog_reason: Some(watchdog.reason),
            maybe_task_watchdog_feed_sequence: watchdog.maybe_sequence,
            maybe_task_watchdog_feed_age_millis: watchdog.maybe_age_millis,
            task_watchdog_read_outcome,
            task_watchdog_owner_phase: TaskWatchdogOwnerPhase::Unavailable,
            task_watchdog_owner_subphase: TaskWatchdogOwnerSubphase::Unavailable,
            task_watchdog_wait_state: TaskWatchdogWaitState::NotWaiting,
        }
    }

    /// Attaches the independently observed owner-loop phase.
    #[must_use]
    pub const fn with_task_watchdog_owner_phase(mut self, phase: TaskWatchdogOwnerPhase) -> Self {
        self.task_watchdog_owner_phase = phase;
        self
    }

    /// Attaches the independently observed closed owner-work boundary.
    #[must_use]
    pub const fn with_task_watchdog_owner_subphase(
        mut self,
        subphase: TaskWatchdogOwnerSubphase,
    ) -> Self {
        self.task_watchdog_owner_subphase = subphase;
        self
    }

    /// Attaches the coherent store read result and preserves a precise failure.
    #[must_use]
    pub const fn with_task_watchdog_read_outcome(
        mut self,
        outcome: TaskWatchdogReadOutcome,
    ) -> Self {
        self.task_watchdog_read_outcome = outcome;
        match outcome {
            TaskWatchdogReadOutcome::Stable | TaskWatchdogReadOutcome::Uninitialized => self,
            TaskWatchdogReadOutcome::RetryExhausted => {
                self.task_watchdog_participation = TaskWatchdogParticipation::NotParticipating;
                self.maybe_task_watchdog_reason = Some("snapshot_retry_exhausted");
                self.maybe_task_watchdog_feed_sequence = None;
                self.maybe_task_watchdog_feed_age_millis = None;
                self
            }
            TaskWatchdogReadOutcome::HistoryPoisoned => {
                self.task_watchdog_participation = TaskWatchdogParticipation::NotParticipating;
                self.maybe_task_watchdog_reason = Some("snapshot_history_poisoned");
                self.maybe_task_watchdog_feed_sequence = None;
                self.maybe_task_watchdog_feed_age_millis = None;
                self
            }
        }
    }

    /// Returns an observation-only fixture with no authenticated health facts.
    #[must_use]
    pub fn fixture_unavailable() -> Self {
        Self::evaluate(
            PassiveSelfTestState::Unavailable,
            None,
            None,
            None,
            None,
            RuntimeHealthTiming::new(0, 0, 0),
        )
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

    #[must_use]
    pub const fn maybe_task_watchdog_feed_sequence(&self) -> Option<u64> {
        self.maybe_task_watchdog_feed_sequence
    }

    #[must_use]
    pub const fn maybe_task_watchdog_feed_age_millis(&self) -> Option<u64> {
        self.maybe_task_watchdog_feed_age_millis
    }

    #[must_use]
    pub const fn task_watchdog_read_outcome(&self) -> TaskWatchdogReadOutcome {
        self.task_watchdog_read_outcome
    }

    #[must_use]
    pub const fn task_watchdog_owner_phase(&self) -> TaskWatchdogOwnerPhase {
        self.task_watchdog_owner_phase
    }

    #[must_use]
    pub const fn task_watchdog_owner_subphase(&self) -> TaskWatchdogOwnerSubphase {
        self.task_watchdog_owner_subphase
    }

    /// Attaches the state derived from the coherent owner wait observation.
    #[must_use]
    pub const fn with_task_watchdog_wait_state(mut self, state: TaskWatchdogWaitState) -> Self {
        self.task_watchdog_wait_state = state;
        self
    }

    #[must_use]
    pub const fn task_watchdog_wait_state(&self) -> TaskWatchdogWaitState {
        self.task_watchdog_wait_state
    }
}

struct SupervisorProjection {
    availability: SupervisorAvailability,
    maybe_category: Option<String>,
    maybe_sequence: Option<u64>,
    maybe_age_millis: Option<u64>,
    health: CheckpointHealth,
}

fn evaluate_supervisor(
    maybe_previous: Option<&CheckpointObservation>,
    maybe_latest: Option<&CheckpointObservation>,
    now_millis: u64,
    publish_interval_millis: u64,
) -> SupervisorProjection {
    let unavailable = || SupervisorProjection {
        availability: SupervisorAvailability::Unavailable,
        maybe_category: None,
        maybe_sequence: None,
        maybe_age_millis: None,
        health: CheckpointHealth::Unavailable,
    };
    let Some(latest) = maybe_latest else {
        return unavailable();
    };
    if maybe_previous.is_some_and(|previous| latest.validate_after(previous).is_err()) {
        return unavailable();
    }
    let Some(age_millis) = now_millis.checked_sub(latest.observed_at_millis()) else {
        return unavailable();
    };
    let Some(stale_after_millis) = publish_interval_millis.checked_mul(STALE_INTERVAL_MULTIPLIER)
    else {
        return unavailable();
    };
    let Some(unhealthy_after_millis) =
        publish_interval_millis.checked_mul(UNHEALTHY_INTERVAL_MULTIPLIER)
    else {
        return unavailable();
    };
    let health = if age_millis <= stale_after_millis {
        CheckpointHealth::Healthy
    } else if age_millis <= unhealthy_after_millis {
        CheckpointHealth::Stale
    } else {
        CheckpointHealth::Unhealthy
    };
    SupervisorProjection {
        availability: SupervisorAvailability::Available,
        maybe_category: Some(latest.category().to_owned()),
        maybe_sequence: Some(latest.sequence()),
        maybe_age_millis: Some(age_millis),
        health,
    }
}

struct TaskWatchdogProjection {
    participation: TaskWatchdogParticipation,
    reason: &'static str,
    maybe_sequence: Option<u64>,
    maybe_age_millis: Option<u64>,
}

const fn watchdog_projection(
    participation: TaskWatchdogParticipation,
    reason: &'static str,
) -> TaskWatchdogProjection {
    TaskWatchdogProjection {
        participation,
        reason,
        maybe_sequence: None,
        maybe_age_millis: None,
    }
}

fn evaluate_task_watchdog(
    maybe_previous: Option<TaskWatchdogObservation>,
    maybe_latest: Option<TaskWatchdogObservation>,
    now_millis: u64,
    timeout_millis: u64,
) -> TaskWatchdogProjection {
    let Some(latest) = maybe_latest else {
        return watchdog_projection(TaskWatchdogParticipation::Unavailable, "unproved");
    };
    if maybe_previous.is_some_and(|previous| !latest.is_valid_after(previous)) {
        return watchdog_projection(
            TaskWatchdogParticipation::NotParticipating,
            "invalid_observation",
        );
    }
    match latest {
        TaskWatchdogObservation::Unavailable => {
            watchdog_projection(TaskWatchdogParticipation::Unavailable, "unproved")
        }
        TaskWatchdogObservation::SubscriptionFailed => watchdog_projection(
            TaskWatchdogParticipation::NotParticipating,
            "subscription_failed",
        ),
        TaskWatchdogObservation::FeedFailed => {
            watchdog_projection(TaskWatchdogParticipation::NotParticipating, "feed_failed")
        }
        TaskWatchdogObservation::UnsubscriptionFailed => watchdog_projection(
            TaskWatchdogParticipation::NotParticipating,
            "unsubscription_failed",
        ),
        TaskWatchdogObservation::Unsubscribed => {
            watchdog_projection(TaskWatchdogParticipation::NotParticipating, "unsubscribed")
        }
        TaskWatchdogObservation::Fed {
            sequence,
            observed_at_millis,
        } => {
            let Some(age_millis) = now_millis.checked_sub(observed_at_millis) else {
                return watchdog_projection(
                    TaskWatchdogParticipation::NotParticipating,
                    "invalid_observation",
                );
            };
            TaskWatchdogProjection {
                participation: if age_millis <= timeout_millis {
                    TaskWatchdogParticipation::Participating
                } else {
                    TaskWatchdogParticipation::NotParticipating
                },
                reason: if age_millis <= timeout_millis {
                    "feed_fresh"
                } else {
                    "feed_stale"
                },
                maybe_sequence: Some(sequence),
                maybe_age_millis: Some(age_millis),
            }
        }
    }
}

#[cfg(test)]
mod tests;
