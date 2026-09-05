//! Ordinary ESP adapter for the pure Ultra 205 mining-actuation coordinator.

use std::thread;
use std::time::Duration;

use bitaxe_asic::bm1366::{
    command::Bm1366AdapterAction,
    init_plan::{
        Bm1366InitPlan, Bm1366Preflight, BoardPreflightEvidence, ChipDetectPlanOptions,
        ConfigPreflightEvidence,
    },
    mining_ready::{
        safe_shutdown_command_actions, Bm1366MiningProfile, MiningReadyConfig,
        MiningReadyInitOptions,
    },
    production::ProductionAsicBlocker,
};
use bitaxe_config::{AsicFrequencyMhz, CoreVoltageMv};
use bitaxe_safety::observation::MonotonicMillis;
use bitaxe_stratum::v1::production_session::{
    HardwarePreparationFailure, HardwareSafeStopPurpose, MiningHardwareProfile,
};

use crate::mining_actuation::{
    execute_preparation, execute_safe_stop_with_progress, MiningActuationBackend,
    PreparationExecutionFailure, PreparationStep, SafeShutdownFailure, SafeShutdownStep,
    CORE_VOLTAGE_STABILIZATION_MS,
};
use crate::safety_adapter::{
    FanDutyPercent, PendingSafetyActuation, SafetyActuationCommand, SafetyActuationPollOutcome,
    SafetyActuationQueueOutcome, SafetyActuationRequestOutcome, Ultra205CoreVoltage,
};

const FAN_PROOF_TIMEOUT_MS: u64 = 3_000;
const FAN_PROOF_POLL_MS: u64 = 50;
const COOLING_PROOF_TIMEOUT_MS: u64 = 120_000;
const COOLING_PROOF_POLL_MS: u64 = 500;
const SAFE_COOLING_THRESHOLD_C: f64 = 45.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiningActuationAdapterError {
    WorkerGenerationRevoked,
    SafetyObservationsUnavailable,
    SafetyOwnerUnavailable,
    SafetyQueueFull,
    SafetyReplyTimedOut,
    SafetyHardwareWriteFailed,
    FanRpmProofTimedOut,
    UnsupportedProfile,
    Asic(ProductionAsicBlocker),
    AsicPlanInvalid,
    CoolingProofTimedOut,
    CoolingProofRequired,
}

impl MiningActuationAdapterError {
    #[must_use]
    pub const fn hardware_preparation_failure(self) -> HardwarePreparationFailure {
        match self {
            Self::WorkerGenerationRevoked
            | Self::SafetyObservationsUnavailable
            | Self::SafetyOwnerUnavailable
            | Self::SafetyQueueFull
            | Self::UnsupportedProfile => HardwarePreparationFailure::Rejected,
            Self::SafetyReplyTimedOut | Self::FanRpmProofTimedOut | Self::CoolingProofTimedOut => {
                HardwarePreparationFailure::TimedOut
            }
            Self::SafetyHardwareWriteFailed
            | Self::Asic(_)
            | Self::AsicPlanInvalid
            | Self::CoolingProofRequired => HardwarePreparationFailure::DeviceFault,
        }
    }

    #[must_use]
    pub const fn category(self) -> &'static str {
        match self {
            Self::WorkerGenerationRevoked => "worker_generation_revoked",
            Self::SafetyObservationsUnavailable => "safety_observations_unavailable",
            Self::SafetyOwnerUnavailable => "safety_owner_unavailable",
            Self::SafetyQueueFull => "safety_queue_full",
            Self::SafetyReplyTimedOut => "safety_reply_timed_out",
            Self::SafetyHardwareWriteFailed => "safety_hardware_write_failed",
            Self::FanRpmProofTimedOut => "fan_rpm_proof_timed_out",
            Self::UnsupportedProfile => "unsupported_profile",
            Self::Asic(_) => "asic_actuation_failed",
            Self::AsicPlanInvalid => "asic_plan_invalid",
            Self::CoolingProofTimedOut => "cooling_proof_timed_out",
            Self::CoolingProofRequired => "cooling_proof_required",
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct ObservationStamp {
    boot_session: u64,
    sequence: u64,
    acquired_at_ms: u64,
}

pub struct Ultra205MiningActuationAdapter {
    maybe_worker_generation: Option<crate::production_mining_session::revocation::WorkerGeneration>,
    maybe_requested_profile: Option<MiningHardwareProfile>,
    maybe_fan_command_baseline: Option<ObservationStamp>,
    maybe_fan_command_started_at_ms: Option<u64>,
    maybe_pending_fan_actuation: Option<PendingSafetyActuation>,
    maybe_cooling_started_at_ms: Option<u64>,
    maybe_asic_profile: Option<Bm1366MiningProfile>,
    cooling_fan_full_applied: bool,
    cooling_proven: bool,
    fan_proof_confirmed: bool,
}

impl Ultra205MiningActuationAdapter {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            maybe_worker_generation: None,
            maybe_requested_profile: None,
            maybe_fan_command_baseline: None,
            maybe_fan_command_started_at_ms: None,
            maybe_pending_fan_actuation: None,
            maybe_cooling_started_at_ms: None,
            maybe_asic_profile: None,
            cooling_fan_full_applied: false,
            cooling_proven: false,
            fan_proof_confirmed: false,
        }
    }

