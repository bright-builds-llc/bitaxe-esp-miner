//! Authoritative readiness observations for the sole production owner.
use super::*;

impl OrdinaryEspProductionSessionAdapter {
    pub(super) fn read_authoritative_readiness(
        &mut self,
        wakeup: Option<ProductionSessionWakeup>,
        snapshot: &ProductionSessionSnapshot,
        pending_observation_recovered: bool,
    ) -> ProductionReadiness {
        let requested_operator_intent = crate::runtime_snapshot::requested_mining_operator_intent();
        let wifi = crate::wifi_adapter::current_wifi_snapshot();
        let observations = crate::safety_adapter::observation_snapshot();
        // Only the fan-preparation stage may begin from fresh zero RPM. The
        // ordered adapter requires a post-command nonzero proof before power.
        let safety_prerequisites_fresh = cooling_core::preparation_safety(
            observations.is_ultra_205_mining_safe_at(now()),
            observations
                .fan_rpm
                .maybe_last_good()
                .is_some_and(|sample| *sample.value() > 0),
            self.maybe_bwg_session
                .as_ref()
                .is_some_and(|session| !session.preparation_started),
        );
        let maybe_campaign_lease = self
            .maybe_bwg_session
            .as_ref()
            .map(|session| session.lease)
            .or_else(|| {
                self.maybe_campaign_status
                    .as_ref()
                    .and_then(CampaignStatusTracker::maybe_lease)
            });
        let operator_intent = if self.maybe_bwg_session.is_some() {
            bitaxe_stratum::v1::state::MiningOperatorIntent::Run
        } else {
            self.maybe_campaign_status
                .as_ref()
                .map_or(requested_operator_intent, |status| {
                    status.operator_intent(requested_operator_intent)
                })
        };
        let actuation_qualified = (self.maybe_bwg_session.is_some()
            || self
                .maybe_campaign_status
                .as_ref()
                .is_some_and(CampaignStatusTracker::authorizes_actuation))
            && crate::safety_adapter::safety_actuation_available()
            && crate::asic_adapter::production::production_handle_available();
        self.protocol_gate = crate::settings_adapter::configured_protocol_gate();
        let readiness = ProductionReadiness {
            operator_intent,
            network_ready: wifi.wifi_status == "connected",
            stratum_v1_supported: self.maybe_bwg_session.is_some() || self.protocol_gate.is_ready(),
            safety_prerequisites_fresh,
            maybe_campaign_lease,
            actuation_qualified,
        };
        if self.maybe_bwg_session.is_some() {
            admission_diagnostics::readiness(readiness);
        }
        self.readiness_trace.observe(
            wakeup,
            readiness,
            &observations,
            snapshot,
            pending_observation_recovered,
        );
        readiness
    }
}
