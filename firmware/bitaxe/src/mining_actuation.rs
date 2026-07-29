//! Pure preparation and safe-shutdown orchestration for Ultra 205 mining.
//!
//! Hardware adapters implement [`MiningActuationBackend`]. This module owns
//! only the closed order of typed effects and earliest-failure preservation;
//! it has no ESP-IDF, GPIO, I2C, or UART dependencies.

use bitaxe_config::{AsicFrequencyMhz, CoreVoltageMv};
use bitaxe_stratum::v1::production_session::MiningHardwareProfile;

/// Required stabilization interval after applying the core-voltage setpoint.
pub const CORE_VOLTAGE_STABILIZATION_MS: u16 = 500;

/// Ordered preparation effects required before mining work may be dispatched.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreparationStep {
    /// Prove all required safety observations are fresh.
    RequireFreshSafetyObservations,
    /// Command the fan to full duty before applying mining power.
    SetFanDutyTo100Percent,
    /// Prove a fresh, post-command fan observation has nonzero RPM.
    RequireFreshNonzeroFanRpm,
    /// Apply the validated profile core-voltage setpoint.
    SetCoreVoltage(CoreVoltageMv),
    /// Wait the fixed 500 ms post-voltage stabilization interval.
    WaitForCoreVoltageStabilization500Ms,
    /// Assert the ASIC-enable output.
    EnableAsic,
    /// Reset the chain and prove that exactly one BM1366 is present.
    ResetAndDetectExactlyOneChip,
    /// Run mining-ready initialization at the profile frequency with a ramp.
    InitializeMiningReadyWithFrequencyRamp(AsicFrequencyMhz),
    /// Retain the initialized production UART for subsequent mining traffic.
    RetainProductionUart,
}

/// Ordered effects that establish the paused, fail-closed hardware state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafeShutdownStep {
    /// Prevent any new work from reaching the ASIC.
    StopDispatch,
    /// Reduce ASIC frequency and reset nonce state before removing power.
    ReduceFrequencyAndResetNonce,
    /// Hold ASIC reset low.
    HoldResetLow,
    /// Set the core-voltage output to off.
    DisableCoreVoltage,
    /// Deassert the ASIC-enable output.
    DisableAsic,
    /// Hold the fan at full duty while the board cools.
    SetFanDutyTo100Percent,
    /// Wait for a fresh temperature observation at or below 45 degrees C.
    WaitForFreshTemperatureAtOrBelow45C,
    /// Set the paused-state fan duty after the temperature proof.
    SetFanDutyTo30Percent,
}

/// Imperative hardware boundary used by the pure ordered orchestration.
pub trait MiningActuationBackend {
    /// Adapter-specific failure detail retained by the orchestration result.
    type Error;

    /// Establishes the state described by one preparation step.
    fn execute_preparation_step(&mut self, step: PreparationStep) -> Result<(), Self::Error>;

    /// Establishes the state described by one idempotent safe-shutdown step.
    fn execute_safe_shutdown_step(&mut self, step: SafeShutdownStep) -> Result<(), Self::Error>;
}

/// The first failed preparation step and its adapter detail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparationStepFailure<E> {
    step: PreparationStep,
    source: E,
}

impl<E> PreparationStepFailure<E> {
    /// Returns the preparation boundary that failed first.
    pub const fn step(&self) -> PreparationStep {
        self.step
    }

    /// Returns the adapter detail for the first preparation failure.
    pub const fn source(&self) -> &E {
        &self.source
    }
}

/// The first failed safe-shutdown step and its adapter detail.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SafeShutdownFailure<E> {
    step: SafeShutdownStep,
    source: E,
}

impl<E> SafeShutdownFailure<E> {
    /// Returns the first safe-shutdown boundary that failed.
    pub const fn step(&self) -> SafeShutdownStep {
        self.step
    }

    /// Returns the adapter detail for the first safe-shutdown failure.
    pub const fn source(&self) -> &E {
        &self.source
    }
}

