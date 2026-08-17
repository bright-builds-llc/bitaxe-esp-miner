/// Closed result of copying the firmware-owned watchdog observation store.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaskWatchdogReadOutcome {
    Stable,
    Uninitialized,
    RetryExhausted,
    HistoryPoisoned,
}

impl TaskWatchdogReadOutcome {
    /// Returns the exact serialized spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stable => "stable",
            Self::Uninitialized => "uninitialized",
            Self::RetryExhausted => "retry_exhausted",
            Self::HistoryPoisoned => "history_poisoned",
        }
    }
}

/// Closed phase vocabulary for the task-watchdog-owned production session loop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TaskWatchdogOwnerPhase {
    Unavailable = 0,
    Subscribing = 1,
    LoopStart = 2,
    WaitingInbox = 3,
    HandlingInbox = 4,
    HandlingObservation = 5,
    HandlingReadiness = 6,
    PublishingCampaignStatus = 7,
    ServicingHashrate = 8,
    Shutdown = 9,
}

impl TaskWatchdogOwnerPhase {
    /// Returns the exact serialized spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::Subscribing => "subscribing",
            Self::LoopStart => "loop_start",
            Self::WaitingInbox => "waiting_inbox",
            Self::HandlingInbox => "handling_inbox",
            Self::HandlingObservation => "handling_observation",
            Self::HandlingReadiness => "handling_readiness",
            Self::PublishingCampaignStatus => "publishing_campaign_status",
            Self::ServicingHashrate => "servicing_hashrate",
            Self::Shutdown => "shutdown",
        }
    }

    /// Decodes the lock-free firmware representation without accepting free text.
    #[must_use]
    pub const fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Subscribing,
            2 => Self::LoopStart,
            3 => Self::WaitingInbox,
            4 => Self::HandlingInbox,
            5 => Self::HandlingObservation,
            6 => Self::HandlingReadiness,
            7 => Self::PublishingCampaignStatus,
            8 => Self::ServicingHashrate,
            9 => Self::Shutdown,
            _ => Self::Unavailable,
        }
    }
}

/// Closed, value-free work boundary within a production-owner phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TaskWatchdogOwnerSubphase {
    Unavailable = 0,
    InboxMapping = 1,
    SessionEvaluation = 2,
    EffectPrepareHardware = 3,
    EffectReadPoolConfiguration = 4,
    EffectConnectPool = 5,
    EffectWritePoolLine = 6,
    EffectApplyVersionMask = 7,
    EffectDispatchChip = 8,
    EffectPollChip = 9,
    EffectBlockSubmissions = 10,
    EffectInvalidateWorkAndSubmissions = 11,
    EffectStopChipInteraction = 12,
    EffectClosePoolConnection = 13,
    EffectSafeStopHardware = 14,
    EffectRecordScoreboard = 15,
    EffectRecordBlockFound = 16,
    EffectPublish = 17,
    SafeStopStopDispatch = 18,
    SafeStopReduceFrequencyAndNonceState = 19,
    SafeStopAssertControlLineLow = 20,
    SafeStopDisableCoreRail = 21,
    SafeStopDisableChip = 22,
    SafeStopSetCoolingMaximum = 23,
    SafeStopWaitForCoolingProof = 24,
    SafeStopSetCoolingPaused = 25,
}

