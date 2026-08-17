//! Pure preparation and safe-shutdown orchestration for Ultra 205 mining.
//!
//! Hardware adapters implement [`MiningActuationBackend`]. This module owns
//! only the closed order of typed effects and earliest-failure preservation;
//! it has no ESP-IDF, GPIO, I2C, or UART dependencies.

use bitaxe_config::{AsicFrequencyMhz, CoreVoltageMv};
use bitaxe_stratum::v1::production_session::{HardwareSafeStopPurpose, MiningHardwareProfile};

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

impl PreparationStep {
    /// Returns the closed redaction-safe evidence label for this boundary.
    pub const fn label(self) -> &'static str {
        match self {
            Self::RequireFreshSafetyObservations => "require_fresh_safety_observations",
            Self::SetFanDutyTo100Percent => "set_fan_duty_to_100_percent",
            Self::RequireFreshNonzeroFanRpm => "require_fresh_nonzero_fan_rpm",
            Self::SetCoreVoltage(_) => "set_core_voltage",
            Self::WaitForCoreVoltageStabilization500Ms => {
                "wait_for_core_voltage_stabilization_500_ms"
            }
            Self::EnableAsic => "enable_asic",
            Self::ResetAndDetectExactlyOneChip => "reset_and_detect_exactly_one_chip",
            Self::InitializeMiningReadyWithFrequencyRamp(_) => {
                "initialize_mining_ready_with_frequency_ramp"
            }
            Self::RetainProductionUart => "retain_production_uart",
        }
    }
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

impl SafeShutdownStep {
    /// Returns the closed redaction-safe evidence label for this boundary.
    pub const fn label(self) -> &'static str {
        match self {
            Self::StopDispatch => "stop_dispatch",
            Self::ReduceFrequencyAndResetNonce => "reduce_frequency_and_reset_nonce",
            Self::HoldResetLow => "hold_reset_low",
            Self::DisableCoreVoltage => "disable_core_voltage",
            Self::DisableAsic => "disable_asic",
            Self::SetFanDutyTo100Percent => "set_fan_duty_to_100_percent",
            Self::WaitForFreshTemperatureAtOrBelow45C => {
                "wait_for_fresh_temperature_at_or_below_45_c"
            }
            Self::SetFanDutyTo30Percent => "set_fan_duty_to_30_percent",
        }
    }
}

/// Imperative hardware boundary used by the pure ordered orchestration.
pub trait MiningActuationBackend {
    /// Adapter-specific failure detail retained by the orchestration result.
    type Error;

    /// Establishes the state described by one preparation step.
    fn execute_preparation_step(&mut self, step: PreparationStep) -> Result<(), Self::Error>;

    /// Establishes the state described by one idempotent safe-shutdown step.
    fn execute_safe_shutdown_step(&mut self, step: SafeShutdownStep) -> Result<(), Self::Error>;

    /// Executes one step while allowing long-running adapters to report progress.
    fn execute_safe_shutdown_step_with_progress(
        &mut self,
        step: SafeShutdownStep,
        progress: &mut dyn FnMut(),
    ) -> Result<(), Self::Error> {
        let result = self.execute_safe_shutdown_step(step);
        progress();
        result
    }
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

/// Returns the prompt fail-closed plan for an operator-resumable pause.
///
/// Full fan duty is retained while paused. The terminal cooling proof and fan
/// settlement remain exclusive to non-resumable cleanup.
#[must_use]
pub const fn resumable_pause_shutdown_plan() -> [SafeShutdownStep; 6] {
    [
        SafeShutdownStep::StopDispatch,
        SafeShutdownStep::ReduceFrequencyAndResetNonce,
        SafeShutdownStep::HoldResetLow,
        SafeShutdownStep::DisableCoreVoltage,
        SafeShutdownStep::DisableAsic,
        SafeShutdownStep::SetFanDutyTo100Percent,
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
    execute_safe_shutdown_steps(backend, safe_shutdown_plan())
}

/// Executes the prompt safe-shutdown plan for an operator-resumable pause.
pub fn execute_resumable_pause_shutdown<B>(
    backend: &mut B,
) -> Result<(), SafeShutdownFailure<B::Error>>
where
    B: MiningActuationBackend,
{
    execute_safe_shutdown_steps(backend, resumable_pause_shutdown_plan())
}

/// Executes the closed plan selected by the production-session stop purpose.
pub fn execute_safe_stop<B>(
    backend: &mut B,
    purpose: HardwareSafeStopPurpose,
) -> Result<(), SafeShutdownFailure<B::Error>>
where
    B: MiningActuationBackend,
{
    execute_safe_stop_with_progress(backend, purpose, &mut || {})
}

/// Executes the selected safe-stop plan with cooperative progress boundaries.
pub fn execute_safe_stop_with_progress<B>(
    backend: &mut B,
    purpose: HardwareSafeStopPurpose,
    progress: &mut dyn FnMut(),
) -> Result<(), SafeShutdownFailure<B::Error>>
where
    B: MiningActuationBackend,
{
    match purpose {
        HardwareSafeStopPurpose::ResumablePause => execute_safe_shutdown_steps_with_progress(
            backend,
            resumable_pause_shutdown_plan(),
            progress,
        ),
        HardwareSafeStopPurpose::Terminal => {
            execute_safe_shutdown_steps_with_progress(backend, safe_shutdown_plan(), progress)
        }
    }
}

fn execute_safe_shutdown_steps<B, const N: usize>(
    backend: &mut B,
    steps: [SafeShutdownStep; N],
) -> Result<(), SafeShutdownFailure<B::Error>>
where
    B: MiningActuationBackend,
{
    execute_safe_shutdown_steps_with_progress(backend, steps, &mut || {})
}

fn execute_safe_shutdown_steps_with_progress<B, const N: usize>(
    backend: &mut B,
    steps: [SafeShutdownStep; N],
    progress: &mut dyn FnMut(),
) -> Result<(), SafeShutdownFailure<B::Error>>
where
    B: MiningActuationBackend,
{
    let mut maybe_earliest_failure = None;

    for step in steps {
        let Err(source) = backend.execute_safe_shutdown_step_with_progress(step, progress) else {
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
