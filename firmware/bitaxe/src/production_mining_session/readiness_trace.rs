use bitaxe_api::TelemetryObservations;
use bitaxe_safety::observation::{Observation, StampedSample};
use bitaxe_stratum::v1::production_session::{
    MiningCampaignState, MiningHardwareState, ProductionReadiness, ProductionSessionBlocker,
    ProductionSessionPhase, ProductionSessionSnapshot, ProductionSessionWakeup,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ObservationStamp {
    boot_session: u64,
    sequence: u64,
    acquired_at_ms: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ObservationEpoch([ObservationStamp; 5]);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ObservationEpochRelation {
    Initial,
    Advanced,
    Unchanged,
    Unavailable,
}

impl ObservationEpochRelation {
    pub(super) const fn label(self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::Advanced => "advanced",
            Self::Unchanged => "unchanged",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) struct ReadinessTransitionEvidence {
    pub(super) wakeup: Option<ProductionSessionWakeup>,
    pub(super) previous_blocker: Option<ProductionSessionBlocker>,
    pub(super) current_blocker: Option<ProductionSessionBlocker>,
    pub(super) session_phase: ProductionSessionPhase,
    pub(super) campaign_state: MiningCampaignState,
    pub(super) hardware_state: MiningHardwareState,
    pub(super) safety_sample_fresh: bool,
    pub(super) observation_epoch_relation: ObservationEpochRelation,
    pub(super) pending_observation_recovered: bool,
}

impl ReadinessTransitionEvidence {
    pub(super) const fn wakeup_label(self) -> &'static str {
        match self.wakeup {
            None => "deadline",
            Some(ProductionSessionWakeup::NetworkChanged) => "network_changed",
            Some(ProductionSessionWakeup::SettingsChanged) => "settings_changed",
            Some(ProductionSessionWakeup::ObservationsChanged) => "observations_changed",
            Some(ProductionSessionWakeup::OperatorIntentChanged) => "operator_intent_changed",
            Some(ProductionSessionWakeup::ShutdownRequested) => "shutdown_requested",
        }
    }

    pub(super) const fn previous_blocker_label(self) -> &'static str {
        blocker_label(self.previous_blocker)
    }

    pub(super) const fn current_blocker_label(self) -> &'static str {
        blocker_label(self.current_blocker)
    }

    pub(super) const fn session_phase_label(self) -> &'static str {
        match self.session_phase {
            ProductionSessionPhase::WaitingForReadiness => "waiting_for_readiness",
            ProductionSessionPhase::ConnectingPrimary => "connecting_primary",
            ProductionSessionPhase::RunningPrimary => "running_primary",
            ProductionSessionPhase::ConnectingFallback => "connecting_fallback",
            ProductionSessionPhase::RunningFallback => "running_fallback",
            ProductionSessionPhase::RecoveryPaused => "recovery_paused",
            ProductionSessionPhase::SafeStopping => "safe_stopping",
            ProductionSessionPhase::Shutdown => "shutdown",
        }
    }

    pub(super) const fn campaign_state_label(self) -> &'static str {
        match self.campaign_state {
            MiningCampaignState::Unavailable => "unavailable",
            MiningCampaignState::Preparing => "preparing",
            MiningCampaignState::Armed => "armed",
            MiningCampaignState::Active => "active",
            MiningCampaignState::SafeStopping => "safe_stopping",
            MiningCampaignState::Consumed => "consumed",
        }
    }

    pub(super) const fn hardware_state_label(self) -> &'static str {
        match self.hardware_state {
            MiningHardwareState::Unprepared => "unprepared",
            MiningHardwareState::Preparing => "preparing",
            MiningHardwareState::Ready => "ready",
            MiningHardwareState::SafeStopping => "safe_stopping",
            MiningHardwareState::Stopped => "stopped",
        }
    }
}

#[derive(Default)]
pub(super) struct ReadinessTransitionTracker {
    maybe_last_epoch: Option<ObservationEpoch>,
    maybe_latest: Option<ReadinessTransitionEvidence>,
    maybe_safety_recovery: Option<ReadinessTransitionEvidence>,
}

impl ReadinessTransitionTracker {
    pub(super) fn observe(
        &mut self,
        wakeup: Option<ProductionSessionWakeup>,
        readiness: ProductionReadiness,
        observations: &TelemetryObservations,
        snapshot: &ProductionSessionSnapshot,
        pending_observation_recovered: bool,
    ) {
        let maybe_epoch = observation_epoch(observations);
        let relation = match (self.maybe_last_epoch, maybe_epoch) {
            (_, None) => ObservationEpochRelation::Unavailable,
            (None, Some(_)) => ObservationEpochRelation::Initial,
            (Some(previous), Some(current)) if previous == current => {
                ObservationEpochRelation::Unchanged
            }
            (Some(_), Some(_)) => ObservationEpochRelation::Advanced,
        };
        let previous_blocker = self.maybe_latest.and_then(|latest| latest.current_blocker);
        let current_blocker = readiness.maybe_blocker();
        if let Some(epoch) = maybe_epoch {
            self.maybe_last_epoch = Some(epoch);
        }
        let evidence = ReadinessTransitionEvidence {
            wakeup,
            previous_blocker,
            current_blocker,
            session_phase: snapshot.phase,
            campaign_state: snapshot.campaign_state,
            hardware_state: snapshot.hardware_state,
            safety_sample_fresh: readiness.safety_prerequisites_fresh,
            observation_epoch_relation: relation,
            pending_observation_recovered,
        };
        if self.maybe_safety_recovery.is_none()
            && evidence.wakeup == Some(ProductionSessionWakeup::ObservationsChanged)
            && evidence.previous_blocker == Some(ProductionSessionBlocker::SafetyPrerequisitesStale)
            && evidence.current_blocker.is_none()
            && evidence.safety_sample_fresh
            && evidence.observation_epoch_relation == ObservationEpochRelation::Advanced
        {
            self.maybe_safety_recovery = Some(evidence);
        }
        self.maybe_latest = Some(evidence);
    }

