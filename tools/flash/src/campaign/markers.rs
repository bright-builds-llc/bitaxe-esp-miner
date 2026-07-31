use super::*;

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
    ProductionAsicUnavailable,
    ProductionAsicVersionMaskUnavailable,
    ProductionAsicDispatchUnavailable,
    ProductionAsicPollUnavailable,
    ProductionAsicQueueFull,
    ProductionAsicWorkerUnavailable,
    ActuationUnqualified,
    PoolConfigurationUnavailable,
    PoolsExhausted,
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
pub(super) enum SafeStopMarker {
    NotRequired,
    Pending,
    Confirmed,
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
    pub(super) terminal_reason: CampaignTerminalReasonMarker,
    pub(super) safety: SafetyMarker,
    pub(super) fresh_observation_count: u8,
    pub(super) observation_freshness: ObservationFreshnessMarker,
    pub(super) observation_requirements: ObservationRequirementsMarker,
    pub(super) pool_config: PoolConfigMarker,
    pub(super) actuation: ActuationMarker,
    pub(super) mineonboot: bool,
    pub(super) safe_stop: SafeStopMarker,
    pub(super) failure: CampaignFailureMarker,
}

pub(super) fn assess_campaign_markers(
    markers: &[CampaignStatusMarker],
    admission: CampaignAdmission,
) -> std::result::Result<CampaignTerminalCategory, CampaignFailure> {
    if markers.is_empty() {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::MarkerMissing,
        ));
    }
    if let Some(category) = first_campaign_marker_failure(markers, admission) {
        return Err(CampaignFailure::new(category));
    }
    let terminal = markers.last().expect("nonempty campaign marker set");
    if !terminal
        .observation_requirements
        .is_satisfied_by(terminal.observation_freshness)
    {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::ObservationContractIncomplete,
        ));
    }
    match admission.stage {
        MiningCampaignStage::Observation => assess_observation_terminal(terminal),
        MiningCampaignStage::LiveShare => assess_live_share_terminal(terminal),
        MiningCampaignStage::Soak => assess_soak_terminal(terminal, admission.duration_seconds),
    }
}

pub(super) fn first_campaign_marker_failure(
    markers: &[CampaignStatusMarker],
    admission: CampaignAdmission,
) -> Option<CampaignTerminalCategory> {
    markers
        .iter()
        .find_map(|marker| campaign_marker_failure(marker, admission))
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
    if marker.safety == SafetyMarker::Stale {
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
        MiningCampaignStage::LiveShare | MiningCampaignStage::Soak => {
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

fn assess_soak_terminal(
    marker: &CampaignStatusMarker,
    duration_seconds: u64,
) -> std::result::Result<CampaignTerminalCategory, CampaignFailure> {
    assess_mining_terminal(marker)?;
    if marker.submit_outcome == SubmitOutcomeMarker::None {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::SubmitResponseMissing,
        ));
    }
    if marker.active_ms < duration_seconds.saturating_mul(1_000) {
        return Err(CampaignFailure::new(
            CampaignTerminalCategory::SoakDurationShort,
        ));
    }
    Ok(CampaignTerminalCategory::SoakDurationComplete)
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