    pub fn prepare(
        &mut self,
        profile: MiningHardwareProfile,
    ) -> Result<(), PreparationExecutionFailure<MiningActuationAdapterError>> {
        self.maybe_requested_profile = Some(profile);
        self.fan_proof_confirmed = false;
        execute_preparation(self, profile)
    }

    pub fn set_worker_generation(
        &mut self,
        maybe_generation: Option<crate::production_mining_session::revocation::WorkerGeneration>,
    ) {
        self.maybe_worker_generation = maybe_generation;
    }

    fn note_shutdown_step(&self, step: SafeShutdownStep) {
        if let Some(generation) = self.maybe_worker_generation {
            let now_ms = crate::runtime_uptime::millis();
            crate::production_mining_session::revocation::revoke_at(generation, now_ms);
            crate::production_mining_session::revocation::note_shutdown(
                generation,
                step as u32 + 1,
                now_ms,
            );
        }
    }

    fn check_preparation_admission(&self) -> Result<(), MiningActuationAdapterError> {
        crate::production_mining_session::revocation::check_deadline(
            crate::runtime_uptime::millis(),
        );
        if !crate::production_mining_session::revocation::permits(self.maybe_worker_generation) {
            return Err(MiningActuationAdapterError::WorkerGenerationRevoked);
        }
        if self.fan_proof_confirmed {
            let observations = crate::safety_adapter::observation_snapshot();
            if !observations
                .is_ultra_205_mining_safe_at(MonotonicMillis::new(crate::runtime_uptime::millis()))
                || !observations
                    .fan_rpm
                    .maybe_last_good()
                    .is_some_and(|sample| *sample.value() > 0)
            {
                return Err(MiningActuationAdapterError::SafetyObservationsUnavailable);
            }
        }
        Ok(())
    }

    fn cancellable_delay(&self, duration_ms: u64) -> Result<(), MiningActuationAdapterError> {
        crate::mining_actuation::wait_with_cancellation(
            duration_ms,
            crate::runtime_uptime::millis,
            |milliseconds| thread::sleep(Duration::from_millis(milliseconds)),
            || self.check_preparation_admission(),
        )
    }

    fn request_preparation_voltage(
        &self,
        voltage: Ultra205CoreVoltage,
    ) -> Result<(), MiningActuationAdapterError> {
        self.check_preparation_admission()?;
        let pending = match crate::safety_adapter::queue_safety_actuation(
            SafetyActuationCommand::SetCoreVoltageForGeneration {
                voltage,
                permit: crate::production_mining_session::revocation::stamp(
                    self.maybe_worker_generation,
                ),
            },
        ) {
            SafetyActuationQueueOutcome::Queued(pending) => pending,
            SafetyActuationQueueOutcome::QueueFull => {
                return Err(MiningActuationAdapterError::SafetyQueueFull)
            }
            SafetyActuationQueueOutcome::OwnerUnavailable => {
                return Err(MiningActuationAdapterError::SafetyOwnerUnavailable)
            }
        };
        let deadline = crate::runtime_uptime::millis().saturating_add(3_000);
        loop {
            self.check_preparation_admission()?;
            match pending.poll() {
                SafetyActuationPollOutcome::Applied => return self.check_preparation_admission(),
                SafetyActuationPollOutcome::OwnerUnavailable => {
                    return Err(MiningActuationAdapterError::SafetyOwnerUnavailable)
                }
                SafetyActuationPollOutcome::HardwareWriteFailed => {
                    return Err(MiningActuationAdapterError::SafetyHardwareWriteFailed)
                }
                SafetyActuationPollOutcome::Pending => {}
            }
            if crate::runtime_uptime::millis() >= deadline {
                return Err(MiningActuationAdapterError::SafetyReplyTimedOut);
            }
            self.cancellable_delay(50)?;
        }
    }

