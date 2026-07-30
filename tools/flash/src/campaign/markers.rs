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
    pub(super) safety: SafetyMarker,
    pub(super) fresh_observation_count: u8,
    pub(super) observation_freshness: ObservationFreshnessMarker,
    pub(super) pool_config: PoolConfigMarker,
    pub(super) actuation: ActuationMarker,
    pub(super) mineonboot: bool,
    pub(super) safe_stop: SafeStopMarker,
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
    if terminal.fresh_observation_count < 6 {
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
