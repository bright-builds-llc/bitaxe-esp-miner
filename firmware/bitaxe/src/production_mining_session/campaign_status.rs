//! Redacted retained status projection for repo-owned mining campaigns.

use bitaxe_stratum::v1::production_session::{
    JobTransitionEvidence, MiningCampaignLease, MiningCampaignState, MiningHardwareProfilePreset,
    ProductionSessionSnapshot,
};
use bitaxe_stratum::v1::state::MiningOperatorIntent;
use serde::Serialize;

use crate::settings_adapter::MiningCampaignStage;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum PoolConfigurationStatus {
    NotRead,
    LocalOwnerSupplied,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CampaignActuationStatus {
    None,
    Qualified,
    SafeStopped,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CampaignSafeStopStatus {
    NotRequired,
    Pending,
    Confirmed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(super) struct CampaignFailureDiagnostic {
    phase: &'static str,
    step: &'static str,
    detail: &'static str,
    rollback_step: &'static str,
    rollback_detail: &'static str,
}

impl CampaignFailureDiagnostic {
    const NONE: Self = Self {
        phase: "none",
        step: "none",
        detail: "none",
        rollback_step: "none",
        rollback_detail: "none",
    };
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(super) struct CampaignObservationFreshness {
    pub(super) power_watts: bool,
    pub(super) bus_voltage_volts: bool,
    pub(super) current_amps: bool,
    pub(super) chip_temp_celsius: bool,
    pub(super) vr_temp_celsius: bool,
    pub(super) fan_rpm: bool,
}

impl CampaignObservationFreshness {
    #[cfg(test)]
    pub(super) const fn all_ultra205_supported_fresh() -> Self {
        Self {
            power_watts: true,
            bus_voltage_volts: true,
            current_amps: true,
            chip_temp_celsius: true,
            vr_temp_celsius: false,
            fan_rpm: true,
        }
    }

    fn fresh_count(self) -> u8 {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub(super) struct CampaignObservationRequirements {
    pub(super) power_watts: bool,
    pub(super) bus_voltage_volts: bool,
    pub(super) current_amps: bool,
    pub(super) chip_temp_celsius: bool,
    pub(super) vr_temp_celsius: bool,
    pub(super) fan_rpm: bool,
}

impl CampaignObservationRequirements {
    const ULTRA_205: Self = Self {
        power_watts: true,
        bus_voltage_volts: true,
        current_amps: true,
        chip_temp_celsius: true,
        vr_temp_celsius: false,
        fan_rpm: true,
    };
}

pub(super) struct CampaignStatusTracker {
    stage: MiningCampaignStage,
    retained_maybe_lease: Option<MiningCampaignLease>,
    lease_authorizing: bool,
    maybe_profile: Option<MiningHardwareProfilePreset>,
    maybe_active_since_ms: Option<u64>,
    retained_active_ms: u64,
    pool_config: PoolConfigurationStatus,
    actuation: CampaignActuationStatus,
    safe_stop: CampaignSafeStopStatus,
    failure: CampaignFailureDiagnostic,
}

impl CampaignStatusTracker {
    pub(super) fn new(
        stage: MiningCampaignStage,
        maybe_lease: Option<MiningCampaignLease>,
        maybe_profile: Option<MiningHardwareProfilePreset>,
    ) -> Self {
        Self {
            stage,
            retained_maybe_lease: maybe_lease,
            lease_authorizing: maybe_lease.is_some(),
            maybe_profile,
            maybe_active_since_ms: None,
            retained_active_ms: 0,
            pool_config: PoolConfigurationStatus::NotRead,
            actuation: if maybe_lease.is_some() {
                CampaignActuationStatus::Qualified
            } else {
                CampaignActuationStatus::None
            },
            safe_stop: CampaignSafeStopStatus::NotRequired,
            failure: CampaignFailureDiagnostic::NONE,
        }
    }

    pub(super) const fn maybe_lease(&self) -> Option<MiningCampaignLease> {
        if self.lease_authorizing {
            self.retained_maybe_lease
        } else {
            None
        }
    }

    pub(super) const fn authorizes_actuation(&self) -> bool {
        self.lease_authorizing
    }

    pub(super) const fn operator_intent(
        &self,
        _persisted_intent: MiningOperatorIntent,
    ) -> MiningOperatorIntent {
        if self.lease_authorizing {
            MiningOperatorIntent::Run
        } else {
            MiningOperatorIntent::Paused
        }
    }

    pub(super) fn note_pool_configuration_read(&mut self, available: bool) {
        if available {
            self.pool_config = PoolConfigurationStatus::LocalOwnerSupplied;
        }
    }

    pub(super) fn note_safe_stop_pending(&mut self) {
        self.safe_stop = CampaignSafeStopStatus::Pending;
    }

    pub(super) fn note_failure(
        &mut self,
        phase: &'static str,
        step: &'static str,
        detail: &'static str,
        rollback_step: &'static str,
        rollback_detail: &'static str,
    ) {
        if self.failure == CampaignFailureDiagnostic::NONE {
            self.failure = CampaignFailureDiagnostic {
                phase,
                step,
                detail,
                rollback_step,
                rollback_detail,
            };
        }
    }

    pub(super) fn note_snapshot(&mut self, snapshot: &ProductionSessionSnapshot, now_ms: u64) {
        if snapshot.campaign_state == MiningCampaignState::Active {
            let active_since = *self.maybe_active_since_ms.get_or_insert(now_ms);
            self.retained_active_ms = now_ms.saturating_sub(active_since);
        } else if let Some(active_since) = self.maybe_active_since_ms.take() {
            self.retained_active_ms = self
                .retained_active_ms
                .max(now_ms.saturating_sub(active_since));
        }

        if snapshot.campaign_state == MiningCampaignState::Consumed {
            self.lease_authorizing = false;
            self.actuation = CampaignActuationStatus::SafeStopped;
            self.safe_stop = CampaignSafeStopStatus::Confirmed;
        }
    }

    pub(super) fn marker(
        &self,
        snapshot: &ProductionSessionSnapshot,
        now_ms: u64,
        safety_fresh: bool,
        observation_freshness: CampaignObservationFreshness,
        mineonboot: bool,
    ) -> String {
        let active_ms = self
            .maybe_active_since_ms
            .map_or(self.retained_active_ms, |started| {
                self.retained_active_ms.max(now_ms.saturating_sub(started))
            });
        let projection = CampaignStatusProjection {
            schema: "mining-campaign-status-v7",
            stage: self.stage.label(),
            lease_id: self.retained_maybe_lease.map(|lease| lease.id().raw()),
            campaign_state: campaign_state_label(snapshot.campaign_state),
            profile: self
                .maybe_profile
                .map_or("none", MiningHardwareProfilePreset::label),
            active_ms,
            submit_outcome: if snapshot.mining.counters.accepted > 0 {
                "accepted"
            } else if snapshot.mining.counters.rejected > 0 {
                "rejected"
            } else {
                "none"
            },
            qualified_candidate_count: snapshot.mining.counters.qualified_candidates,
            below_pool_target_count: snapshot.mining.counters.below_pool_target,
            duplicate_candidate_count: snapshot.mining.counters.duplicate_candidates,
            accepted_share_count: snapshot.mining.counters.accepted,
            rejected_share_count: snapshot.mining.counters.rejected,
            job_transition: CampaignJobTransitionProjection::from(snapshot.job_transition),
            terminal_reason: snapshot
                .maybe_blocker
                .map_or("none", |blocker| blocker.label()),
            safety: if safety_fresh { "fresh" } else { "stale" },
            fresh_observation_count: observation_freshness.fresh_count(),
            observation_freshness,
            observation_requirements: CampaignObservationRequirements::ULTRA_205,
            pool_config: match self.pool_config {
                PoolConfigurationStatus::NotRead => "not_read",
                PoolConfigurationStatus::LocalOwnerSupplied => "local_owner_supplied",
            },
            actuation: match self.actuation {
                CampaignActuationStatus::None => "none",
                CampaignActuationStatus::Qualified => "qualified",
                CampaignActuationStatus::SafeStopped => "safe_stopped",
            },
            mineonboot,
            safe_stop: match self.safe_stop {
                CampaignSafeStopStatus::NotRequired => "not_required",
                CampaignSafeStopStatus::Pending => "pending",
                CampaignSafeStopStatus::Confirmed => "confirmed",
            },
            failure: self.failure,
        };
        serde_json::to_string(&projection)
            .expect("closed campaign status projection must always serialize")
    }
}

#[derive(Serialize)]
struct CampaignStatusProjection {
    schema: &'static str,
    stage: &'static str,
    lease_id: Option<u64>,
    campaign_state: &'static str,
    profile: &'static str,
    active_ms: u64,
    submit_outcome: &'static str,
    qualified_candidate_count: u64,
    below_pool_target_count: u64,
    duplicate_candidate_count: u64,
    accepted_share_count: u64,
    rejected_share_count: u64,
    job_transition: CampaignJobTransitionProjection,
    terminal_reason: &'static str,
    safety: &'static str,
    fresh_observation_count: u8,
    observation_freshness: CampaignObservationFreshness,
    observation_requirements: CampaignObservationRequirements,
    pool_config: &'static str,
    actuation: &'static str,
    mineonboot: bool,
    safe_stop: &'static str,
    failure: CampaignFailureDiagnostic,
}

#[derive(Serialize)]
struct CampaignJobTransitionProjection {
    pool_notify_count: u64,
    clean_jobs_notify_count: u64,
    previous_block_change_count: u64,
    new_block_generation_count: u64,
    replacement_dispatch_count: u64,
    post_transition_correlated_result_count: u64,
    completed_transition_count: u64,
    stale_generation_result_discard_count: u64,
    stale_generation_submit_count: u64,
    reconnect_count: u64,
    latest_state: &'static str,
}

impl From<JobTransitionEvidence> for CampaignJobTransitionProjection {
    fn from(evidence: JobTransitionEvidence) -> Self {
        Self {
            pool_notify_count: evidence.pool_notify_count,
            clean_jobs_notify_count: evidence.clean_jobs_notify_count,
            previous_block_change_count: evidence.previous_block_change_count,
            new_block_generation_count: evidence.new_block_generation_count,
            replacement_dispatch_count: evidence.replacement_dispatch_count,
            post_transition_correlated_result_count: evidence.replacement_result_count,
            completed_transition_count: evidence.completed_transition_count,
            stale_generation_result_discard_count: evidence.stale_generation_result_discard_count,
            stale_generation_submit_count: evidence.stale_generation_submit_count,
            reconnect_count: evidence.reconnect_count,
            latest_state: evidence.latest_state.label(),
        }
    }
}

const fn campaign_state_label(state: MiningCampaignState) -> &'static str {
    match state {
        MiningCampaignState::Unavailable => "unavailable",
        MiningCampaignState::Preparing => "preparing",
        MiningCampaignState::Armed => "armed",
        MiningCampaignState::Active => "active",
        MiningCampaignState::SafeStopping => "safe_stopping",
        MiningCampaignState::Consumed => "consumed",
    }
}

#[cfg(test)]
mod tests {
    use bitaxe_stratum::v1::production_session::{
        MiningCampaignDuration, MiningCampaignLeaseId, MiningCampaignStopCondition,
        MiningHardwareProfilePreset, MiningHardwareState, ProductionSessionBlocker,
        ProductionSessionPhase, ProductionSessionSnapshot,
    };
    use bitaxe_stratum::v1::production_work::PoolSessionGeneration;
    use bitaxe_stratum::v1::state::{MiningRuntimeState, ShareCounters};
    use serde_json::Value;

    use super::*;

    fn snapshot(campaign_state: MiningCampaignState) -> ProductionSessionSnapshot {
        ProductionSessionSnapshot {
            phase: ProductionSessionPhase::WaitingForReadiness,
            maybe_blocker: None,
            maybe_active_pool: None,
            generation: PoolSessionGeneration::initial(),
            hardware_state: MiningHardwareState::Unprepared,
            campaign_state,
            job_transition: JobTransitionEvidence::default(),
            mining: MiningRuntimeState::default(),
        }
    }

    #[test]
    fn observation_marker_is_non_authorizing_and_terminal_safe() {
        // Arrange
        let tracker = CampaignStatusTracker::new(MiningCampaignStage::Observation, None, None);

        // Act
        let marker = tracker.marker(
            &snapshot(MiningCampaignState::Unavailable),
            360_000,
            true,
            CampaignObservationFreshness::all_ultra205_supported_fresh(),
            false,
        );
        let value: Value = serde_json::from_str(&marker).expect("marker should be JSON");

        // Assert
        assert_eq!(value["schema"], "mining-campaign-status-v7");
        assert_eq!(value["stage"], "observation");
        assert!(value["lease_id"].is_null());
        assert_eq!(value["campaign_state"], "unavailable");
        assert_eq!(value["profile"], "none");
        assert_eq!(value["pool_config"], "not_read");
        assert_eq!(value["actuation"], "none");
        assert_eq!(value["mineonboot"], false);
        assert_eq!(value["safety"], "fresh");
        assert_eq!(value["fresh_observation_count"], 5);
        assert_eq!(
            value["observation_freshness"],
            serde_json::json!({
                "power_watts": true,
                "bus_voltage_volts": true,
                "current_amps": true,
                "chip_temp_celsius": true,
                "vr_temp_celsius": false,
                "fan_rpm": true,
            })
        );
        assert_eq!(
            value["observation_requirements"],
            serde_json::json!({
                "power_watts": true,
                "bus_voltage_volts": true,
                "current_amps": true,
                "chip_temp_celsius": true,
                "vr_temp_celsius": false,
                "fan_rpm": true,
            })
        );
        assert_eq!(value["safe_stop"], "not_required");
        assert_eq!(
            value["failure"],
            serde_json::json!({
                "phase": "none",
                "step": "none",
                "detail": "none",
                "rollback_step": "none",
                "rollback_detail": "none",
            })
        );
    }

    #[test]
    fn earliest_typed_campaign_failure_survives_later_cleanup_failure() {
        // Arrange
        let profile = MiningHardwareProfilePreset::Conservative;
        let lease = MiningCampaignLease::new(
            MiningCampaignLeaseId::new(8).expect("lease id"),
            profile.profile(),
            MiningCampaignStopCondition::FirstSubmitResponse {
                timeout: MiningCampaignDuration::new(600_000).expect("duration"),
            },
        );
        let mut tracker =
            CampaignStatusTracker::new(MiningCampaignStage::LiveShare, Some(lease), Some(profile));

        // Act
        tracker.note_failure(
            "hardware_preparation",
            "reset_and_detect_exactly_one_chip",
            "asic_actuation_failed",
            "wait_for_fresh_temperature_at_or_below_45_c",
            "cooling_proof_timed_out",
        );
        tracker.note_failure(
            "hardware_safe_stop",
            "disable_core_voltage",
            "safety_hardware_write_failed",
            "none",
            "none",
        );
        let marker = tracker.marker(
            &snapshot(MiningCampaignState::Consumed),
            1_000,
            true,
            CampaignObservationFreshness::all_ultra205_supported_fresh(),
            false,
        );
        let value: Value = serde_json::from_str(&marker).expect("marker should be JSON");

        // Assert
        assert_eq!(
            value["failure"],
            serde_json::json!({
                "phase": "hardware_preparation",
                "step": "reset_and_detect_exactly_one_chip",
                "detail": "asic_actuation_failed",
                "rollback_step": "wait_for_fresh_temperature_at_or_below_45_c",
                "rollback_detail": "cooling_proof_timed_out",
            })
        );
    }

    #[test]
    fn consumed_live_share_retains_metadata_and_terminal_outcome() {
        // Arrange
        let profile = MiningHardwareProfilePreset::Conservative;
        let lease = MiningCampaignLease::new(
            MiningCampaignLeaseId::new(7).expect("lease id"),
            profile.profile(),
            MiningCampaignStopCondition::FirstSubmitResponse {
                timeout: MiningCampaignDuration::new(600_000).expect("duration"),
            },
        );
        let mut tracker =
            CampaignStatusTracker::new(MiningCampaignStage::LiveShare, Some(lease), Some(profile));
        let mut active = snapshot(MiningCampaignState::Active);
        tracker.note_pool_configuration_read(true);
        tracker.note_snapshot(&active, 100);
        active.mining.counters = ShareCounters {
            accepted: 1,
            rejected: 0,
            qualified_candidates: 1,
            below_pool_target: 7,
            duplicate_candidates: 2,
            ..ShareCounters::default()
        };
        active.campaign_state = MiningCampaignState::Consumed;
        active.maybe_blocker = Some(ProductionSessionBlocker::CampaignLeaseConsumed);

        // Act
        tracker.note_safe_stop_pending();
        tracker.note_snapshot(&active, 1_100);
        let marker = tracker.marker(
            &active,
            1_100,
            true,
            CampaignObservationFreshness::all_ultra205_supported_fresh(),
            false,
        );
        let value: Value = serde_json::from_str(&marker).expect("marker should be JSON");

        // Assert
        assert_eq!(value["lease_id"], 7);
        assert_eq!(value["profile"], "conservative");
        assert_eq!(value["campaign_state"], "consumed");
        assert_eq!(value["submit_outcome"], "accepted");
        assert_eq!(value["qualified_candidate_count"], 1);
        assert_eq!(value["below_pool_target_count"], 7);
        assert_eq!(value["duplicate_candidate_count"], 2);
        assert_eq!(value["terminal_reason"], "campaign_lease_consumed");
        assert_eq!(value["pool_config"], "local_owner_supplied");
        assert_eq!(value["actuation"], "safe_stopped");
        assert_eq!(value["safe_stop"], "confirmed");
        assert_eq!(value["active_ms"], 1_000);
        assert!(!tracker.authorizes_actuation());
        assert!(tracker.maybe_lease().is_none());
    }

    #[test]
    fn lease_scoped_run_override_returns_to_paused_after_consumption() {
        // Arrange
        let profile = MiningHardwareProfilePreset::Conservative;
        let lease = MiningCampaignLease::new(
            MiningCampaignLeaseId::new(9).expect("lease id"),
            profile.profile(),
            MiningCampaignStopCondition::ActiveDuration {
                duration: MiningCampaignDuration::new(1_000).expect("duration"),
            },
        );
        let mut tracker =
            CampaignStatusTracker::new(MiningCampaignStage::Soak, Some(lease), Some(profile));

        // Act
        let during_lease = tracker.operator_intent(MiningOperatorIntent::Paused);
        tracker.note_snapshot(&snapshot(MiningCampaignState::Consumed), 1_000);
        let after_consumption = tracker.operator_intent(MiningOperatorIntent::Run);

        // Assert
        assert_eq!(during_lease, MiningOperatorIntent::Run);
        assert_eq!(after_consumption, MiningOperatorIntent::Paused);
    }
}