    pub fn safe_stop(
        &mut self,
        purpose: HardwareSafeStopPurpose,
        progress: &mut dyn FnMut(SafeShutdownStep),
    ) -> Result<(), SafeShutdownFailure<MiningActuationAdapterError>> {
        execute_safe_stop_with_progress(self, purpose, progress)
    }

    pub fn set_self_test_fan_duty(percent: u8) -> Result<(), MiningActuationAdapterError> {
        let duty = FanDutyPercent::try_from(percent)
            .map_err(|_| MiningActuationAdapterError::UnsupportedProfile)?;
        Self::request_safety(SafetyActuationCommand::SetFanDuty(duty))
    }

    fn request_safety(command: SafetyActuationCommand) -> Result<(), MiningActuationAdapterError> {
        match crate::safety_adapter::request_safety_actuation(command) {
            SafetyActuationRequestOutcome::Applied => Ok(()),
            SafetyActuationRequestOutcome::QueueFull => {
                Err(MiningActuationAdapterError::SafetyQueueFull)
            }
            SafetyActuationRequestOutcome::OwnerUnavailable => {
                Err(MiningActuationAdapterError::SafetyOwnerUnavailable)
            }
            SafetyActuationRequestOutcome::ReplyTimedOut => {
                Err(MiningActuationAdapterError::SafetyReplyTimedOut)
            }
            SafetyActuationRequestOutcome::HardwareWriteFailed => {
                Err(MiningActuationAdapterError::SafetyHardwareWriteFailed)
            }
        }
    }

    fn set_fan_full(&mut self) -> Result<(), MiningActuationAdapterError> {
        let observations = crate::safety_adapter::observation_snapshot();
        self.maybe_fan_command_baseline =
            observations
                .fan_rpm
                .maybe_last_good()
                .map(|sample| ObservationStamp {
                    boot_session: sample.boot_session().get(),
                    sequence: sample.sequence().get(),
                    acquired_at_ms: sample.acquired_at().get(),
                });
        self.maybe_fan_command_started_at_ms = Some(crate::runtime_uptime::millis());
        match crate::safety_adapter::queue_safety_actuation(SafetyActuationCommand::SetFanDuty(
            FanDutyPercent::FULL,
        )) {
            SafetyActuationQueueOutcome::Queued(pending) => {
                self.maybe_pending_fan_actuation = Some(pending);
                Ok(())
            }
            SafetyActuationQueueOutcome::QueueFull => {
                Err(MiningActuationAdapterError::SafetyQueueFull)
            }
            SafetyActuationQueueOutcome::OwnerUnavailable => {
                Err(MiningActuationAdapterError::SafetyOwnerUnavailable)
            }
        }
    }

