#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) enum WatchdogOwnerSubphase {
    #[default]
    Unavailable,
    InboxMapping,
    SessionEvaluation,
    EffectPrepareHardware,
    EffectReadPoolConfiguration,
    EffectConnectPool,
    EffectWritePoolLine,
    EffectApplyVersionMask,
    EffectDispatchChip,
    EffectPollChip,
    EffectBlockSubmissions,
    EffectInvalidateWorkAndSubmissions,
    EffectStopChipInteraction,
    EffectClosePoolConnection,
    EffectSafeStopHardware,
    EffectRecordScoreboard,
    EffectRecordBlockFound,
    EffectPublish,
    SafeStopStopDispatch,
    SafeStopReduceFrequencyAndNonceState,
    SafeStopAssertControlLineLow,
    SafeStopDisableCoreRail,
    SafeStopDisableChip,
    SafeStopSetCoolingMaximum,
    SafeStopWaitForCoolingProof,
    SafeStopSetCoolingPaused,
}

impl WatchdogOwnerSubphase {
    pub(crate) const fn label(self) -> &'static str {
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

    pub(crate) fn parse(value: &str) -> Option<Self> {
        match value {
            "unavailable" => Some(Self::Unavailable),
            "inbox_mapping" => Some(Self::InboxMapping),
            "session_evaluation" => Some(Self::SessionEvaluation),
            "effect_prepare_hardware" => Some(Self::EffectPrepareHardware),
            "effect_read_pool_configuration" => Some(Self::EffectReadPoolConfiguration),
            "effect_connect_pool" => Some(Self::EffectConnectPool),
            "effect_write_pool_line" => Some(Self::EffectWritePoolLine),
            "effect_apply_version_mask" => Some(Self::EffectApplyVersionMask),
            "effect_dispatch_chip" => Some(Self::EffectDispatchChip),
            "effect_poll_chip" => Some(Self::EffectPollChip),
            "effect_block_submissions" => Some(Self::EffectBlockSubmissions),
            "effect_invalidate_work_and_submissions" => {
                Some(Self::EffectInvalidateWorkAndSubmissions)
            }
            "effect_stop_chip_interaction" => Some(Self::EffectStopChipInteraction),
            "effect_close_pool_connection" => Some(Self::EffectClosePoolConnection),
            "effect_safe_stop_hardware" => Some(Self::EffectSafeStopHardware),
            "effect_record_scoreboard" => Some(Self::EffectRecordScoreboard),
            "effect_record_block_found" => Some(Self::EffectRecordBlockFound),
            "effect_publish" => Some(Self::EffectPublish),
            "safe_stop_stop_dispatch" => Some(Self::SafeStopStopDispatch),
            "safe_stop_reduce_frequency_and_nonce_state" => {
                Some(Self::SafeStopReduceFrequencyAndNonceState)
            }
            "safe_stop_assert_control_line_low" => Some(Self::SafeStopAssertControlLineLow),
            "safe_stop_disable_core_rail" => Some(Self::SafeStopDisableCoreRail),
            "safe_stop_disable_chip" => Some(Self::SafeStopDisableChip),
            "safe_stop_set_cooling_maximum" => Some(Self::SafeStopSetCoolingMaximum),
            "safe_stop_wait_for_cooling_proof" => Some(Self::SafeStopWaitForCoolingProof),
            "safe_stop_set_cooling_paused" => Some(Self::SafeStopSetCoolingPaused),
            _ => None,
        }
    }
}
