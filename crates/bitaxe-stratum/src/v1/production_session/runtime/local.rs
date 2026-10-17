//! Probe before power for local mining runs (ADR-0038). A `Local` run connects and
//! authorizes with the ASIC off; only an authorized pool powers it up. The probe
//! connection then closes, and the run reconnects once the hardware is ready, in
//! the same order a lease uses. Exhausted pools stop the hardware resumably, so the
//! recovery policy keeps probing every 30 s with the ASIC off, as upstream does.
//! Nothing here applies to Gate leases or campaigns.

use super::*;
use crate::v1::production_session::campaign::MiningCampaignStopCondition;
use crate::v1::recovery_policy::RecoveryAction;

impl ProductionMiningSession {
    /// Whether `lease` is a local mining run.
    pub(super) fn is_local_lease(lease: MiningCampaignLease) -> bool {
        lease.stop_condition() == MiningCampaignStopCondition::Local
    }

    /// Whether the current local run is probing its pool with the ASIC off.
    pub(super) fn probing_local(&self) -> bool {
        self.maybe_lease.is_some_and(Self::is_local_lease)
            && matches!(
                self.hardware_state,
                MiningHardwareState::Unprepared | MiningHardwareState::Stopped
            )
    }

    /// Adopts an offered local run while the ASIC is off and drives the pool probe.
    pub(super) fn probe_local(
        &mut self,
        lease: MiningCampaignLease,
        wakeup: Option<ProductionSessionWakeup>,
        readiness: ProductionReadiness,
        now_ms: u64,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<(), StratumV1Error> {
        if self.maybe_lease.is_none() {
            self.maybe_lease = Some(lease);
            self.maybe_activation_started_at_ms = Some(now_ms);
            self.campaign_state = MiningCampaignState::Preparing;
        }
        let actions = self.recovery.on_wakeup(wakeup, readiness, now_ms);
        self.apply_recovery_actions(actions, effects)
    }

    /// The probe's pool authorized: close it and power up. Returns whether it did.
    pub(in crate::v1::production_session) fn finish_local_probe(
        &mut self,
        effects: &mut Vec<ProductionSessionEffect>,
    ) -> Result<bool, StratumV1Error> {
        if !self.probing_local() {
            return Ok(false);
        }
        let Some(lease) = self.maybe_lease else {
            return Ok(false);
        };
        self.apply_recovery_actions(vec![RecoveryAction::ClosePoolConnection], effects)?;
        self.bridge.invalidate_session();
        self.recovery.reset_after_probe();
        self.hardware_state = MiningHardwareState::Preparing;
        self.campaign_state = MiningCampaignState::Preparing;
        self.maybe_prepared_at_ms = None;
        self.maybe_active_since_ms = None;
        self.job_transition = JobTransitionTracker::default();
        self.asic_diagnostics = AsicBridgeDiagnosticsTracker::default();
        self.maybe_retained_mining = None;
        self.dispatched_since_power_up = false;
        effects.push(ProductionSessionEffect::PrepareHardware {
            lease_id: lease.id(),
            profile: lease.profile(),
        });
        Ok(true)
    }

    /// Bookkeeping for work handed to the ASIC, for leases and local runs alike.
    pub(in crate::v1::production_session) fn note_asic_dispatch(
        &mut self,
        generation: crate::v1::production_work::PoolSessionGeneration,
        now_ms: u64,
    ) {
        self.job_transition.note_dispatch(generation);
        self.asic_diagnostics.note_dispatch(generation, now_ms);
        self.dispatched_since_power_up = true;
    }

    /// A blocker while a local run has no powered hardware ends the run at once:
    /// there is nothing to stop, and the owner decides when the next begins.
    pub(super) fn end_unpowered_local_run(&mut self) {
        if !self.probing_local() {
            return;
        }
        if let Some(lease) = self.maybe_lease {
            self.finish_terminal_safe_stop(lease.id());
        }
    }

    /// Exhausted pools pause a powered local run's hardware without ending the run,
    /// once work reached the ASIC since this power-up: an outage then probes every
    /// 30 s with the ASIC off. A power-up that never dispatched ends the run instead,
    /// so the owner's restart guard paces a pool that authorizes and then drops.
    pub(super) fn note_local_pools_exhausted(&mut self) {
        if self.maybe_lease.is_some_and(Self::is_local_lease)
            && self.dispatched_since_power_up
            && matches!(
                self.hardware_state,
                MiningHardwareState::Preparing | MiningHardwareState::Ready
            )
        {
            self.resumable_pause_pending = true;
        }
    }
}