    fn wait_for_post_command_fan_proof(&mut self) -> Result<(), MiningActuationAdapterError> {
        let deadline_ms = crate::runtime_uptime::millis().saturating_add(FAN_PROOF_TIMEOUT_MS);
        let mut actuation_applied = false;
        loop {
            self.check_preparation_admission()?;
            if !actuation_applied {
                let Some(pending) = self.maybe_pending_fan_actuation.as_ref() else {
                    return Err(MiningActuationAdapterError::SafetyOwnerUnavailable);
                };
                match pending.poll() {
                    SafetyActuationPollOutcome::Pending => {}
                    SafetyActuationPollOutcome::Applied => {
                        actuation_applied = true;
                        self.maybe_pending_fan_actuation = None;
                    }
                    SafetyActuationPollOutcome::OwnerUnavailable => {
                        self.maybe_pending_fan_actuation = None;
                        return Err(MiningActuationAdapterError::SafetyOwnerUnavailable);
                    }
                    SafetyActuationPollOutcome::HardwareWriteFailed => {
                        self.maybe_pending_fan_actuation = None;
                        return Err(MiningActuationAdapterError::SafetyHardwareWriteFailed);
                    }
                }
            }
            let observations = crate::safety_adapter::observation_snapshot();
            let maybe_sample = observations.fan_rpm.maybe_last_good();
            if actuation_applied
                && observations.is_ultra_205_mining_safe_at(MonotonicMillis::new(
                    crate::runtime_uptime::millis(),
                ))
                && observations.fan_rpm.is_fresh()
                && maybe_sample.is_some_and(|sample| {
                    *sample.value() > 0
                        && self.fan_sample_is_post_command(ObservationStamp {
                            boot_session: sample.boot_session().get(),
                            sequence: sample.sequence().get(),
                            acquired_at_ms: sample.acquired_at().get(),
                        })
                })
            {
                self.fan_proof_confirmed = true;
                if let Some(generation) = self.maybe_worker_generation {
                    crate::production_mining_session::revocation::note_fan_proof(
                        generation,
                        crate::runtime_uptime::millis(),
                    );
                }
                return Ok(());
            }
            if crate::runtime_uptime::millis() >= deadline_ms {
                if !actuation_applied {
                    self.maybe_pending_fan_actuation = None;
                    return Err(MiningActuationAdapterError::SafetyReplyTimedOut);
                }
                return Err(MiningActuationAdapterError::FanRpmProofTimedOut);
            }
            thread::sleep(Duration::from_millis(FAN_PROOF_POLL_MS));
        }
    }

    fn fan_sample_is_post_command(&self, candidate: ObservationStamp) -> bool {
        let Some(started_at_ms) = self.maybe_fan_command_started_at_ms else {
            return false;
        };
        if candidate.acquired_at_ms <= started_at_ms {
            return false;
        }
        let Some(baseline) = self.maybe_fan_command_baseline else {
            return true;
        };
        candidate.boot_session != baseline.boot_session
            || candidate.sequence > baseline.sequence
            || candidate.acquired_at_ms > baseline.acquired_at_ms
    }

    fn wait_for_cooling_proof(&mut self) -> Result<(), MiningActuationAdapterError> {
        self.wait_for_cooling_proof_with_progress(&mut |_| {})
    }

    fn reduce_frequency_and_reset_nonce(
        &mut self,
        progress: &mut dyn FnMut(),
    ) -> Result<(), MiningActuationAdapterError> {
        let profile = self
            .maybe_asic_profile
            .unwrap_or(Bm1366MiningProfile::Conservative);
        let config = MiningReadyConfig::ultra_205_profile(1, profile);
        let actions = safe_shutdown_command_actions(config)
            .map_err(|_| MiningActuationAdapterError::AsicPlanInvalid)?;
        crate::asic_adapter::production::execute_safe_shutdown_actions_with_progress(
            &actions, progress,
        )
        .map_err(MiningActuationAdapterError::Asic)
    }

    fn wait_for_cooling_proof_with_progress(
        &mut self,
        progress: &mut dyn FnMut(SafeShutdownStep),
    ) -> Result<(), MiningActuationAdapterError> {
        if !self.cooling_fan_full_applied {
            return Err(MiningActuationAdapterError::CoolingProofRequired);
        }
        let started_at_ms = self
            .maybe_cooling_started_at_ms
            .ok_or(MiningActuationAdapterError::CoolingProofRequired)?;
        let deadline_ms = crate::runtime_uptime::millis().saturating_add(COOLING_PROOF_TIMEOUT_MS);
        loop {
            let observations = crate::safety_adapter::observation_snapshot();
            let maybe_temperature = observations.chip_temp_celsius.maybe_last_good();
            if observations
                .is_ultra_205_mining_safe_at(MonotonicMillis::new(crate::runtime_uptime::millis()))
                && observations.chip_temp_celsius.is_fresh()
                && maybe_temperature.is_some_and(|sample| {
                    sample.acquired_at().get() > started_at_ms
                        && sample.value().is_finite()
                        && *sample.value() <= SAFE_COOLING_THRESHOLD_C
                })
            {
                self.cooling_proven = true;
                return Ok(());
            }
            if crate::runtime_uptime::millis() >= deadline_ms {
                return Err(MiningActuationAdapterError::CoolingProofTimedOut);
            }
            progress(SafeShutdownStep::WaitForFreshTemperatureAtOrBelow45C);
            thread::sleep(Duration::from_millis(COOLING_PROOF_POLL_MS));
        }
    }

