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
    execute_preparation, execute_safe_stop, MiningActuationBackend, PreparationExecutionFailure,
    PreparationStep, SafeShutdownFailure, SafeShutdownStep, CORE_VOLTAGE_STABILIZATION_MS,
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
            Self::SafetyObservationsUnavailable
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
    maybe_requested_profile: Option<MiningHardwareProfile>,
    maybe_fan_command_baseline: Option<ObservationStamp>,
    maybe_fan_command_started_at_ms: Option<u64>,
    maybe_pending_fan_actuation: Option<PendingSafetyActuation>,
    maybe_cooling_started_at_ms: Option<u64>,
    maybe_asic_profile: Option<Bm1366MiningProfile>,
    cooling_fan_full_applied: bool,
    cooling_proven: bool,
}

impl Ultra205MiningActuationAdapter {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            maybe_requested_profile: None,
            maybe_fan_command_baseline: None,
            maybe_fan_command_started_at_ms: None,
            maybe_pending_fan_actuation: None,
            maybe_cooling_started_at_ms: None,
            maybe_asic_profile: None,
            cooling_fan_full_applied: false,
            cooling_proven: false,
        }
    }

    pub fn prepare(
        &mut self,
        profile: MiningHardwareProfile,
    ) -> Result<(), PreparationExecutionFailure<MiningActuationAdapterError>> {
        self.maybe_requested_profile = Some(profile);
        execute_preparation(self, profile)
    }

    pub fn safe_stop(
        &mut self,
        purpose: HardwareSafeStopPurpose,
    ) -> Result<(), SafeShutdownFailure<MiningActuationAdapterError>> {
        execute_safe_stop(self, purpose)
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

    fn execute_preparation_step(&mut self, step: PreparationStep) -> Result<(), Self::Error> {
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
            PreparationStep::SetCoreVoltage(voltage) => Self::request_safety(
                SafetyActuationCommand::SetCoreVoltage(Self::core_voltage(voltage)?),
            ),
            PreparationStep::WaitForCoreVoltageStabilization500Ms => {
                thread::sleep(Duration::from_millis(u64::from(
                    CORE_VOLTAGE_STABILIZATION_MS,
                )));
                Ok(())
            }
            PreparationStep::EnableAsic => {
                crate::asic_adapter::production::set_asic_power_enabled(true)
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
                crate::asic_adapter::production::execute_chip_detection_actions(decision.actions())
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
                crate::asic_adapter::production::execute_mining_ready_actions(decision.actions())
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
        match step {
            SafeShutdownStep::StopDispatch => {
                crate::asic_adapter::production::block_production_dispatch()
                    .map_err(MiningActuationAdapterError::Asic)
            }
            SafeShutdownStep::ReduceFrequencyAndResetNonce => {
                let profile = self
                    .maybe_asic_profile
                    .unwrap_or(Bm1366MiningProfile::Conservative);
                let config = MiningReadyConfig::ultra_205_profile(1, profile);
                let actions = safe_shutdown_command_actions(config)
                    .map_err(|_| MiningActuationAdapterError::AsicPlanInvalid)?;
                crate::asic_adapter::production::execute_safe_shutdown_actions(&actions)
                    .map_err(MiningActuationAdapterError::Asic)
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
                Self::request_safety(SafetyActuationCommand::SetFanDuty(FanDutyPercent::PAUSED))
            }
        }
    }
}

fn log_preparation_progress(step: PreparationStep, outcome: &'static str) {
    log::info!(
        "mining_campaign_preparation={{\"schema\":\"mining-campaign-preparation-v1\",\"step\":\"{}\",\"outcome\":\"{outcome}\"}}",
        step.label()
    );
}