impl TaskWatchdogOwnerSubphase {
    /// Returns the exact serialized spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Unavailable => "unavailable",
            Self::InboxMapping => "inbox_mapping",
            Self::SessionEvaluation => "session_evaluation",
            Self::EffectPrepareHardware => "effect_prepare_hardware",
            Self::EffectReadPoolConfiguration => "effect_read_pool_configuration",
            Self::EffectConnectPool => "effect_connect_pool",
            Self::EffectWritePoolLine => "effect_write_pool_line",
            Self::EffectApplyVersionMask => "effect_apply_version_mask",
            Self::EffectDispatchChip => "effect_dispatch_chip",
            Self::EffectPollChip => "effect_poll_chip",
            Self::EffectBlockSubmissions => "effect_block_submissions",
            Self::EffectInvalidateWorkAndSubmissions => "effect_invalidate_work_and_submissions",
            Self::EffectStopChipInteraction => "effect_stop_chip_interaction",
            Self::EffectClosePoolConnection => "effect_close_pool_connection",
            Self::EffectSafeStopHardware => "effect_safe_stop_hardware",
            Self::EffectRecordScoreboard => "effect_record_scoreboard",
            Self::EffectRecordBlockFound => "effect_record_block_found",
            Self::EffectPublish => "effect_publish",
            Self::SafeStopStopDispatch => "safe_stop_stop_dispatch",
            Self::SafeStopReduceFrequencyAndNonceState => {
                "safe_stop_reduce_frequency_and_nonce_state"
            }
            Self::SafeStopAssertControlLineLow => "safe_stop_assert_control_line_low",
            Self::SafeStopDisableCoreRail => "safe_stop_disable_core_rail",
            Self::SafeStopDisableChip => "safe_stop_disable_chip",
            Self::SafeStopSetCoolingMaximum => "safe_stop_set_cooling_maximum",
            Self::SafeStopWaitForCoolingProof => "safe_stop_wait_for_cooling_proof",
            Self::SafeStopSetCoolingPaused => "safe_stop_set_cooling_paused",
        }
    }

    /// Decodes the lock-free firmware representation without accepting free text.
    #[must_use]
    pub const fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::InboxMapping,
            2 => Self::SessionEvaluation,
            3 => Self::EffectPrepareHardware,
            4 => Self::EffectReadPoolConfiguration,
            5 => Self::EffectConnectPool,
            6 => Self::EffectWritePoolLine,
            7 => Self::EffectApplyVersionMask,
            8 => Self::EffectDispatchChip,
            9 => Self::EffectPollChip,
            10 => Self::EffectBlockSubmissions,
            11 => Self::EffectInvalidateWorkAndSubmissions,
            12 => Self::EffectStopChipInteraction,
            13 => Self::EffectClosePoolConnection,
            14 => Self::EffectSafeStopHardware,
            15 => Self::EffectRecordScoreboard,
            16 => Self::EffectRecordBlockFound,
            17 => Self::EffectPublish,
            18 => Self::SafeStopStopDispatch,
            19 => Self::SafeStopReduceFrequencyAndNonceState,
            20 => Self::SafeStopAssertControlLineLow,
            21 => Self::SafeStopDisableCoreRail,
            22 => Self::SafeStopDisableChip,
            23 => Self::SafeStopSetCoolingMaximum,
            24 => Self::SafeStopWaitForCoolingProof,
            25 => Self::SafeStopSetCoolingPaused,
            _ => Self::Unavailable,
        }
    }
}

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
    fn owner_subphases_have_exact_closed_spellings_and_encoding() {
        // Arrange
        let cases = [
            (TaskWatchdogOwnerSubphase::Unavailable, "unavailable"),
            (TaskWatchdogOwnerSubphase::InboxMapping, "inbox_mapping"),
            (
                TaskWatchdogOwnerSubphase::SessionEvaluation,
                "session_evaluation",
            ),
            (
                TaskWatchdogOwnerSubphase::EffectPrepareHardware,
                "effect_prepare_hardware",
            ),
            (
                TaskWatchdogOwnerSubphase::EffectReadPoolConfiguration,
                "effect_read_pool_configuration",
            ),
            (
                TaskWatchdogOwnerSubphase::EffectConnectPool,
                "effect_connect_pool",
            ),
            (
                TaskWatchdogOwnerSubphase::EffectWritePoolLine,
                "effect_write_pool_line",
            ),
            (
                TaskWatchdogOwnerSubphase::EffectApplyVersionMask,
                "effect_apply_version_mask",
            ),
            (
                TaskWatchdogOwnerSubphase::EffectDispatchChip,
                "effect_dispatch_chip",
            ),
            (
                TaskWatchdogOwnerSubphase::EffectPollChip,
                "effect_poll_chip",
            ),
            (
                TaskWatchdogOwnerSubphase::EffectBlockSubmissions,
                "effect_block_submissions",
            ),
            (
                TaskWatchdogOwnerSubphase::EffectInvalidateWorkAndSubmissions,
                "effect_invalidate_work_and_submissions",
            ),
            (
                TaskWatchdogOwnerSubphase::EffectStopChipInteraction,
                "effect_stop_chip_interaction",
            ),
            (
                TaskWatchdogOwnerSubphase::EffectClosePoolConnection,
                "effect_close_pool_connection",
            ),
            (
                TaskWatchdogOwnerSubphase::EffectSafeStopHardware,
                "effect_safe_stop_hardware",
            ),
            (
                TaskWatchdogOwnerSubphase::EffectRecordScoreboard,
                "effect_record_scoreboard",
            ),
            (
                TaskWatchdogOwnerSubphase::EffectRecordBlockFound,
                "effect_record_block_found",
            ),
            (TaskWatchdogOwnerSubphase::EffectPublish, "effect_publish"),
            (
                TaskWatchdogOwnerSubphase::SafeStopStopDispatch,
                "safe_stop_stop_dispatch",
            ),
            (
                TaskWatchdogOwnerSubphase::SafeStopReduceFrequencyAndNonceState,
                "safe_stop_reduce_frequency_and_nonce_state",
            ),
            (
                TaskWatchdogOwnerSubphase::SafeStopAssertControlLineLow,
                "safe_stop_assert_control_line_low",
            ),
            (
                TaskWatchdogOwnerSubphase::SafeStopDisableCoreRail,
                "safe_stop_disable_core_rail",
            ),
            (
                TaskWatchdogOwnerSubphase::SafeStopDisableChip,
                "safe_stop_disable_chip",
            ),
            (
                TaskWatchdogOwnerSubphase::SafeStopSetCoolingMaximum,
                "safe_stop_set_cooling_maximum",
            ),
            (
                TaskWatchdogOwnerSubphase::SafeStopWaitForCoolingProof,
                "safe_stop_wait_for_cooling_proof",
            ),
            (
                TaskWatchdogOwnerSubphase::SafeStopSetCoolingPaused,
                "safe_stop_set_cooling_paused",
            ),
        ];

        // Act / Assert
        for (subphase, expected) in cases {
            assert_eq!(subphase.as_str(), expected);
            assert_eq!(TaskWatchdogOwnerSubphase::from_u8(subphase as u8), subphase);
        }
        assert_eq!(
            TaskWatchdogOwnerSubphase::from_u8(u8::MAX),
            TaskWatchdogOwnerSubphase::Unavailable
        );
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