    fn asic_profile(
        frequency: AsicFrequencyMhz,
    ) -> Result<Bm1366MiningProfile, MiningActuationAdapterError> {
        match frequency.mhz() {
            400 => Ok(Bm1366MiningProfile::Conservative),
            485 => Ok(Bm1366MiningProfile::UpstreamDefault),
            _ => Err(MiningActuationAdapterError::UnsupportedProfile),
        }
    }

    fn core_voltage(
        voltage: CoreVoltageMv,
    ) -> Result<Ultra205CoreVoltage, MiningActuationAdapterError> {
        match voltage.millivolts() {
            1_100 => Ok(Ultra205CoreVoltage::Conservative1100Millivolts),
            1_200 => Ok(Ultra205CoreVoltage::UpstreamDefault1200Millivolts),
            _ => Err(MiningActuationAdapterError::UnsupportedProfile),
        }
    }

    fn preflight() -> Bm1366Preflight {
        Bm1366Preflight::chip_detect(
            BoardPreflightEvidence::active_ultra_205(),
            ConfigPreflightEvidence::ultra_205_defaults(),
        )
    }

    fn requested_profile_is_closed(&self) -> bool {
        self.maybe_requested_profile
            .is_some_and(MiningHardwareProfile::is_closed_ultra_205_production_profile)
    }
}

impl Default for Ultra205MiningActuationAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl MiningActuationBackend for Ultra205MiningActuationAdapter {
    type Error = MiningActuationAdapterError;

    fn check_preparation_admission(&mut self) -> Result<(), Self::Error> {
        Ultra205MiningActuationAdapter::check_preparation_admission(self)
    }

    fn execute_preparation_step(&mut self, step: PreparationStep) -> Result<(), Self::Error> {
        self.check_preparation_admission()?;
        log_preparation_progress(step, "started");
        let result = (|| match step {
            PreparationStep::RequireFreshSafetyObservations => {
                if !self.requested_profile_is_closed() {
                    return Err(MiningActuationAdapterError::UnsupportedProfile);
                }
                let observations = crate::safety_adapter::observation_snapshot();
                if !crate::safety_adapter::safety_actuation_available()
                    || !observations.is_ultra_205_mining_safe_at(MonotonicMillis::new(
                        crate::runtime_uptime::millis(),
                    ))
                {
                    return Err(MiningActuationAdapterError::SafetyObservationsUnavailable);
                }
                Ok(())
            }
            PreparationStep::SetFanDutyTo100Percent => self.set_fan_full(),
            PreparationStep::RequireFreshNonzeroFanRpm => self.wait_for_post_command_fan_proof(),
            PreparationStep::SetCoreVoltage(voltage) => {
                self.request_preparation_voltage(Self::core_voltage(voltage)?)
            }
            PreparationStep::WaitForCoreVoltageStabilization500Ms => {
                self.cancellable_delay(u64::from(CORE_VOLTAGE_STABILIZATION_MS))
            }
            PreparationStep::EnableAsic => {
                crate::asic_adapter::production::set_asic_power_enabled_guarded(
                    true,
                    self.maybe_worker_generation,
                )
                .map_err(MiningActuationAdapterError::Asic)
            }
            PreparationStep::ResetAndDetectExactlyOneChip => {
                let decision = Bm1366InitPlan::chip_detect_with_options(
                    Self::preflight(),
                    ChipDetectPlanOptions {
                        skip_reset_pulse: false,
                        version_mask_prelude_count: 3,
                        wait_tx_done_after_chip_id_write: true,
                    },
                );
                if decision.maybe_fail_closed_action().is_some() {
                    return Err(MiningActuationAdapterError::AsicPlanInvalid);
                }
                crate::asic_adapter::production::execute_chip_detection_actions_guarded(
                    decision.actions(),
                    self.maybe_worker_generation,
                )
                .map_err(MiningActuationAdapterError::Asic)
            }
            PreparationStep::InitializeMiningReadyWithFrequencyRamp(frequency) => {
                let profile = Self::asic_profile(frequency)?;
                self.maybe_asic_profile = Some(profile);
                let decision = Bm1366InitPlan::mining_ready_init_for_profile(
                    Self::preflight(),
                    1,
                    profile,
                    MiningReadyInitOptions::production_with_frequency_ramp(),
                );
                if decision.maybe_fail_closed_action().is_some() {
                    return Err(MiningActuationAdapterError::AsicPlanInvalid);
                }
                crate::asic_adapter::production::execute_mining_ready_actions_guarded(
                    decision.actions(),
                    self.maybe_worker_generation,
                )
                .map_err(MiningActuationAdapterError::Asic)
            }
            PreparationStep::RetainProductionUart => {
                if crate::asic_adapter::production::production_handle_available()
                    && crate::asic_adapter::production::production_ready()
                {
                    Ok(())
                } else {
                    Err(MiningActuationAdapterError::Asic(
                        ProductionAsicBlocker::AsicInitFailed,
                    ))
                }
            }
        })();
        let result = result.and_then(|()| self.check_preparation_admission());
        log_preparation_progress(
            step,
            if result.is_ok() {
                "completed"
            } else {
                "failed"
            },
        );
        result
    }

