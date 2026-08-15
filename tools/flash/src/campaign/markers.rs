use super::*;
mod command_effects;
use command_effects::{
    assess_command_effects_terminal, is_recoverable_command_effects_stopped_readiness,
};
mod soak;
use soak::assess_soak_terminal;
mod protocol;
use protocol::ProtocolGateMarker;
mod readiness;
pub(super) use readiness::ReadinessTransitionMarker;
mod pause;
pub(super) use pause::*;

mod asic;
pub(super) use asic::*;

const JOB_TRANSITION_MAXIMUM_MARKER_GAP_MS: u64 = 5_000;

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum CampaignStateMarker {
    Unavailable,
    Armed,
    Preparing,
    Active,
    SafeStopping,
    Consumed,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub(super) enum CampaignProfileMarker {
    None,
    Conservative,
    UpstreamDefault,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum SubmitOutcomeMarker {
    None,
    Accepted,
    Rejected,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum CampaignTerminalReasonMarker {
    None,
    OperatorPaused,
    NetworkUnavailable,
    StratumV1Unsupported,
    SafetyPrerequisitesStale,
    CampaignLeaseUnavailable,
    CampaignLeaseConsumed,
    CampaignActivationTimedOut,
    ProductionAsicUnavailable,
    ProductionAsicVersionMaskUnavailable,
    ProductionAsicDispatchUnavailable,
    ProductionAsicPollUnavailable,
    ProductionAsicQueueFull,
    ProductionAsicWorkerUnavailable,
    ActuationUnqualified,
    PoolConfigurationUnavailable,
    PoolsExhausted,
    JobTransitionProtocolInconsistent,
}

impl CampaignTerminalReasonMarker {
    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::OperatorPaused => "operator_paused",
            Self::NetworkUnavailable => "network_unavailable",
            Self::StratumV1Unsupported => "stratum_v1_unsupported",
            Self::SafetyPrerequisitesStale => "safety_prerequisites_stale",
            Self::CampaignLeaseUnavailable => "campaign_lease_unavailable",
            Self::CampaignLeaseConsumed => "campaign_lease_consumed",
            Self::CampaignActivationTimedOut => "campaign_activation_timed_out",
            Self::ProductionAsicUnavailable => "production_asic_unavailable",
            Self::ProductionAsicVersionMaskUnavailable => {
                "production_asic_version_mask_unavailable"
            }
            Self::ProductionAsicDispatchUnavailable => "production_asic_dispatch_unavailable",
            Self::ProductionAsicPollUnavailable => "production_asic_poll_unavailable",
            Self::ProductionAsicQueueFull => "production_asic_queue_full",
            Self::ProductionAsicWorkerUnavailable => "production_asic_worker_unavailable",
            Self::ActuationUnqualified => "actuation_unqualified",
            Self::PoolConfigurationUnavailable => "pool_configuration_unavailable",
            Self::PoolsExhausted => "pools_exhausted",
            Self::JobTransitionProtocolInconsistent => "job_transition_protocol_inconsistent",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum SafetyMarker {
    Fresh,
    Stale,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum PoolConfigMarker {
    NotRead,
    LocalOwnerSupplied,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum ActuationMarker {
    None,
    Qualified,
    SafeStopped,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum CampaignFailurePhaseMarker {
    None,
    HardwarePreparation,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum CampaignFailureStepMarker {
    None,
    RequireFreshSafetyObservations,
    #[serde(rename = "set_fan_duty_to_100_percent")]
    SetFanDutyTo100Percent,
    RequireFreshNonzeroFanRpm,
    SetCoreVoltage,
    #[serde(rename = "wait_for_core_voltage_stabilization_500_ms")]
    WaitForCoreVoltageStabilization500Ms,
    EnableAsic,
    ResetAndDetectExactlyOneChip,
    InitializeMiningReadyWithFrequencyRamp,
    RetainProductionUart,
    StopDispatch,
    ReduceFrequencyAndResetNonce,
    HoldResetLow,
    DisableCoreVoltage,
    DisableAsic,
    #[serde(rename = "wait_for_fresh_temperature_at_or_below_45_c")]
    WaitForFreshTemperatureAtOrBelow45C,
    #[serde(rename = "set_fan_duty_to_30_percent")]
    SetFanDutyTo30Percent,
}

impl CampaignFailureStepMarker {
    pub(super) fn is_preparation(self) -> bool {
        matches!(
            self,
            Self::RequireFreshSafetyObservations
                | Self::SetFanDutyTo100Percent
                | Self::RequireFreshNonzeroFanRpm
                | Self::SetCoreVoltage
                | Self::WaitForCoreVoltageStabilization500Ms
                | Self::EnableAsic
                | Self::ResetAndDetectExactlyOneChip
                | Self::InitializeMiningReadyWithFrequencyRamp
                | Self::RetainProductionUart
        )
    }

    fn is_safe_shutdown(self) -> bool {
        matches!(
            self,
            Self::StopDispatch
                | Self::ReduceFrequencyAndResetNonce
                | Self::HoldResetLow
                | Self::DisableCoreVoltage
                | Self::DisableAsic
                | Self::SetFanDutyTo100Percent
                | Self::WaitForFreshTemperatureAtOrBelow45C
                | Self::SetFanDutyTo30Percent
        )
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum CampaignFailureDetailMarker {
    None,
    SafetyObservationsUnavailable,
    SafetyOwnerUnavailable,
    SafetyQueueFull,
    SafetyReplyTimedOut,
    SafetyHardwareWriteFailed,
    FanRpmProofTimedOut,
    UnsupportedProfile,
    AsicActuationFailed,
    AsicPlanInvalid,
    CoolingProofTimedOut,
    CoolingProofRequired,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CampaignFailureMarker {
    pub(super) phase: CampaignFailurePhaseMarker,
    pub(super) step: CampaignFailureStepMarker,
    pub(super) detail: CampaignFailureDetailMarker,
    pub(super) rollback_step: CampaignFailureStepMarker,
    pub(super) rollback_detail: CampaignFailureDetailMarker,
}

impl CampaignFailureMarker {
    pub(super) fn is_valid(self) -> bool {
        match self.phase {
            CampaignFailurePhaseMarker::None => {
                self.step == CampaignFailureStepMarker::None
                    && self.detail == CampaignFailureDetailMarker::None
                    && self.rollback_step == CampaignFailureStepMarker::None
                    && self.rollback_detail == CampaignFailureDetailMarker::None
            }
            CampaignFailurePhaseMarker::HardwarePreparation => {
                self.step.is_preparation()
                    && self.detail != CampaignFailureDetailMarker::None
                    && ((self.rollback_step == CampaignFailureStepMarker::None
                        && self.rollback_detail == CampaignFailureDetailMarker::None)
                        || (self.rollback_step.is_safe_shutdown()
                            && self.rollback_detail != CampaignFailureDetailMarker::None))
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ObservationFreshnessMarker {
    pub(super) power_watts: bool,
    pub(super) bus_voltage_volts: bool,
    pub(super) current_amps: bool,
    pub(super) chip_temp_celsius: bool,
    pub(super) vr_temp_celsius: bool,
    pub(super) fan_rpm: bool,
}

impl ObservationFreshnessMarker {
    pub(super) fn fresh_count(self) -> u8 {
        [
            self.power_watts,
            self.bus_voltage_volts,
            self.current_amps,
            self.chip_temp_celsius,
            self.vr_temp_celsius,
            self.fan_rpm,
        ]
        .into_iter()
        .filter(|fresh| *fresh)
        .count() as u8
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct ObservationRequirementsMarker {
    pub(super) power_watts: bool,
    pub(super) bus_voltage_volts: bool,
    pub(super) current_amps: bool,
    pub(super) chip_temp_celsius: bool,
    pub(super) vr_temp_celsius: bool,
    pub(super) fan_rpm: bool,
}

impl ObservationRequirementsMarker {
    pub(super) const ULTRA_205: Self = Self {
        power_watts: true,
        bus_voltage_volts: true,
        current_amps: true,
        chip_temp_celsius: true,
        vr_temp_celsius: false,
        fan_rpm: true,
    };

    fn is_satisfied_by(self, freshness: ObservationFreshnessMarker) -> bool {
        (!self.power_watts || freshness.power_watts)
            && (!self.bus_voltage_volts || freshness.bus_voltage_volts)
            && (!self.current_amps || freshness.current_amps)
            && (!self.chip_temp_celsius || freshness.chip_temp_celsius)
            && (!self.vr_temp_celsius || freshness.vr_temp_celsius)
            && (!self.fan_rpm || freshness.fan_rpm)
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct CampaignStatusMarker {
    pub(super) schema: String,
    pub(super) stage: MiningCampaignStage,
    pub(super) lease_id: Option<u64>,
    pub(super) campaign_state: CampaignStateMarker,
    pub(super) profile: CampaignProfileMarker,
    pub(super) active_ms: u64,
    pub(super) submit_outcome: SubmitOutcomeMarker,
    pub(super) qualified_candidate_count: u64,
    pub(super) below_pool_target_count: u64,
    pub(super) duplicate_candidate_count: u64,
    pub(super) accepted_share_count: u64,
    pub(super) rejected_share_count: u64,
    pub(super) job_transition: JobTransitionMarker,
    pub(super) asic_bridge: AsicBridgeMarker,
    pub(super) terminal_reason: CampaignTerminalReasonMarker,
    pub(super) protocol_gate: ProtocolGateMarker,
    pub(super) readiness_transition: ReadinessTransitionMarker,
    pub(super) resumable_pause_safe_stop: ResumablePauseSafeStopMarker,
    pub(super) safety: SafetyMarker,
    pub(super) fresh_observation_count: u8,
    pub(super) observation_freshness: ObservationFreshnessMarker,
    pub(super) observation_requirements: ObservationRequirementsMarker,
    pub(super) pool_config: PoolConfigMarker,
    pub(super) pool_config_persisted: bool,
    pub(super) actuation: ActuationMarker,
    pub(super) mineonboot: bool,
    pub(super) safe_stop: SafeStopMarker,
    pub(super) failure: CampaignFailureMarker,
}

#[derive(Clone, Debug, Default, Serialize)]
pub(super) struct CampaignMarkerAggregate {
    pub(super) marker_count: u64,
    pub(super) maximum_active_marker_gap_ms: u64,
    pub(super) terminal: Option<CampaignStatusMarker>,
    pub(super) asic_event_trace: CampaignAsicEventTrace,
    #[serde(skip)]
    pub(super) maybe_failure_category: Option<CampaignTerminalCategory>,
    #[serde(skip)]
    pub(super) failure_observation_freshness: Option<ObservationFreshnessMarker>,
    #[serde(skip)]
    maybe_previous_active_ms: Option<u64>,
}

impl CampaignMarkerAggregate {
    pub(super) fn observe(
        &mut self,
        marker: CampaignStatusMarker,
        admission: CampaignAdmission,
    ) -> Option<CampaignTerminalCategory> {
        self.marker_count = self.marker_count.saturating_add(1);
        self.asic_event_trace
            .observe(marker.asic_bridge.latest_event);
        if matches!(
            marker.campaign_state,
            CampaignStateMarker::Active | CampaignStateMarker::SafeStopping
        ) {
            if let Some(previous) = self.maybe_previous_active_ms {
                self.maximum_active_marker_gap_ms = self
                    .maximum_active_marker_gap_ms
                    .max(marker.active_ms.saturating_sub(previous));
            }
            self.maybe_previous_active_ms = Some(marker.active_ms);
        }
        let maybe_failure = campaign_marker_failure(&marker, admission);
        if let Some(category) = maybe_failure {
            if self.maybe_failure_category.is_none() {
                self.maybe_failure_category = Some(category);
                self.failure_observation_freshness = Some(marker.observation_freshness);
            }
        }
        self.terminal = Some(marker);
        maybe_failure
    }

    pub(super) fn assess(
        &self,
        admission: CampaignAdmission,
    ) -> std::result::Result<CampaignTerminalCategory, CampaignFailure> {
        let Some(terminal) = self.terminal.as_ref() else {
            return Err(CampaignFailure::new(
                CampaignTerminalCategory::MarkerMissing,
            ));
        };
        if let Some(category) = self.maybe_failure_category {
            return Err(CampaignFailure::new(category));
        }
        if !terminal
            .observation_requirements
            .is_satisfied_by(terminal.observation_freshness)
        {
            return Err(CampaignFailure::new(
                CampaignTerminalCategory::ObservationContractIncomplete,
            ));
        }
        if admission.stage == MiningCampaignStage::JobTransition
            && self.maximum_active_marker_gap_ms > JOB_TRANSITION_MAXIMUM_MARKER_GAP_MS
        {
            return Err(CampaignFailure::new(
                CampaignTerminalCategory::MarkerContinuityFailed,
            ));
        }
        match admission.stage {
            MiningCampaignStage::Observation => assess_observation_terminal(terminal),
            MiningCampaignStage::LiveShare => assess_live_share_terminal(terminal),
            MiningCampaignStage::Soak => assess_soak_terminal(terminal, admission.duration_seconds),
            MiningCampaignStage::JobTransition => {
                assess_job_transition_terminal(terminal, admission.duration_seconds)
            }
            MiningCampaignStage::CommandEffects => assess_command_effects_terminal(terminal),
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum JobTransitionStateMarker {
    NotObserved,
    ReplacementQueued,
    ReplacementDispatched,
    ReplacementResultCorrelated,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct JobTransitionMarker {
    pub(super) pool_notify_count: u64,
    pub(super) clean_jobs_notify_count: u64,
    pub(super) previous_block_change_count: u64,
    pub(super) new_block_generation_count: u64,
    pub(super) replacement_dispatch_count: u64,
    pub(super) post_transition_correlated_result_count: u64,
    pub(super) completed_transition_count: u64,
    pub(super) stale_generation_result_discard_count: u64,
    pub(super) stale_generation_submit_count: u64,
    pub(super) reconnect_count: u64,
    pub(super) latest_state: JobTransitionStateMarker,
}

pub(super) fn campaign_marker_failure(
    marker: &CampaignStatusMarker,
    admission: CampaignAdmission,
) -> Option<CampaignTerminalCategory> {
    if marker.stage != admission.stage {
        return Some(CampaignTerminalCategory::StageMismatch);
    }
    if marker.mineonboot {
        return Some(CampaignTerminalCategory::MineOnBootEnabled);
    }
    if !marker.resumable_pause_safe_stop.is_valid_for(marker.stage) {
        return Some(CampaignTerminalCategory::MarkerInvalid);
    }
    if marker.safety == SafetyMarker::Stale
        && !is_recoverable_command_effects_stopped_readiness(marker, admission)
    {
        return Some(CampaignTerminalCategory::SafetyStale);
    }
    if marker.failure.phase == CampaignFailurePhaseMarker::HardwarePreparation {
        return Some(CampaignTerminalCategory::HardwarePreparationFailed);
    }
    match admission.stage {
        MiningCampaignStage::Observation => {
            if marker.lease_id.is_some() {
                return Some(CampaignTerminalCategory::LeaseMismatch);
            }
            if marker.profile != CampaignProfileMarker::None {
                return Some(CampaignTerminalCategory::ProfileMismatch);
            }
            if marker.pool_config != PoolConfigMarker::NotRead {
                return Some(CampaignTerminalCategory::PoolReadDuringObservation);
            }
            if marker.actuation != ActuationMarker::None {
                return Some(CampaignTerminalCategory::ActuationDuringObservation);
            }
        }
        MiningCampaignStage::LiveShare
        | MiningCampaignStage::Soak
        | MiningCampaignStage::JobTransition
        | MiningCampaignStage::CommandEffects => {
            let terminal_consumed = marker.campaign_state == CampaignStateMarker::Consumed
                && marker.actuation == ActuationMarker::SafeStopped
                && marker.safe_stop == SafeStopMarker::Confirmed;
            if marker.lease_id != admission.maybe_lease_id
                && !(terminal_consumed && marker.lease_id.is_none())
            {
                return Some(CampaignTerminalCategory::LeaseMismatch);
            }
            if marker.profile != expected_profile_marker(admission) {
                return Some(CampaignTerminalCategory::ProfileMismatch);
            }
            if marker.campaign_state == CampaignStateMarker::Consumed {
                let terminal_result = match admission.stage {
                    MiningCampaignStage::LiveShare => assess_live_share_terminal(marker),
                    MiningCampaignStage::Soak => {
                        assess_soak_terminal(marker, admission.duration_seconds)
                    }
                    MiningCampaignStage::JobTransition => {
                        assess_job_transition_terminal(marker, admission.duration_seconds)
                    }
                    MiningCampaignStage::CommandEffects => assess_command_effects_terminal(marker),
                    MiningCampaignStage::Observation => unreachable!("mining stage"),
                };
                if let Err(failure) = terminal_result {
                    return Some(failure.category);
                }
            }
        }
    }
    None
}

fn assess_job_transition_terminal(
    marker: &CampaignStatusMarker,
    duration_seconds: u64,
) -> std::result::Result<CampaignTerminalCategory, CampaignFailure> {
    assess_mining_terminal(marker)?;
    if marker.active_ms < duration_seconds.saturating_mul(1_000) {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::SoakDurationShort,
        ));
    }
    if marker.terminal_reason == CampaignTerminalReasonMarker::JobTransitionProtocolInconsistent {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::JobTransitionProtocolInconsistent,
        ));
    }
    if marker.rejected_share_count > 0 {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::RejectedShareObserved,
        ));
    }
    if marker.job_transition.stale_generation_submit_count > 0 {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::StaleGenerationSubmissionObserved,
        ));
    }
    if marker.job_transition.reconnect_count > 0 {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::ReconnectObserved,
        ));
    }
    let transition = marker.job_transition;
    if transition.previous_block_change_count == 0 {
        if transition.new_block_generation_count == 0
            && transition.replacement_dispatch_count == 0
            && transition.post_transition_correlated_result_count == 0
            && transition.completed_transition_count == 0
            && transition.latest_state == JobTransitionStateMarker::NotObserved
        {
            return Ok(CampaignTerminalCategory::JobTransitionNotObserved);
        }
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::JobTransitionEvidenceIncomplete,
        ));
    }
    if transition.clean_jobs_notify_count < transition.previous_block_change_count
        || transition.new_block_generation_count != transition.previous_block_change_count
        || transition.completed_transition_count > transition.new_block_generation_count
        || transition.completed_transition_count > transition.replacement_dispatch_count
        || transition.completed_transition_count
            > transition.post_transition_correlated_result_count
    {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::JobTransitionEvidenceIncomplete,
        ));
    }
    if transition.new_block_generation_count == 0
        || transition.replacement_dispatch_count == 0
        || transition.post_transition_correlated_result_count == 0
        || transition.completed_transition_count == 0
        || marker.asic_bridge.post_transition_poll_request_count == 0
        || marker.asic_bridge.post_transition_completion_count == 0
        || marker.asic_bridge.post_transition_nonce_emission_count == 0
        || marker.asic_bridge.post_transition_correlation_count == 0
        || marker.asic_bridge.final_poll_state == AsicPollStateMarker::InFlight
    {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::JobTransitionEvidenceIncomplete,
        ));
    }
    Ok(CampaignTerminalCategory::JobTransitionComplete)
}

fn expected_profile_marker(admission: CampaignAdmission) -> CampaignProfileMarker {
    match admission.maybe_profile {
        Some(MiningCampaignProfile::Conservative) => CampaignProfileMarker::Conservative,
        Some(MiningCampaignProfile::UpstreamDefault) => CampaignProfileMarker::UpstreamDefault,
        None => CampaignProfileMarker::None,
    }
}

fn assess_observation_terminal(
    marker: &CampaignStatusMarker,
) -> std::result::Result<CampaignTerminalCategory, CampaignFailure> {
    if marker.campaign_state != CampaignStateMarker::Unavailable
        || marker.active_ms != 0
        || marker.submit_outcome != SubmitOutcomeMarker::None
        || marker.safe_stop != SafeStopMarker::NotRequired
    {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::ObservationContractIncomplete,
        ));
    }
    Ok(CampaignTerminalCategory::ObservationComplete)
}

fn assess_live_share_terminal(
    marker: &CampaignStatusMarker,
) -> std::result::Result<CampaignTerminalCategory, CampaignFailure> {
    assess_mining_terminal(marker)?;
    if marker.submit_outcome == SubmitOutcomeMarker::None {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::SubmitResponseMissing,
        ));
    }
    Ok(CampaignTerminalCategory::SubmitResponseObserved)
}

fn assess_mining_terminal(
    marker: &CampaignStatusMarker,
) -> std::result::Result<(), CampaignFailure> {
    if marker.pool_config != PoolConfigMarker::LocalOwnerSupplied {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::PoolConfigurationMissing,
        ));
    }
    if marker.campaign_state != CampaignStateMarker::Consumed
        || marker.safe_stop != SafeStopMarker::Confirmed
        || marker.actuation != ActuationMarker::SafeStopped
    {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::SafeStopUnconfirmed,
        ));
    }
    Ok(())
}
