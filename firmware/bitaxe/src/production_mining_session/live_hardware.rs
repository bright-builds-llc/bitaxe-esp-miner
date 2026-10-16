//! The live hardware settings owner (ADR-0037 decision 4). While a Gate lease mines,
//! each tick re-reads the saved frequency and core voltage and takes one safe step
//! toward them on the owner thread, so a change never overlaps the ordered safe-stop.
//! An unusable saved setting keeps the applied values; a failed step revokes the lease.
use super::*;
use bitaxe_runtime::live_hardware::{
    next_live_hardware_action, AppliedHardware, LiveHardwareAction,
};
use bitaxe_stratum::v1::production_session::{
    DeviceProfileRefusal, MiningHardwareProfile, MiningHardwareState,
};

/// Upstream re-reads every 100 ms; once a second keeps each tick's work bounded.
const LIVE_HARDWARE_CADENCE_MS: u64 = 1_000;

#[derive(Default)]
pub(super) struct LiveHardwareOwner {
    maybe_applied: Option<AppliedHardware>,
    next_due_ms: u64,
    maybe_last_refusal: Option<&'static str>,
}

impl LiveHardwareOwner {
    /// A completed preparation leaves `profile` running.
    pub(super) fn prepared(&mut self, profile: MiningHardwareProfile) {
        self.maybe_applied = Some(AppliedHardware::prepared(profile));
        self.maybe_last_refusal = None;
    }

    /// Safe-stop owns the hardware from here.
    pub(super) fn stopped(&mut self) {
        self.maybe_applied = None;
    }
}

impl OrdinaryEspProductionSessionAdapter {
    pub(super) fn service_live_hardware_settings(
        &mut self,
        snapshot: &ProductionSessionSnapshot,
        now_ms: u64,
    ) {
        if now_ms < self.live_hardware.next_due_ms {
            return;
        }
        self.live_hardware.next_due_ms = now_ms.saturating_add(LIVE_HARDWARE_CADENCE_MS);
        let (Some(session), Some(mut applied)) = (
            self.maybe_bwg_session.as_ref(),
            self.live_hardware.maybe_applied,
        ) else {
            return;
        };
        let generation = session.generation;
        // Skip, never revoke, while a step could not run: unsafe readings, a stopped
        // fan (the zero-RPM revocation owns that) or stopped dispatch (pool loss).
        let observations = crate::safety_adapter::observation_snapshot();
        let ready = observations.is_ultra_205_mining_safe_at(MonotonicMillis::new(now_ms))
            && observations
                .fan_rpm
                .maybe_last_good()
                .is_some_and(|sample| *sample.value() > 0)
            && crate::asic_adapter::production::production_ready();
        if snapshot.hardware_state != MiningHardwareState::Ready
            || !revocation::permits(Some(generation))
            || !ready
        {
            return;
        }
        let target = match crate::settings_adapter::device_profile::device_mining_profile()
            .and_then(|target| {
                shutdown_budget::profile_admissible(target)
                    .then_some(target)
                    .ok_or(DeviceProfileRefusal::FrequencyOutOfRange)
            }) {
            Ok(target) => {
                self.live_hardware.maybe_last_refusal = None;
                target
            }
            Err(refusal) => {
                if self.live_hardware.maybe_last_refusal != Some(refusal.label()) {
                    self.live_hardware.maybe_last_refusal = Some(refusal.label());
                    crate::info_retained(&format!(
                        "live_hardware=refused reason={} kept=applied",
                        refusal.label()
                    ));
                }
                return;
            }
        };
        let Some(action) = next_live_hardware_action(applied, target) else {
            return;
        };
        let result = match action {
            LiveHardwareAction::RaiseVoltage { millivolts }
            | LiveHardwareAction::LowerVoltage { millivolts } => self
                .mining_actuation
                .apply_live_core_voltage(&mut applied, millivolts),
            LiveHardwareAction::StepFrequency { target_quarter_mhz } => self
                .mining_actuation
                .step_live_frequency(&mut applied, target_quarter_mhz),
            LiveHardwareAction::SetNonceSpace { .. } => {
                self.mining_actuation.set_live_nonce_space(&mut applied)
            }
        };
        self.live_hardware.maybe_applied = Some(applied);
        match result {
            Ok(()) => crate::info_retained(&format!(
                "live_hardware=applied action={action:?} frequency_quarter_mhz={} core_voltage_mv={}",
                applied.frequency_quarter_mhz, applied.core_voltage_mv
            )),
            Err(_) if !revocation::permits(Some(generation)) => crate::info_retained(&format!(
                "live_hardware=stopped action={action:?} reason=lease_revoked"
            )),
            Err(error) => {
                crate::info_retained(&format!(
                    "live_hardware=failed action={action:?} category={}",
                    error.category()
                ));
                revocation::revoke_reason_at(
                    generation,
                    crate::runtime_uptime::millis(),
                    revocation::RevocationReason::ControlFailed,
                );
            }
        }
    }
}
