//! Status publication from the existing production owner; no independent actuation owner.
use super::campaign_status::{
    publication::CampaignStatusPublicationError, CampaignObservationFreshness,
};
use super::*;
use bitaxe_safety::observation::Observation;
use bitaxe_safety::power::POWER_SAMPLE_STALE_AFTER_MS;

impl OrdinaryEspProductionSessionAdapter {
    pub(super) fn publish_campaign_status(
        &mut self,
        snapshot: &bitaxe_stratum::v1::production_session::ProductionSessionSnapshot,
        now_ms: u64,
    ) -> Result<(), CampaignStatusPublicationError> {
        if self.maybe_campaign_status.is_none() {
            return Ok(());
        }
        if snapshot.campaign_state
            == bitaxe_stratum::v1::production_session::MiningCampaignState::Consumed
            && self.maybe_terminal_pool_persisted.is_none()
        {
            self.maybe_terminal_pool_persisted = Some(matches!(
                crate::settings_adapter::read_production_pool_set(),
                Ok(Some(_))
            ));
        }
        let pool_config_persisted = self.maybe_terminal_pool_persisted.unwrap_or(false);
        let Some(status) = self.maybe_campaign_status.as_ref() else {
            return Ok(());
        };
        let Some(readiness_transition) = self.readiness_trace.evidence() else {
            log::error!("mining_campaign_status=withheld category=readiness_transition_missing");
            return Ok(());
        };
        let terminal = snapshot.campaign_state
            == bitaxe_stratum::v1::production_session::MiningCampaignState::Consumed;
        if !self
            .campaign_status_publication
            .should_publish(now_ms, terminal)?
        {
            return Ok(());
        }
        let observations = crate::safety_adapter::observation_snapshot();
        let safety_now = now();
        let safety_fresh = observations.is_ultra_205_mining_safe_at(safety_now);
        let observation_freshness = CampaignObservationFreshness {
            power_watts: is_current(&observations.power_watts, safety_now),
            bus_voltage_volts: is_current(&observations.bus_voltage_volts, safety_now),
            current_amps: is_current(&observations.current_amps, safety_now),
            chip_temp_celsius: is_current(&observations.chip_temp_celsius, safety_now),
            vr_temp_celsius: is_current(&observations.vr_temp_celsius, safety_now),
            fan_rpm: is_current(&observations.fan_rpm, safety_now),
        };
        let marker = status.marker(
            snapshot,
            now_ms,
            safety_fresh,
            observation_freshness,
            crate::settings_adapter::start_mining_on_boot(),
            pool_config_persisted,
            self.protocol_gate.label(),
            readiness_transition,
        );
        crate::info_retained(&format!("mining_campaign_status={marker}"));
        Ok(())
    }

    pub(super) fn service_hashrate_monitor(
        &mut self,
        snapshot: &ProductionSessionSnapshot,
        now_ms: u64,
    ) {
        let Ok(maybe_tick) = self.hashrate.service_snapshot(snapshot, now_ms) else {
            log::warn!("hashrate_monitor=unavailable category=schedule_overflow");
            return;
        };
        let Some(tick) = maybe_tick else { return };
        crate::runtime_snapshot::publish_hashrate_snapshot(tick.snapshot);
        if tick.request_registers
            && self
                .asic
                .try_send(
                    AsicWorkerCommand::ReadHashrateRegisters {
                        generation: snapshot.generation,
                    },
                    self.maybe_bwg_session
                        .as_ref()
                        .map(|session| session.generation),
                )
                .is_err()
        {
            log::warn!("hashrate_monitor_read=skipped category=worker_unavailable");
        }
    }
}

fn is_current<T>(observation: &Observation<T>, now: MonotonicMillis) -> bool {
    observation.is_fresh()
        && observation.maybe_last_good().is_some_and(|sample| {
            now.get().saturating_sub(sample.acquired_at().get())
                <= u64::from(POWER_SAMPLE_STALE_AFTER_MS)
        })
}
