//! Live hardware changes while a lease mines (ADR-0037 decision 4), on the
//! production owner thread between dispatches. Every slice re-checks the
//! generation and the runtime safety predicate, and the shutdown profile follows
//! each applied frequency so a safe-stop never ramps down from below it.
use super::*;
use bitaxe_asic::bm1366::live_transition::{live_frequency_steps, live_nonce_space_actions};
use bitaxe_runtime::live_hardware::AppliedHardware;

/// At most this many ramp steps run per owner tick (100 ms each), so the owner stays
/// responsive to revocation and safe-stop.
const LIVE_STEPS_PER_TICK: usize = 5;

impl Default for Ultra205MiningActuationAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl Ultra205MiningActuationAdapter {
    /// Writes a new core voltage for the live lease. One action runs per owner tick,
    /// so a rise has at least a full second (more than the 500 ms preparation settle)
    /// before any frequency increase follows.
    pub fn apply_live_core_voltage(
        &mut self,
        applied: &mut AppliedHardware,
        millivolts: u16,
    ) -> Result<(), MiningActuationAdapterError> {
        let voltage = Ultra205CoreVoltage::ultra_205(millivolts)
            .ok_or(MiningActuationAdapterError::UnsupportedProfile)?;
        self.request_preparation_voltage(voltage, &mut || {})?;
        applied.core_voltage_mv = millivolts;
        Ok(())
    }

    /// Ramps toward `target_quarter_mhz`, a bounded number of steps this tick.
    pub fn step_live_frequency(
        &mut self,
        applied: &mut AppliedHardware,
        target_quarter_mhz: u32,
    ) -> Result<(), MiningActuationAdapterError> {
        let generation = self
            .maybe_worker_generation
            .ok_or(MiningActuationAdapterError::WorkerGenerationRevoked)?;
        let steps = live_frequency_steps(applied.frequency_quarter_mhz, target_quarter_mhz)
            .map_err(|_| MiningActuationAdapterError::AsicPlanInvalid)?;
        for step in steps.into_iter().take(LIVE_STEPS_PER_TICK) {
            self.check_preparation_admission()?;
            self.follow_applied_frequency(applied.while_stepping_to(step.quarter_mhz))?;
            crate::asic_adapter::production::execute_live_actions_guarded(
                &step.actions,
                generation,
            )
            .map_err(MiningActuationAdapterError::Asic)?;
            applied.frequency_quarter_mhz = step.quarter_mhz;
            self.follow_applied_frequency(*applied)?;
        }
        Ok(())
    }

    /// Sets the nonce space for the frequency now applied.
    pub fn set_live_nonce_space(
        &mut self,
        applied: &mut AppliedHardware,
    ) -> Result<(), MiningActuationAdapterError> {
        let generation = self
            .maybe_worker_generation
            .ok_or(MiningActuationAdapterError::WorkerGenerationRevoked)?;
        self.check_preparation_admission()?;
        let profile = self
            .maybe_asic_profile
            .ok_or(MiningActuationAdapterError::AsicPlanInvalid)?;
        let config = MiningReadyConfig::ultra_205_profile(1, profile);
        let actions = live_nonce_space_actions(config, applied.frequency_quarter_mhz)
            .map_err(|_| MiningActuationAdapterError::AsicPlanInvalid)?;
        crate::asic_adapter::production::execute_live_actions_guarded(&actions, generation)
            .map_err(MiningActuationAdapterError::Asic)?;
        applied.nonce_space_quarter_mhz = applied.frequency_quarter_mhz;
        Ok(())
    }

    fn follow_applied_frequency(
        &mut self,
        applied: AppliedHardware,
    ) -> Result<(), MiningActuationAdapterError> {
        let frequency = AsicFrequencyMhz::parse(i64::from(applied.shutdown_frequency_mhz()))
            .map_err(|_| MiningActuationAdapterError::UnsupportedProfile)?;
        self.maybe_asic_profile = Some(Self::asic_profile(frequency)?);
        Ok(())
    }
}
