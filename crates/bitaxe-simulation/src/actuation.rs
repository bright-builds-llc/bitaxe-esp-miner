//! Physical-policy orchestration over independently modeled board facts.
use crate::{Failure, JournalEvent, SimulationError};
use bitaxe_api::TelemetryObservations;
use bitaxe_asic::bm1366::{
    chip_detect::validate_single_chip_detect_response,
    command::Bm1366AdapterAction,
    init_plan::{
        Bm1366InitPlan, Bm1366Preflight, BoardPreflightEvidence, ChipDetectPlanOptions,
        ConfigPreflightEvidence,
    },
    mining_ready::{
        safe_shutdown_command_actions, Bm1366MiningProfile, MiningReadyConfig,
        MiningReadyInitOptions,
    },
};
use bitaxe_runtime::{
    mining_actuation::{MiningActuationBackend, PreparationStep, SafeShutdownStep},
    revocation::{GenerationGate, RevocationReason, WorkerGeneration},
};
use bitaxe_safety::observation::{
    BootSessionId, MonotonicMillis, Observation, ObservationSequence, UnavailableReason,
};
use bitaxe_stratum::v1::production_session::MiningHardwareProfile;
use bitaxe_virtual_board::{ModelError, VirtualBoard};
use std::cell::RefCell;
use std::rc::Rc;