/// A preparation failure with the optional failure from its safe rollback.
///
/// The original preparation failure is immutable and always remains primary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparationExecutionFailure<E> {
    original: PreparationStepFailure<E>,
    maybe_safe_shutdown_failure: Option<SafeShutdownFailure<E>>,
}

impl<E> PreparationExecutionFailure<E> {
    /// Returns the original preparation failure.
    pub const fn original(&self) -> &PreparationStepFailure<E> {
        &self.original
    }

    /// Returns a later rollback failure without replacing the original cause.
    pub const fn maybe_safe_shutdown_failure(&self) -> Option<&SafeShutdownFailure<E>> {
        self.maybe_safe_shutdown_failure.as_ref()
    }
}

/// Returns the closed preparation plan for a validated hardware profile.
#[must_use]
pub const fn preparation_plan(profile: MiningHardwareProfile) -> [PreparationStep; 9] {
    [
        PreparationStep::RequireFreshSafetyObservations,
        PreparationStep::SetFanDutyTo100Percent,
        PreparationStep::RequireFreshNonzeroFanRpm,
        PreparationStep::SetCoreVoltage(profile.core_voltage()),
        PreparationStep::WaitForCoreVoltageStabilization500Ms,
        PreparationStep::EnableAsic,
        PreparationStep::ResetAndDetectExactlyOneChip,
        PreparationStep::InitializeMiningReadyWithFrequencyRamp(profile.frequency()),
        PreparationStep::RetainProductionUart,
    ]
}

/// Returns the closed safe-shutdown plan.
///
/// Every step describes a desired state rather than a toggle. Repeated calls
/// therefore produce the same idempotent plan regardless of preparation
/// progress or a prior safe-shutdown attempt.
#[must_use]
pub const fn safe_shutdown_plan() -> [SafeShutdownStep; 8] {
    [
        SafeShutdownStep::StopDispatch,
        SafeShutdownStep::ReduceFrequencyAndResetNonce,
        SafeShutdownStep::HoldResetLow,
        SafeShutdownStep::DisableCoreVoltage,
        SafeShutdownStep::DisableAsic,
        SafeShutdownStep::SetFanDutyTo100Percent,
        SafeShutdownStep::WaitForFreshTemperatureAtOrBelow45C,
        SafeShutdownStep::SetFanDutyTo30Percent,
    ]
}

/// Executes preparation and rolls every partial failure through safe shutdown.
///
/// If rollback also fails, both failures are returned while the original
/// preparation failure remains primary.
pub fn execute_preparation<B>(
    backend: &mut B,
    profile: MiningHardwareProfile,
) -> Result<(), PreparationExecutionFailure<B::Error>>
where
    B: MiningActuationBackend,
{
    for step in preparation_plan(profile) {
        let Err(source) = backend.execute_preparation_step(step) else {
            continue;
        };

        let original = PreparationStepFailure { step, source };
        let maybe_safe_shutdown_failure = execute_safe_shutdown(backend).err();
        return Err(PreparationExecutionFailure {
            original,
            maybe_safe_shutdown_failure,
        });
    }

    Ok(())
}

/// Executes every safe-shutdown step and retains its first failed boundary.
///
/// Later idempotent steps are still attempted so an early adapter fault cannot
/// prevent independent voltage-off, ASIC-disable, or cooling actions.
pub fn execute_safe_shutdown<B>(backend: &mut B) -> Result<(), SafeShutdownFailure<B::Error>>
where
    B: MiningActuationBackend,
{
    let mut maybe_earliest_failure = None;

    for step in safe_shutdown_plan() {
        let Err(source) = backend.execute_safe_shutdown_step(step) else {
            continue;
        };

        if maybe_earliest_failure.is_none() {
            maybe_earliest_failure = Some(SafeShutdownFailure { step, source });
        }
    }

    match maybe_earliest_failure {
        Some(failure) => Err(failure),
        None => Ok(()),
    }
}
