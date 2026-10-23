//! The terminal safe-stop's cooling proof, checked once per owner tick (ADR-0016).
//!
//! The blocking wait and the polled stop share one predicate, so a proof means the same
//! thing on both paths.

use super::{
    MiningActuationAdapterError, Ultra205MiningActuationAdapter, SAFE_COOLING_THRESHOLD_C,
};
use crate::mining_actuation::{
    begin_safe_stop_with_progress, poll_cooling_proof, CoolingProofBackend, PendingCoolingProof,
    SafeShutdownStep, SafeStopProgress,
};
use bitaxe_safety::observation::MonotonicMillis;
use bitaxe_stratum::v1::production_session::HardwareSafeStopPurpose;

impl Ultra205MiningActuationAdapter {
    /// Starts the stop for `purpose`; a terminal stop returns once power is off.
    pub fn begin_safe_stop(
        &mut self,
        purpose: HardwareSafeStopPurpose,
        progress: &mut dyn FnMut(SafeShutdownStep),
    ) -> SafeStopProgress<MiningActuationAdapterError> {
        begin_safe_stop_with_progress(self, purpose, &crate::runtime_uptime::millis, progress)
    }

    /// Checks an open cooling proof once.
    pub fn poll_safe_stop(
        &mut self,
        pending: PendingCoolingProof<MiningActuationAdapterError>,
        progress: &mut dyn FnMut(SafeShutdownStep),
    ) -> SafeStopProgress<MiningActuationAdapterError> {
        poll_cooling_proof(self, pending, crate::runtime_uptime::millis(), progress)
    }

    /// When the fan-100 % step opened the proof window; the proof needs a later reading.
    pub(super) fn cooling_window_started_at_ms(&self) -> Result<u64, MiningActuationAdapterError> {
        if !self.cooling_fan_full_applied {
            return Err(MiningActuationAdapterError::CoolingProofRequired);
        }
        self.maybe_cooling_started_at_ms
            .ok_or(MiningActuationAdapterError::CoolingProofRequired)
    }

    /// One check: mining-safe observations and a fresh, finite chip reading at or below
    /// 45 C, taken after the window opened. A proof is kept for the paused fan duty.
    pub(super) fn observe_cooling_proof(&mut self) -> bool {
        let Ok(started_at_ms) = self.cooling_window_started_at_ms() else {
            return false;
        };
        let observations = crate::safety_adapter::observation_snapshot();
        let maybe_temperature = observations.chip_temp_celsius.maybe_last_good();
        let proven = observations.is_mining_safe_at(
            &crate::board_identity::MINING_SAFETY_LIMITS,
            MonotonicMillis::new(crate::runtime_uptime::millis()),
        ) && observations.chip_temp_celsius.is_fresh()
            && maybe_temperature.is_some_and(|sample| {
                sample.acquired_at().get() > started_at_ms
                    && sample.value().is_finite()
                    && *sample.value() <= SAFE_COOLING_THRESHOLD_C
            });
        if proven {
            self.cooling_proven = true;
        }
        proven
    }
}

impl CoolingProofBackend for Ultra205MiningActuationAdapter {
    fn start_cooling_proof(&mut self) -> Result<(), Self::Error> {
        self.note_shutdown_step(SafeShutdownStep::WaitForFreshTemperatureAtOrBelow45C);
        self.cooling_window_started_at_ms().map(|_| ())
    }

    fn cooling_proof_observed(&mut self) -> bool {
        self.observe_cooling_proof()
    }

    fn cooling_proof_timeout(&self) -> Self::Error {
        MiningActuationAdapterError::CoolingProofTimedOut
    }
}
