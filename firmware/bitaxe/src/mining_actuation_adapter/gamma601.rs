//! The Gamma 601's board steps in the shared preparation and safe stop (ADR-0041): its TPS546
//! through the safe-boot thread's actuation owner, and the BM1370's bring-up through the shared
//! ASIC executor. The preparation order is upstream's `mining_start`: the core voltage, 500 ms,
//! the reset and the init, then 2000 ms before work.
//!
//! The stop deviates from upstream's `mining_stop` (frequency down, reset, then voltage off):
//! revoking the lease turns the 601's regulator off within 100 ms (`gamma601/actuation.rs`), so
//! by the time the stop would ramp the frequency down the chip is unpowered. The stop skips the
//! ramp and holds the reset.

use std::thread;
use std::time::Duration;

use bitaxe_asic::bm1370::{init::Bm1370InitConfig, plan};
use bitaxe_config::{AsicFrequencyMhz, CoreVoltageMv};

use super::{MiningActuationAdapterError, Ultra205MiningActuationAdapter};
use crate::safety_adapter::{
    PendingSafetyActuation, SafetyActuationCommand, SafetyActuationPollOutcome,
    SafetyActuationQueueOutcome,
};

/// Turning the regulator on may take its off confirmation (up to 2 s) and the configuration.
const POWER_ON_REPLY_MS: u64 = 8_000;
const VERIFY_REPLY_MS: u64 = 2_000;
/// The off confirmation repeats for up to 2 s.
const POWER_OFF_REPLY_MS: u64 = 4_000;
/// Upstream's `asic_initialize` stabilization before the ASIC is used.
const POST_INIT_STABILIZATION_MS: u64 = 2_000;
const STABILIZATION_SLICE_MS: u64 = 100;
const REPLY_POLL_MS: u64 = 50;

impl Ultra205MiningActuationAdapter {
    /// The regulator configured and turned on at the profile's voltage, with the ASIC reset
    /// held (upstream `VCORE_set_voltage`). The voltage is judged settled at `EnableAsic`. The
    /// builds that mine power the ASIC (ADR-0041 item 15); the bounded checks refuse.
    pub(super) fn gamma601_power_on(
        &self,
        voltage: CoreVoltageMv,
        progress: &mut dyn FnMut(),
    ) -> Result<(), MiningActuationAdapterError> {
        #[cfg(gamma601_dispatch)]
        return self.gamma601_request(
            SafetyActuationCommand::PowerOnRegulator {
                millivolts: voltage.millivolts(),
                permit: crate::production_mining_session::revocation::stamp(
                    self.maybe_worker_generation,
                ),
            },
            POWER_ON_REPLY_MS,
            progress,
        );
        #[cfg(not(gamma601_dispatch))]
        {
            let _ = (voltage, progress);
            Err(MiningActuationAdapterError::UnsupportedProfile)
        }
    }

    /// The settled output judged, after the preparation's 500 ms stabilization: the 601 has no
    /// enable line, so this is its `EnableAsic`.
    pub(super) fn gamma601_verify_power(
        &self,
        progress: &mut dyn FnMut(),
    ) -> Result<(), MiningActuationAdapterError> {
        #[cfg(gamma601_dispatch)]
        return self.gamma601_request(
            SafetyActuationCommand::VerifyRegulator,
            VERIFY_REPLY_MS,
            progress,
        );
        #[cfg(not(gamma601_dispatch))]
        {
            let _ = progress;
            Err(MiningActuationAdapterError::UnsupportedProfile)
        }
    }

    /// The reset pulse and the count of exactly the catalog's chips.
    pub(super) fn gamma601_detect_chips(
        &self,
        progress: &mut dyn FnMut(),
    ) -> Result<(), MiningActuationAdapterError> {
        let actions = plan::chip_detect_actions(gamma601_config(None))
            .map_err(|_| MiningActuationAdapterError::AsicPlanInvalid)?;
        crate::asic_adapter::production::execute_chip_detection_actions_guarded_with_progress(
            &actions,
            self.maybe_worker_generation,
            progress,
        )
        .map_err(MiningActuationAdapterError::Asic)
    }

    /// The mining-ready init with its ramp to `frequency`, then upstream's stabilization.
    pub(super) fn gamma601_initialize(
        &mut self,
        frequency: AsicFrequencyMhz,
        progress: &mut dyn FnMut(),
    ) -> Result<(), MiningActuationAdapterError> {
        let actions = plan::mining_ready_actions(gamma601_config(Some(frequency.mhz())))
            .map_err(|_| MiningActuationAdapterError::AsicPlanInvalid)?;
        crate::asic_adapter::production::execute_mining_ready_actions_guarded_with_progress(
            &actions,
            self.maybe_worker_generation,
            progress,
        )
        .map_err(MiningActuationAdapterError::Asic)?;
        for _ in 0..POST_INIT_STABILIZATION_MS / STABILIZATION_SLICE_MS {
            self.cancellable_delay(STABILIZATION_SLICE_MS)?;
            progress();
        }
        Ok(())
    }

