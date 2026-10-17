//! The local mining owner (ADR-0038): the device mines on its own, as upstream does,
//! under a local revocation generation. Off until the crash-loop guard lands
//! (`task-boot-resume-and-crash-loop-guard`), so `LOCAL_MINING_ENABLED` is false.
//!
//! Each owner tick, with no run, the pure `LocalOwnerCore` decides whether one may
//! begin. A run is a `Local` campaign lease: the session probes the pool with the
//! ASIC off, prepares, and mines; every device-local stop revokes the generation
//! and the session's ordered safe-stop follows. When the session has consumed the
//! run's lease with the hardware stopped, the run ends here: the generation is
//! retired and the ending feeds the restart guard. A run the session never
//! adopted (readiness changed between the decision and the session's wake) is
//! retired at once, before any hardware effect, and counts as failed.
use super::*;
use bitaxe_runtime::local_mining::hardware_fault::{
    faulted_local_generation, hardware_fault_message,
};
use bitaxe_runtime::local_mining::owner::{LocalOwnerCore, UnsafeStop};
use bitaxe_runtime::local_mining::policy::{LocalDecision, LocalHoldReason, LocalInputs};
use bitaxe_stratum::v1::production_session::{
    DeviceProfileRefusal, MiningCampaignLease, MiningCampaignLeaseId, MiningCampaignState,
    MiningCampaignStopCondition, MiningHardwareProfile, MiningHardwareState,
    ProductionSessionBlocker,
};
use bitaxe_stratum::v1::state::MiningOperatorIntent;
use std::sync::atomic::{AtomicBool, Ordering};

/// Local mining stays off until the crash-loop guard lands (owner decision 2026-10-17).
pub(crate) const LOCAL_MINING_ENABLED: bool = false;

/// Set at boot when a stimulus owns the hardware, so local mining stays off.
static BOOT_OWNED_ELSEWHERE: AtomicBool = AtomicBool::new(false);

/// Upstream's `hardware_fault`: latched by a failed fan write under a local run and
/// held until reboot. Like upstream's message, it names the latest failed context.
static HARDWARE_FAULT_CONTEXT: Mutex<Option<&'static str>> = Mutex::new(None);

/// A real fan write failed. Under a local run this latches the hardware fault and
/// revokes the run, as upstream stops mining on `hardware_fault`.
pub(crate) fn note_fan_control_failure(
    owner: revocation::FanControllerOwner,
    context: &'static str,
    now_ms: u64,
) {
    let Some(generation) = faulted_local_generation(owner) else {
        return;
    };
    *HARDWARE_FAULT_CONTEXT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner) = Some(context);
    log::error!("local_mining=hardware_fault context={context}");
    // `false` means the run was already revoked; the latch still holds it off.
    let _already_revoked = !revocation::revoke_reason_at(
        generation,
        now_ms,
        revocation::RevocationReason::ControlFailed,
    );
}

fn maybe_hardware_fault_context() -> Option<&'static str> {
    *HARDWARE_FAULT_CONTEXT
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner)
}

/// Upstream's `hardware_fault` message, while the latch holds.
pub(crate) fn maybe_hardware_fault_message() -> Option<String> {
    maybe_hardware_fault_context().map(hardware_fault_message)
}

/// A boot-time owner other than the production owner holds the hardware.
pub(crate) fn note_other_boot_owner() {
    BOOT_OWNED_ELSEWHERE.store(true, Ordering::Release);
}

pub(super) struct LocalRun {
    pub(super) generation: revocation::WorkerGeneration,
    pub(super) lease: MiningCampaignLease,
    pub(super) preparation_started: bool,
}

#[derive(Default)]
pub(super) struct LocalOwner {
    pub(super) maybe_run: Option<LocalRun>,
    core: LocalOwnerCore,
    maybe_logged_hold: Option<LocalHoldReason>,
    last_requested_intent: Option<MiningOperatorIntent>,
}

impl LocalOwner {
    pub(super) fn maybe_generation(&self) -> Option<revocation::WorkerGeneration> {
        self.maybe_run.as_ref().map(|run| run.generation)
    }

    pub(super) fn maybe_lease(&self) -> Option<MiningCampaignLease> {
        self.maybe_run.as_ref().map(|run| run.lease)
    }

    /// The session ends a local run on a settings change; the ending is requested.
    pub(super) fn note_wakeup(&mut self, wakeup: ProductionSessionWakeup) {
        if wakeup == ProductionSessionWakeup::SettingsChanged {
            self.core.note_settings_changed();
        }
    }
}

impl OrdinaryEspProductionSessionAdapter {
    /// One owner tick of local mining; returns an event to drive when a run begins.
    #[inline(never)]
    pub(super) fn service_local_mining(
        &mut self,
        session: &ProductionMiningSession,
        now_ms: u64,
    ) -> Option<ProductionSessionEvent> {
        if !LOCAL_MINING_ENABLED {
            return None;
        }
        let snapshot = &session.snapshot();
        self.note_operator_resume();
        if let Some(run) = self.local.maybe_run.as_ref() {
            let id = run.lease.id();
            let ended = session.maybe_consumed_lease_id() == Some(id)
                && matches!(
                    snapshot.hardware_state,
                    MiningHardwareState::Unprepared | MiningHardwareState::Stopped
                );
            let adopted = ended || session.maybe_lease_id() == Some(id);
            self.local.core.note_run_progress(
                snapshot.campaign_state == MiningCampaignState::Active,
                now_ms,
            );
            if !adopted {
                // The session's readiness blocker says why; a lost readiness fact is
                // then requested, as it would be after adoption.
                let cause = snapshot
                    .maybe_blocker
                    .unwrap_or(ProductionSessionBlocker::CampaignLeaseUnavailable);
                self.end_local_run(Some(cause), now_ms);
            } else if ended {
                self.end_local_run(session.stop_cause_for(id), now_ms);
            }
            return None;
        }
        let inputs = self.local_inputs(now_ms);
        match self.local.core.decide_idle(inputs, now_ms) {
            LocalDecision::Hold(reason) => {
                if self.local.maybe_logged_hold != Some(reason) {
                    self.local.maybe_logged_hold = Some(reason);
                    log::info!("local_mining=hold reason={}", reason.label());
                }
                None
            }
            LocalDecision::Begin(profile) => {
                self.begin_local_run(profile, session.next_campaign_lease_id()?, snapshot, now_ms)
            }
        }
    }