    pub(super) const fn evidence(&self) -> Option<ReadinessTransitionEvidence> {
        match self.maybe_safety_recovery {
            Some(evidence) => Some(evidence),
            None => self.maybe_latest,
        }
    }
}

const fn blocker_label(maybe_blocker: Option<ProductionSessionBlocker>) -> &'static str {
    match maybe_blocker {
        Some(blocker) => blocker.label(),
        None => "none",
    }
}

fn observation_epoch(observations: &TelemetryObservations) -> Option<ObservationEpoch> {
    Some(ObservationEpoch([
        stamp(&observations.power_watts)?,
        stamp(&observations.bus_voltage_volts)?,
        stamp(&observations.current_amps)?,
        stamp(&observations.chip_temp_celsius)?,
        stamp(&observations.fan_rpm)?,
    ]))
}

fn stamp<T>(observation: &Observation<T>) -> Option<ObservationStamp> {
    observation.maybe_last_good().map(stamp_from_sample)
}

fn stamp_from_sample<T>(sample: &StampedSample<T>) -> ObservationStamp {
    ObservationStamp {
        boot_session: sample.boot_session().get(),
        sequence: sample.sequence().get(),
        acquired_at_ms: sample.acquired_at().get(),
    }
}

#[cfg(test)]
mod tests {
    use bitaxe_safety::observation::{
        BootSessionId, MonotonicMillis, ObservationSequence, UnavailableReason,
    };
    use bitaxe_stratum::v1::production_session::{
        MiningCampaignDuration, MiningCampaignLease, MiningCampaignLeaseId,
        MiningCampaignStopCondition, MiningHardwareProfile,
    };
    use bitaxe_stratum::v1::state::MiningOperatorIntent;

    use super::*;

    fn fresh<T>(value: T, sequence: u64) -> Observation<T> {
        Observation::record_success(
            value,
            BootSessionId::new(7),
            ObservationSequence::new(sequence.saturating_sub(1)),
            MonotonicMillis::new(sequence * 100),
        )
        .expect("fixture sequence must advance")
        .0
    }

    fn observations(sequence: u64) -> TelemetryObservations {
        TelemetryObservations {
            power_watts: fresh(5.0, sequence),
            bus_voltage_volts: fresh(5.0, sequence),
            current_amps: fresh(1.0, sequence),
            core_voltage_actual_mv: fresh(1_000.0, sequence),
            chip_temp_celsius: fresh(40.0, sequence),
            vr_temp_celsius: Observation::unavailable(UnavailableReason::ThermalReadingUnavailable),
            fan_rpm: fresh(2_000, sequence),
        }
    }

    fn readiness(safety_prerequisites_fresh: bool) -> ProductionReadiness {
        let profile = MiningHardwareProfile::ultra_205_bm1366(400, 1_100, 100)
            .expect("fixture profile must be valid");
        let lease = MiningCampaignLease::new(
            MiningCampaignLeaseId::new(7).expect("fixture lease id must be valid"),
            profile,
            MiningCampaignStopCondition::ActiveDuration {
                duration: MiningCampaignDuration::new(600_000)
                    .expect("fixture duration must be valid"),
            },
        );
        ProductionReadiness {
            operator_intent: MiningOperatorIntent::Run,
            network_ready: true,
            stratum_v1_supported: true,
            safety_prerequisites_fresh,
            maybe_campaign_lease: Some(lease),
            actuation_qualified: true,
        }
    }

    #[test]
    fn retains_the_first_advanced_safety_recovery_transition() {
        // Arrange
        let mut tracker = ReadinessTransitionTracker::default();
        tracker.observe(
            Some(ProductionSessionWakeup::OperatorIntentChanged),
            readiness(false),
            &observations(1),
            &snapshot(),
            false,
        );

        // Act
        tracker.observe(
            Some(ProductionSessionWakeup::ObservationsChanged),
            readiness(true),
            &observations(2),
            &snapshot(),
            true,
        );
        tracker.observe(None, readiness(true), &observations(2), &snapshot(), false);
        let evidence = tracker.evidence().expect("recovery evidence must exist");

        // Assert
        assert_eq!(
            evidence.previous_blocker,
            Some(ProductionSessionBlocker::SafetyPrerequisitesStale)
        );
        assert_eq!(evidence.current_blocker, None);
        assert_eq!(
            evidence.observation_epoch_relation,
            ObservationEpochRelation::Advanced
        );
        assert!(evidence.safety_sample_fresh);
        assert!(evidence.pending_observation_recovered);
    }

    fn snapshot() -> ProductionSessionSnapshot {
        use bitaxe_stratum::v1::production_session::{AsicBridgeEvidence, JobTransitionEvidence};
        use bitaxe_stratum::v1::production_work::PoolSessionGeneration;
        use bitaxe_stratum::v1::state::MiningRuntimeState;

        ProductionSessionSnapshot {
            phase: ProductionSessionPhase::WaitingForReadiness,
            maybe_blocker: None,
            maybe_active_pool: None,
            generation: PoolSessionGeneration::initial(),
            hardware_state: MiningHardwareState::Stopped,
            campaign_state: MiningCampaignState::Armed,
            job_transition: JobTransitionEvidence::default(),
            asic_bridge: AsicBridgeEvidence::default(),
            mining: MiningRuntimeState::default(),
        }
    }
}