    fn execute_safe_shutdown_step(&mut self, step: SafeShutdownStep) -> Result<(), Self::Error> {
        self.note_shutdown_step(step);
        match step {
            SafeShutdownStep::StopDispatch => {
                crate::asic_adapter::production::block_production_dispatch()
                    .map_err(MiningActuationAdapterError::Asic)
            }
            SafeShutdownStep::ReduceFrequencyAndResetNonce => {
                self.reduce_frequency_and_reset_nonce(&mut || {})
            }
            SafeShutdownStep::HoldResetLow => {
                crate::asic_adapter::production::execute_safe_shutdown_actions(&[
                    Bm1366AdapterAction::HOLD_RESET_LOW,
                ])
                .map_err(MiningActuationAdapterError::Asic)
            }
            // Pinned upstream VCORE_set_voltage(0) performs no DS4432U write;
            // GPIO10 removes VCORE. Repeating the desired low state is safe.
            SafeShutdownStep::DisableCoreVoltage | SafeShutdownStep::DisableAsic => {
                crate::asic_adapter::production::set_asic_power_enabled(false)
                    .map_err(MiningActuationAdapterError::Asic)
            }
            SafeShutdownStep::SetFanDutyTo100Percent => {
                self.cooling_fan_full_applied = false;
                self.cooling_proven = false;
                self.maybe_cooling_started_at_ms = Some(crate::runtime_uptime::millis());
                Self::request_safety(SafetyActuationCommand::SetFanDuty(FanDutyPercent::FULL))?;
                self.cooling_fan_full_applied = true;
                Ok(())
            }
            SafeShutdownStep::WaitForFreshTemperatureAtOrBelow45C => self.wait_for_cooling_proof(),
            SafeShutdownStep::SetFanDutyTo30Percent => {
                if !self.cooling_proven {
                    return Err(MiningActuationAdapterError::CoolingProofRequired);
                }
                Self::request_safety(SafetyActuationCommand::SetFanDutyAfterCoolingProof)
            }
        }
    }

    fn execute_safe_shutdown_step_with_progress(
        &mut self,
        step: SafeShutdownStep,
        progress: &mut dyn FnMut(SafeShutdownStep),
    ) -> Result<(), Self::Error> {
        self.note_shutdown_step(step);
        if step == SafeShutdownStep::ReduceFrequencyAndResetNonce {
            let mut action_progress = || progress(step);
            return self.reduce_frequency_and_reset_nonce(&mut action_progress);
        }
        if step == SafeShutdownStep::WaitForFreshTemperatureAtOrBelow45C {
            return self.wait_for_cooling_proof_with_progress(progress);
        }
        self.execute_safe_shutdown_step(step)
    }
}

fn log_preparation_progress(step: PreparationStep, outcome: &'static str) {
    log::info!(
        "mining_campaign_preparation={{\"schema\":\"mining-campaign-preparation-v1\",\"step\":\"{}\",\"outcome\":\"{outcome}\"}}",
        step.label()
    );
}