    /// A Paused-to-Run edge is the operator's resume; it clears the restart guard.
    fn note_operator_resume(&mut self) {
        let intent = crate::runtime_snapshot::requested_mining_operator_intent();
        if self.local.last_requested_intent == Some(MiningOperatorIntent::Paused)
            && intent == MiningOperatorIntent::Run
        {
            self.local.core.resume();
        }
        self.local.last_requested_intent = Some(intent);
    }

    fn local_inputs(&mut self, now_ms: u64) -> LocalInputs {
        // Fresh, as the session's next readiness read will be, so it adopts the run.
        self.protocol_gate = crate::settings_adapter::configured_protocol_gate();
        let observations = crate::safety_adapter::observation_snapshot();
        let nonzero_rpm = observations
            .fan_rpm
            .maybe_last_good()
            .is_some_and(|sample| *sample.value() > 0);
        LocalInputs {
            other_boot_owner: self.maybe_campaign_status.is_some()
                || BOOT_OWNED_ELSEWHERE.load(Ordering::Acquire),
            gate_link: self.maybe_bwg_session.is_some() || revocation::holds_link(),
            operator_paused: crate::runtime_snapshot::requested_mining_operator_intent()
                == MiningOperatorIntent::Paused,
            hardware_fault: maybe_hardware_fault_context().is_some(),
            overheat_holds: false,
            network_ready: crate::wifi_adapter::current_wifi_snapshot().wifi_status == "connected",
            stratum_v1_supported: self.protocol_gate.is_ready(),
            // The run's preparation proves the fan, so it may begin from a stopped fan.
            safety_fresh: cooling_core::preparation_safety(
                observations.is_ultra_205_mining_safe_at(MonotonicMillis::new(now_ms)),
                nonzero_rpm,
                true,
            ),
            actuation_available: crate::safety_adapter::safety_actuation_available()
                && crate::asic_adapter::production::production_handle_available(),
            profile: saved_profile(),
        }
    }

    fn begin_local_run(
        &mut self,
        profile: MiningHardwareProfile,
        id: MiningCampaignLeaseId,
        snapshot: &ProductionSessionSnapshot,
        now_ms: u64,
    ) -> Option<ProductionSessionEvent> {
        let generation = revocation::begin_local()?;
        if !revocation::admit_budget(generation, u64::MAX)
            || !revocation::activate(generation, now_ms)
        {
            revocation::revoke_at(generation, now_ms);
            revocation::finish_shutdown(generation);
            log::warn!("local_mining=begin_refused");
            return None;
        }
        set_fan_controller_owner(None);
        self.local.maybe_run = Some(LocalRun {
            generation,
            lease: MiningCampaignLease::new(id, profile, MiningCampaignStopCondition::Local),
            preparation_started: false,
        });
        self.local.core.run_began();
        self.local.maybe_logged_hold = None;
        log::info!("local_mining=run_began generation={}", generation.raw());
        Some(self.wake_event(
            Some(ProductionSessionWakeup::ObservationsChanged),
            now_ms,
            snapshot,
            false,
        ))
    }

    /// The run's hardware is stopped (or was never powered): retire the generation
    /// and tell the restart guard how the run ended. An ordinary retirement is a
    /// restoration request, never `ControlFailed`, which the fault latch reads.
    fn end_local_run(&mut self, maybe_stop_cause: Option<ProductionSessionBlocker>, now_ms: u64) {
        let Some(run) = self.local.maybe_run.take() else {
            return;
        };
        revocation::revoke_at(run.generation, now_ms);
        // The gate publishes an unsafe detail just after revoking; the hardware has
        // been stopping for seconds since, so it is in place by now.
        let maybe_unsafe_stop = revocation::maybe_unsafe_detail_for(run.generation).map(|detail| {
            let maybe_chip_c = detail
                .maybe_chip_temp_milli
                .map(|milli| f64::from(milli) / 1_000.0);
            if bitaxe_runtime::local_mining::is_overheat(
                revocation::RevocationReason::UnsafeObservation,
                maybe_chip_c,
            ) {
                UnsafeStop::Overheat
            } else {
                UnsafeStop::Other
            }
        });
        revocation::finish_shutdown(run.generation);
        self.live_hardware.stopped();
        let maybe_ending = self
            .local
            .core
            .run_ended(maybe_unsafe_stop, maybe_stop_cause, now_ms);
        log::info!(
            "local_mining=run_ended generation={} ending={maybe_ending:?}",
            run.generation.raw()
        );
    }
}

/// The saved pair, checked against the board catalog and the shutdown budget.
fn saved_profile() -> Result<MiningHardwareProfile, DeviceProfileRefusal> {
    crate::settings_adapter::device_profile::device_mining_profile().and_then(|profile| {
        shutdown_budget::profile_admissible(profile)
            .then_some(profile)
            .ok_or(DeviceProfileRefusal::FrequencyOutOfRange)
    })
}
