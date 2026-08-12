use super::{CampaignStateMarker, CampaignTerminalReasonMarker};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReadinessWakeupMarker {
    Deadline,
    NetworkChanged,
    SettingsChanged,
    ObservationsChanged,
    OperatorIntentChanged,
    ShutdownRequested,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ObservationEpochMarker {
    Initial,
    Advanced,
    Unchanged,
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReadinessSafetySampleMarker {
    Fresh,
    Stale,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReadinessSessionPhaseMarker {
    WaitingForReadiness,
    ConnectingPrimary,
    RunningPrimary,
    ConnectingFallback,
    RunningFallback,
    RecoveryPaused,
    SafeStopping,
    Shutdown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReadinessHardwareStateMarker {
    Unprepared,
    Preparing,
    Ready,
    SafeStopping,
    Stopped,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct ReadinessTransitionMarker {
    pub(super) wakeup: ReadinessWakeupMarker,
    pub(super) previous_blocker: CampaignTerminalReasonMarker,
    pub(super) current_blocker: CampaignTerminalReasonMarker,
    pub(super) session_phase: ReadinessSessionPhaseMarker,
    pub(super) campaign_state: CampaignStateMarker,
    pub(super) hardware_state: ReadinessHardwareStateMarker,
    pub(super) safety_sample: ReadinessSafetySampleMarker,
    pub(super) observation_epoch: ObservationEpochMarker,
    pub(super) pending_observation_recovered: bool,
}