pub(crate) struct RuntimeBoardAdapter {
    pub board: Rc<RefCell<VirtualBoard>>,
    pub gate: GenerationGate,
    pub generation: WorkerGeneration,
    pub scenario: String,
    pub journal: Vec<JournalEvent>,
    pub maybe_earliest_failure: Option<Failure>,
    pub cleanup_failures: Vec<Failure>,
    fan_proven: bool,
    maybe_reply: Option<Vec<u8>>,
    pub maybe_cancel_preparation: Option<PreparationStep>,
    pub maybe_fail_shutdown: Option<SafeShutdownStep>,
}
impl RuntimeBoardAdapter {
    pub fn profile() -> MiningHardwareProfile {
        MiningHardwareProfile::ultra_205_bm1366(400, 1100, 100)
            .expect("constant production profile")
    }
    pub fn new(board: Rc<RefCell<VirtualBoard>>, scenario: &str) -> Result<Self, SimulationError> {
        if board.borrow().now_ms() < 100 {
            board.borrow_mut().advance_to(100)?;
        }
        let gate = GenerationGate::new();
        let generation = gate
            .begin_link(board.borrow().now_ms())
            .ok_or(SimulationError::Boundary("generation"))?;
        Ok(Self {
            board,
            gate,
            generation,
            scenario: scenario.into(),
            journal: Vec::new(),
            maybe_earliest_failure: None,
            cleanup_failures: Vec::new(),
            fan_proven: false,
            maybe_reply: None,
            maybe_cancel_preparation: None,
            maybe_fail_shutdown: None,
        })
    }
    pub fn now_ms(&self) -> u64 {
        self.board.borrow().now_ms()
    }
    fn event(&mut self, phase: &str, category: &str) {
        self.journal.push(JournalEvent {
            at_ms: self.now_ms(),
            phase: phase.into(),
            category: category.into(),
        });
    }
    pub fn observations(&self) -> Result<TelemetryObservations, &'static str> {
        let b = self.board.borrow();
        let s = b.peripherals.sample;
        if !b.peripherals.sample_valid || s.boot_ordinal != b.storage.boot_ordinal {
            return Ok(TelemetryObservations::unavailable(
                UnavailableReason::NotYetObserved,
            ));
        }
        fn stamped<T: Copy>(
            value: T,
            boot: u64,
            seq: u64,
            at: u64,
        ) -> Result<Observation<T>, &'static str> {
            Observation::record_success(
                value,
                BootSessionId::new(boot),
                ObservationSequence::new(seq),
                MonotonicMillis::new(at),
            )
            .map(|(o, _)| o)
            .map_err(|_| "invalid_sample")
        }
        let f = |value: f64| stamped(value, s.boot_ordinal, s.sequence, s.sampled_at_ms);
        Ok(TelemetryObservations {
            power_watts: f(f64::from(s.power_mw) / 1000.0)?,
            bus_voltage_volts: f(f64::from(s.input_mv) / 1000.0)?,
            current_amps: f(f64::from(s.input_ma) / 1000.0)?,
            core_voltage_actual_mv: f(f64::from(s.core_mv))?,
            chip_temp_celsius: f(f64::from(s.temperature_millicelsius) / 1000.0)?,
            vr_temp_celsius: f(f64::from(s.temperature_millicelsius) / 1000.0)?,
            fan_rpm: stamped(
                u16::try_from(s.fan_rpm).map_err(|_| "invalid_fan")?,
                s.boot_ordinal,
                s.sequence,
                s.sampled_at_ms,
            )?,
        })
    }
    pub fn advance(&mut self, duration: u64, heartbeat: bool) -> Result<(), &'static str> {
        let end = self
            .now_ms()
            .checked_add(duration)
            .ok_or("clock_overflow")?;
        while self.now_ms() < end {
            let next = self.now_ms().saturating_add(50).min(end);
            self.board
                .borrow_mut()
                .advance_to(next)
                .map_err(model_error)?;
            if heartbeat {
                self.gate.heartbeat(self.generation, next);
            }
            self.gate.check_deadline(next);
            let facts = self.observations()?;
            self.gate.check_safety(
                facts.is_ultra_205_mining_safe_at(MonotonicMillis::new(next)),
                self.board.borrow().peripherals.sample.fan_rpm > 0,
                next,
            );
        }
        Ok(())
    }
    fn admission(&self) -> Result<(), &'static str> {
        self.gate.check_deadline(self.now_ms());
        if !self.gate.permits(Some(self.generation)) {
            return Err("generation_revoked");
        }
        let facts = self.observations()?;
        if !facts.is_ultra_205_mining_safe_at(MonotonicMillis::new(self.now_ms()))
            || (self.fan_proven && self.board.borrow().peripherals.sample.fan_rpm == 0)
        {
            return Err("unsafe_observation");
        }
        Ok(())
    }
    fn delay(&mut self, duration: u64) -> Result<(), &'static str> {
        let end = self
            .now_ms()
            .checked_add(duration)
            .ok_or("clock_overflow")?;
        loop {
            self.admission()?;
            if self.now_ms() >= end {
                return Ok(());
            }
            self.advance(50.min(end - self.now_ms()), true)?;
        }
    }
    fn actions(
        &mut self,
        actions: &[Bm1366AdapterAction],
        guarded: bool,
    ) -> Result<(), &'static str> {
        for action in actions {
            if guarded {
                self.admission()?;
            }
            match action {
                Bm1366AdapterAction::UseDefaultBaud { baud }
                | Bm1366AdapterAction::UseMaxBaud { baud } => {
                    self.board.borrow_mut().asic.host_baud = *baud
                }
                Bm1366AdapterAction::ClearRx => self.maybe_reply = None,
                Bm1366AdapterAction::WriteFrame(frame) => {
                    self.maybe_reply = Some(
                        self.board
                            .borrow_mut()
                            .uart_exchange(frame.as_slice())
                            .map_err(model_error)?,
                    )
                }
                Bm1366AdapterAction::ReadChipId { expected_chips, .. } => {
                    let reply = self.maybe_reply.take().ok_or("missing_chip_reply")?;
                    validate_single_chip_detect_response(&reply, *expected_chips)
                        .map_err(|_| "invalid_chip_reply")?;
                }
                Bm1366AdapterAction::ReadExact { len, .. } => {
                    if self
                        .maybe_reply
                        .take()
                        .is_none_or(|reply| reply.len() != *len)
                    {
                        return Err("missing_exact_reply");
                    }
                }
                Bm1366AdapterAction::DelayMs(ms) => {
                    if guarded {
                        self.delay(u64::from(*ms))?;
                    } else {
                        self.advance(u64::from(*ms), false)?;
                    }
                }
                Bm1366AdapterAction::ResetPulse { low_ms, high_ms } => {
                    self.board.borrow_mut().peripherals.reset_asserted = true;
                    self.board.borrow_mut().asic.reset();
                    self.advance(u64::from(*low_ms), true)?;
                    self.board.borrow_mut().peripherals.reset_asserted = false;
                    self.advance(u64::from(*high_ms), true)?;
                }
                Bm1366AdapterAction::HoldResetLow => {
                    self.board.borrow_mut().peripherals.reset_asserted = true
                }
                Bm1366AdapterAction::WaitTxDone { .. } => self.advance(1, guarded)?,
                Bm1366AdapterAction::PublishStatus(_) => {}
            }
        }
        Ok(())
    }
    fn saturated_queue(&mut self) -> Result<(), &'static str> {
        use bitaxe_runtime::request_queue::{enqueue, EnqueueOutcome, ACTUATION_REQUEST_CAPACITY};
        let (sender, receiver) = std::sync::mpsc::sync_channel(ACTUATION_REQUEST_CAPACITY);
        for _ in 0..ACTUATION_REQUEST_CAPACITY {
            let (reply_sender, _reply_receiver) = bitaxe_runtime::reply::reply::<u8>();
            if enqueue(
                &sender,
                PreparationStep::SetFanDutyTo100Percent,
                reply_sender,
            ) != EnqueueOutcome::Queued
            {
                return Err("queue_fixture_failed");
            }
        }
        let (reply_sender, _reply_receiver) = bitaxe_runtime::reply::reply::<u8>();
        let blocked = enqueue(
            &sender,
            PreparationStep::SetFanDutyTo100Percent,
            reply_sender,
        ) == EnqueueOutcome::Full;
        drop(receiver);
        self.event(
            "safety_request_queue",
            if blocked {
                "queue_full"
            } else {
                "unexpected_queue_admission"
            },
        );
        if blocked {
            Err("safety_queue_full")
        } else {
            Ok(())
        }
    }

    fn delayed_i2c(&mut self) -> Result<(), &'static str> {
        use bitaxe_runtime::i2c_retry::{
            retry_runtime_transfer, RuntimeI2cBudget, RuntimeI2cTransferError,
        };
        let board = self.board.clone();
        let now = self.now_ms();
        board
            .borrow_mut()
            .scheduler
            .schedule(now, bitaxe_virtual_board::BoardEvent::I2cDelay(5_000))
            .map_err(model_error)?;
        board.borrow_mut().advance_to(now).map_err(model_error)?;
        let mut budget = RuntimeI2cBudget::new(now + 1000);
        let result = retry_runtime_transfer(
            &mut budget,
            || board.borrow().now_ms(),
            |timeout| {
                let result = board.borrow_mut().i2c_write(0x4c, 0x4c, &[63]);
                if result.is_err() {
                    let end = board.borrow().now_ms() + timeout;
                    board.borrow_mut().advance_to(end).map_err(model_error)?;
                }
                result.map_err(model_error)
            },
            |delay| {
                let end = board.borrow().now_ms() + u64::from(delay);
                // Clock advance cannot regress or overflow under this 1-second plan.
                board
                    .borrow_mut()
                    .advance_to(end)
                    .expect("bounded monotonic board delay");
            },
        );
        let category = match result {
            Ok(()) => "i2c_unexpected_success",
            Err(RuntimeI2cTransferError::BudgetExhausted) => "i2c_budget_exhausted",
            Err(RuntimeI2cTransferError::Driver(_)) => "i2c_driver_failed",
        };
        self.event("i2c_runtime_transfer", category);
        Err(category)
    }

    fn preflight() -> Bm1366Preflight {
        Bm1366Preflight::chip_detect(
            BoardPreflightEvidence::active_ultra_205(),
            ConfigPreflightEvidence::ultra_205_defaults(),
        )
    }
}
fn model_error(error: ModelError) -> &'static str {
    match error {
        ModelError::Unsupported(_) => "unsupported_model_operation",
        ModelError::Busy { .. } => "i2c_busy",
        ModelError::Persistence => "persistence_failed",
        _ => "model_rejected",
    }
}
impl MiningActuationBackend for RuntimeBoardAdapter {
    type Error = &'static str;
    fn check_preparation_admission(&mut self) -> Result<(), Self::Error> {
        self.admission()
    }
    fn observe_preparation_rejection(&mut self, step: PreparationStep, error: &Self::Error) {
        if self.maybe_earliest_failure.is_none() {
            self.maybe_earliest_failure = Some(Failure {
                phase: step.label().into(),
                category: (*error).into(),
            });
        }
        self.event(step.label(), error);
        if step == PreparationStep::WaitForCoreVoltageStabilization500Ms {
            let sample = self.board.borrow().peripherals.sample;
            let discriminator = if self.now_ms().saturating_sub(sample.sampled_at_ms)
                > u64::from(bitaxe_safety::power::POWER_SAMPLE_STALE_AFTER_MS)
            {
                "stale_safety_sample"
            } else if sample.fan_rpm == 0 {
                "zero_fan_after_proof"
            } else if self.observations().is_ok_and(|facts| {
                !facts.is_ultra_205_mining_safe_at(MonotonicMillis::new(self.now_ms()))
            }) {
                "unsafe_sensor_values"
            } else {
                "authority_revoked"
            };
            self.event("rejection_discriminator", discriminator);
        }
    }
    fn execute_preparation_step(&mut self, step: PreparationStep) -> Result<(), Self::Error> {
        self.event(step.label(), "started");
        if self.maybe_cancel_preparation == Some(step) {
            self.gate.revoke_reason_at(
                self.generation,
                self.now_ms(),
                RevocationReason::ControlFailed,
            );
            return Err("injected_preparation_cancellation");
        }
        match step {
            PreparationStep::RequireFreshSafetyObservations => self.admission()?,
            PreparationStep::SetFanDutyTo100Percent => {
                if self.scenario == "queue-saturation" {
                    self.saturated_queue()?;
                }
                if self.scenario == "delayed-i2c" {
                    self.delayed_i2c()?;
                }
                self.board.borrow_mut().peripherals.fan_percent = 100
            }
            PreparationStep::RequireFreshNonzeroFanRpm => {
                let before = self.board.borrow().peripherals.sample;
                self.delay(100)?;
                let after = self.board.borrow().peripherals.sample;
                if after.fan_rpm == 0
                    || !bitaxe_runtime::cooling::post_command_fan(
                        bitaxe_runtime::cooling::FanStamp {
                            boot_session: self.board.borrow().storage.boot_ordinal,
                            sequence: after.sequence,
                            acquired_at_ms: after.sampled_at_ms,
                        },
                        Some(bitaxe_runtime::cooling::FanStamp {
                            boot_session: self.board.borrow().storage.boot_ordinal,
                            sequence: before.sequence,
                            acquired_at_ms: before.sampled_at_ms,
                        }),
                        before.sampled_at_ms,
                    )
                {
                    return Err("fan_proof_failed");
                }
                self.fan_proven = true;
                self.gate.note_fan_proof(self.generation, self.now_ms());
            }
            PreparationStep::SetCoreVoltage(voltage) => {
                self.board.borrow_mut().peripherals.target_core_mv = u32::from(voltage.millivolts())
            }
            PreparationStep::WaitForCoreVoltageStabilization500Ms => {
                match self.scenario.as_str() {
                    "stale-safety-step5" => {
                        self.board
                            .borrow_mut()
                            .peripherals
                            .sensor_publication_blocked = true;
                        self.delay(1100)?;
                    }
                    "zero-fan-step5" => {
                        self.board.borrow_mut().peripherals.fan_stalled = true;
                    }
                    "unsafe-reading-step5" => {
                        self.board.borrow_mut().peripherals.sample.input_mv = 1000;
                        self.gate.check_safety(
                            self.observations()?
                                .is_ultra_205_mining_safe_at(MonotonicMillis::new(self.now_ms())),
                            self.board.borrow().peripherals.sample.fan_rpm > 0,
                            self.now_ms(),
                        );
                    }
                    _ => {}
                }
                self.delay(500)?;
            }
            PreparationStep::EnableAsic => self.board.borrow_mut().peripherals.power_enabled = true,
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
                    return Err("asic_plan_rejected");
                }
                self.actions(decision.actions(), true)?;
            }
            PreparationStep::InitializeMiningReadyWithFrequencyRamp(_) => {
                let decision = Bm1366InitPlan::mining_ready_init_for_profile(
                    Self::preflight(),
                    1,
                    Bm1366MiningProfile::Conservative,
                    MiningReadyInitOptions::production_with_frequency_ramp(),
                );
                if decision.maybe_fail_closed_action().is_some() {
                    return Err("asic_plan_rejected");
                }
                self.actions(decision.actions(), true)?;
            }
            PreparationStep::RetainProductionUart => {
                if self.board.borrow().asic.frequency_mhz != 400 {
                    return Err("asic_not_ready");
                }
            }
        }
        self.admission()?;
        self.event(step.label(), "completed");
        Ok(())
    }
    fn execute_safe_shutdown_step(&mut self, step: SafeShutdownStep) -> Result<(), Self::Error> {
        self.event(step.label(), "cleanup_started");
        if self.maybe_fail_shutdown == Some(step) {
            let category = "shutdown_step_cancelled";
            self.cleanup_failures.push(Failure {
                phase: step.label().into(),
                category: category.into(),
            });
            self.maybe_fail_shutdown = None;
            return Err(category);
        }
        let result = (|| {
            match step {
                SafeShutdownStep::StopDispatch => self.gate.block_work(),
                SafeShutdownStep::ReduceFrequencyAndResetNonce => {
                    if self.board.borrow().peripherals.power_enabled
                        && !self.board.borrow().peripherals.reset_asserted
                    {
                        let actions =
                            safe_shutdown_command_actions(MiningReadyConfig::ultra_205_profile(
                                1,
                                Bm1366MiningProfile::Conservative,
                            ))
                            .map_err(|_| "shutdown_plan")?;
                        self.actions(&actions, false)?;
                    }
                }
                SafeShutdownStep::HoldResetLow => {
                    self.board.borrow_mut().peripherals.reset_asserted = true
                }
                SafeShutdownStep::DisableCoreVoltage | SafeShutdownStep::DisableAsic => {
                    self.board.borrow_mut().peripherals.power_enabled = false
                }
                SafeShutdownStep::SetFanDutyTo100Percent => {
                    self.board.borrow_mut().peripherals.fan_percent = 100
                }
                SafeShutdownStep::WaitForFreshTemperatureAtOrBelow45C => {
                    // Sensor fault injection ends at cleanup; the model produces a new proof.
                    self.board
                        .borrow_mut()
                        .peripherals
                        .sensor_publication_blocked = false;
                    self.board.borrow_mut().peripherals.fan_stalled = false;
                    let before = self.now_ms();
                    self.advance(500, false)?;
                    if !bitaxe_runtime::cooling::baseline_safe(&self.observations()?, self.now_ms())
                        || self.board.borrow().peripherals.sample.sampled_at_ms <= before
                    {
                        return Err("cooling_proof_failed");
                    }
                }
                SafeShutdownStep::SetFanDutyTo30Percent => {
                    self.board.borrow_mut().peripherals.fan_percent = 30
                }
            }
            Ok(())
        })();
        if let Err(category) = result {
            self.cleanup_failures.push(Failure {
                phase: step.label().into(),
                category: category.into(),
            });
        }
        result
    }
}