    /// No ramp: the revoked lease has already turned the chip's power off (see the module).
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)]
    pub(super) fn reduce_frequency_and_reset_nonce(
        &mut self,
        _progress: &mut dyn FnMut(),
    ) -> Result<(), MiningActuationAdapterError> {
        Ok(())
    }

    /// The reset held low through the executor, or, before the executor holds the pin, by the
    /// safe boot's own hold on it.
    #[allow(clippy::unused_self)]
    pub(super) fn gamma601_hold_reset(&self) -> Result<(), MiningActuationAdapterError> {
        if crate::asic_adapter::production::production_handle_available() {
            return crate::asic_adapter::production::execute_safe_shutdown_actions(&[
                bitaxe_asic::bm1366::command::Bm1366AdapterAction::HOLD_RESET_LOW,
            ])
            .map_err(MiningActuationAdapterError::Asic);
        }
        if crate::gamma601::reset_held() {
            Ok(())
        } else {
            Err(MiningActuationAdapterError::Asic(
                bitaxe_asic::bm1366::production::ProductionAsicBlocker::UartFailed,
            ))
        }
    }

    /// The regulator off and confirmed off. A stop runs whatever the session's admission says,
    /// so this waits for the owner's reply without it. A bounded check's mining owner never turns
    /// the output on; its check turns it off itself.
    #[cfg(not(gamma601_dispatch))]
    #[allow(clippy::unused_self)]
    pub(super) fn gamma601_power_off(&self) -> Result<(), MiningActuationAdapterError> {
        Ok(())
    }

    /// The regulator off and confirmed off. A stop runs whatever the session's admission says,
    /// so this waits for the owner's reply without it.
    #[cfg(gamma601_dispatch)]
    pub(super) fn gamma601_power_off(&self) -> Result<(), MiningActuationAdapterError> {
        let pending = queue(SafetyActuationCommand::TurnRegulatorOff)?;
        let deadline = crate::runtime_uptime::millis().saturating_add(POWER_OFF_REPLY_MS);
        loop {
            match pending.poll() {
                SafetyActuationPollOutcome::Applied => return Ok(()),
                SafetyActuationPollOutcome::Pending => {}
                outcome => return Err(poll_failure(outcome)),
            }
            if crate::runtime_uptime::millis() >= deadline {
                return Err(MiningActuationAdapterError::SafetyReplyTimedOut);
            }
            thread::sleep(Duration::from_millis(REPLY_POLL_MS));
        }
    }

    #[cfg(gamma601_dispatch)]
    fn gamma601_request(
        &self,
        command: SafetyActuationCommand,
        timeout_ms: u64,
        progress: &mut dyn FnMut(),
    ) -> Result<(), MiningActuationAdapterError> {
        self.check_preparation_admission()?;
        let pending = queue(command)?;
        self.await_preparation_actuation(&pending, timeout_ms, progress)
    }
}

/// The 601's BM1370 configuration: the catalog's chip count, at `maybe_frequency_mhz` or the
/// family default.
fn gamma601_config(maybe_frequency_mhz: Option<u16>) -> Bm1370InitConfig {
    let default = Bm1370InitConfig::gamma_single_chip();
    Bm1370InitConfig {
        chip_count: crate::board_identity::catalog_entry().asic_count(),
        target_frequency_mhz: maybe_frequency_mhz.unwrap_or(default.target_frequency_mhz),
        ..default
    }
}

fn queue(
    command: SafetyActuationCommand,
) -> Result<PendingSafetyActuation, MiningActuationAdapterError> {
    match crate::safety_adapter::queue_safety_actuation(command) {
        SafetyActuationQueueOutcome::Queued(pending) => Ok(pending),
        SafetyActuationQueueOutcome::QueueFull => Err(MiningActuationAdapterError::SafetyQueueFull),
        SafetyActuationQueueOutcome::OwnerUnavailable => {
            Err(MiningActuationAdapterError::SafetyOwnerUnavailable)
        }
    }
}

fn poll_failure(outcome: SafetyActuationPollOutcome) -> MiningActuationAdapterError {
    match outcome {
        SafetyActuationPollOutcome::OwnerUnavailable => {
            MiningActuationAdapterError::SafetyOwnerUnavailable
        }
        SafetyActuationPollOutcome::Refused => MiningActuationAdapterError::WorkerGenerationRevoked,
        _ => MiningActuationAdapterError::SafetyHardwareWriteFailed,
    }
}
