//! Redacted retained status projection for repo-owned mining campaigns.

use bitaxe_stratum::v1::production_session::{
    AsicBridgeEvidence, JobTransitionEvidence, MiningCampaignLease, MiningCampaignState,
    MiningHardwareProfilePreset, MiningHardwareState, ProductionSessionSnapshot,
};
use bitaxe_stratum::v1::state::MiningOperatorIntent;
use serde::Serialize;

use crate::settings_adapter::MiningCampaignStage;

#[cfg(test)]
use super::readiness_trace::ObservationEpochRelation;
use super::readiness_trace::ReadinessTransitionEvidence;

#[path = "campaign_status/projection.rs"]
mod projection;
use projection::*;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResumablePauseSafeStopStatus {
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
    logged_generation_invalidation_count: u64,
    logged_stale_completion_count: u64,
    poll_rearm_logged: bool,
    nonce_correlation_logged: bool,
    active_seen: bool,
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
            logged_generation_invalidation_count: 0,
            logged_stale_completion_count: 0,
            poll_rearm_logged: false,
            nonce_correlation_logged: false,
            active_seen: false,
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

    pub(super) const fn requires_requested_run_bootstrap(&self) -> bool {
        self.lease_authorizing && matches!(self.stage, MiningCampaignStage::CommandEffects)
    }

    pub(super) fn operator_intent(
        &self,
        persisted_intent: MiningOperatorIntent,
    ) -> MiningOperatorIntent {
        if !self.lease_authorizing {
            return MiningOperatorIntent::Paused;
        }
        if self.stage == MiningCampaignStage::CommandEffects && self.active_seen {
            return persisted_intent;
        }
        MiningOperatorIntent::Run
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
        self.maybe_log_asic_evidence(snapshot.asic_bridge);
        if snapshot.campaign_state == MiningCampaignState::Active {
            self.active_seen = true;
            self.maybe_active_since_ms.get_or_insert(now_ms);
        } else if let Some(active_since) = self.maybe_active_since_ms.take() {
            self.retained_active_ms = self
                .retained_active_ms
                .saturating_add(now_ms.saturating_sub(active_since));
        }

        if snapshot.campaign_state == MiningCampaignState::Consumed {
            self.lease_authorizing = false;
            self.actuation = CampaignActuationStatus::SafeStopped;
            self.safe_stop = CampaignSafeStopStatus::Confirmed;
        }
    }

    fn maybe_log_asic_evidence(&mut self, evidence: AsicBridgeEvidence) {
        if evidence.generation_invalidation_count > self.logged_generation_invalidation_count {
            self.logged_generation_invalidation_count = evidence.generation_invalidation_count;
            log::info!("asic_bridge=generation_invalidated");
        }
        if !self.poll_rearm_logged && evidence.post_transition_poll_request_count > 0 {
            self.poll_rearm_logged = true;
            log::info!("asic_bridge=poll_rearmed");
        }
        if evidence.stale_completion_count > self.logged_stale_completion_count {
            self.logged_stale_completion_count = evidence.stale_completion_count;
            log::warn!("asic_bridge=stale_completion");
        }
        if !self.nonce_correlation_logged && evidence.post_transition_correlation_count > 0 {
            self.nonce_correlation_logged = true;
            log::info!("asic_bridge=nonce_correlated");
        }
    }

    pub(super) fn marker(
        &self,
        snapshot: &ProductionSessionSnapshot,
        now_ms: u64,
        safety_fresh: bool,
        observation_freshness: CampaignObservationFreshness,
        mineonboot: bool,
        pool_config_persisted: bool,
        protocol_gate: &'static str,
        readiness_transition: ReadinessTransitionEvidence,
    ) -> String {
        let active_ms = self
            .maybe_active_since_ms
            .map_or(self.retained_active_ms, |started| {
                self.retained_active_ms
                    .saturating_add(now_ms.saturating_sub(started))
            });
        let projection = CampaignStatusProjection {
            schema: "mining-campaign-status-v12",
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
            asic_bridge: snapshot.asic_bridge,
            terminal_reason: snapshot
                .maybe_blocker
                .map_or("none", |blocker| blocker.label()),
            protocol_gate,
            readiness_transition: CampaignReadinessTransitionProjection::from(readiness_transition),
            resumable_pause_safe_stop: match self.resumable_pause_safe_stop(snapshot) {
                ResumablePauseSafeStopStatus::NotRequired => "not_required",
                ResumablePauseSafeStopStatus::Pending => "pending",
                ResumablePauseSafeStopStatus::Confirmed => "confirmed",
            },
            safety: if safety_fresh { "fresh" } else { "stale" },
            fresh_observation_count: observation_freshness.fresh_count(),
            observation_freshness,
            observation_requirements: CampaignObservationRequirements::ULTRA_205,
            pool_config: match self.pool_config {
                PoolConfigurationStatus::NotRead => "not_read",
                PoolConfigurationStatus::LocalOwnerSupplied => "local_owner_supplied",
            },
            pool_config_persisted,
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

    fn resumable_pause_safe_stop(
        &self,
        snapshot: &ProductionSessionSnapshot,
    ) -> ResumablePauseSafeStopStatus {
        if self.stage != MiningCampaignStage::CommandEffects
            || !self.active_seen
            || !self.lease_authorizing
            || snapshot.mining.operator_intent != MiningOperatorIntent::Paused
        {
            return ResumablePauseSafeStopStatus::NotRequired;
        }
        if snapshot.campaign_state == MiningCampaignState::Armed
            && snapshot.hardware_state == MiningHardwareState::Stopped
        {
            return ResumablePauseSafeStopStatus::Confirmed;
        }
        ResumablePauseSafeStopStatus::Pending
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
        ProductionSessionPhase, ProductionSessionSnapshot, ProductionSessionWakeup,
    };
    use bitaxe_stratum::v1::production_work::PoolSessionGeneration;
    use bitaxe_stratum::v1::state::{MiningRuntimeState, ShareCounters};
    use serde_json::Value;

    use super::*;

    include!("campaign_status/tests/operator_intent.rs");
    include!("campaign_status/tests/resumable_pause.rs");

    fn snapshot(campaign_state: MiningCampaignState) -> ProductionSessionSnapshot {
        ProductionSessionSnapshot {
            phase: ProductionSessionPhase::WaitingForReadiness,
            maybe_blocker: None,
            maybe_active_pool: None,
            generation: PoolSessionGeneration::initial(),
            hardware_state: MiningHardwareState::Unprepared,
            campaign_state,
            job_transition: JobTransitionEvidence::default(),
            asic_bridge: AsicBridgeEvidence::default(),
            mining: MiningRuntimeState::default(),
        }
    }

    fn readiness_transition() -> ReadinessTransitionEvidence {
        ReadinessTransitionEvidence {
            wakeup: Some(ProductionSessionWakeup::ObservationsChanged),
            previous_blocker: Some(ProductionSessionBlocker::SafetyPrerequisitesStale),
            current_blocker: None,
            session_phase: ProductionSessionPhase::WaitingForReadiness,
            campaign_state: MiningCampaignState::Armed,
            hardware_state: MiningHardwareState::Stopped,
            safety_sample_fresh: true,
            observation_epoch_relation: ObservationEpochRelation::Advanced,
            pending_observation_recovered: true,
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
            false,
            "ready",
            readiness_transition(),
        );
        let value: Value = serde_json::from_str(&marker).expect("marker should be JSON");

        // Assert
        assert_eq!(value["schema"], "mining-campaign-status-v12");
        assert_eq!(value["resumable_pause_safe_stop"], "not_required");
        assert_eq!(value["protocol_gate"], "ready");
        assert_eq!(
            value["readiness_transition"],
            serde_json::json!({
                "wakeup": "observations_changed",
                "previous_blocker": "safety_prerequisites_stale",
                "current_blocker": "none",
                "session_phase": "waiting_for_readiness",
                "campaign_state": "armed",
                "hardware_state": "stopped",
                "safety_sample": "fresh",
                "observation_epoch": "advanced",
                "pending_observation_recovered": true,
            })
        );
        assert_eq!(value["stage"], "observation");
        assert!(value["lease_id"].is_null());
        assert_eq!(value["campaign_state"], "unavailable");
        assert_eq!(value["profile"], "none");
        assert_eq!(value["pool_config"], "not_read");
        assert_eq!(value["pool_config_persisted"], false);
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
            true,
            "primary_selector_invalid",
            readiness_transition(),
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
            true,
            "ready",
            readiness_transition(),
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
        assert_eq!(value["pool_config_persisted"], true);
        assert_eq!(value["actuation"], "safe_stopped");
        assert_eq!(value["safe_stop"], "confirmed");
        assert_eq!(value["active_ms"], 1_000);
        assert!(!tracker.authorizes_actuation());
        assert!(tracker.maybe_lease().is_none());
    }
}
