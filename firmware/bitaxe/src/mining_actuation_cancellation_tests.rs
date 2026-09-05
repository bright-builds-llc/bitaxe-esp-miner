use std::cell::Cell;

use crate::mining_actuation::{
    execute_preparation, wait_with_cancellation, MiningActuationBackend, PreparationStep,
    SafeShutdownStep,
};
use crate::revocation::{GenerationGate, WorkerGeneration};
use bitaxe_stratum::v1::production_session::MiningHardwareProfile;

struct Backend {
    gate: GenerationGate,
    generation: WorkerGeneration,
    now_ms: Cell<u64>,
    cancel_at_step: PreparationStep,
    late_success: bool,
    preparation: Vec<PreparationStep>,
    shutdown: Vec<(SafeShutdownStep, u64)>,
}

impl Backend {
    fn new(cancel_at_step: PreparationStep, late_success: bool) -> Self {
        let gate = GenerationGate::new();
        let generation = gate.begin_link(0).expect("generation");
        assert!(gate.admit_budget(generation, 180_000));
        assert!(gate.activate(generation));
        Self {
            gate,
            generation,
            now_ms: Cell::new(2_500),
            cancel_at_step,
            late_success,
            preparation: Vec::new(),
            shutdown: Vec::new(),
        }
    }
}

impl MiningActuationBackend for Backend {
    type Error = &'static str;

    fn check_preparation_admission(&mut self) -> Result<(), Self::Error> {
        self.gate.check_deadline(self.now_ms.get());
        self.gate
            .permits(Some(self.generation))
            .then_some(())
            .ok_or("revoked")
    }

    fn execute_preparation_step(&mut self, step: PreparationStep) -> Result<(), Self::Error> {
        self.preparation.push(step);
        if step != self.cancel_at_step {
            return Ok(());
        }
        if self.late_success {
            self.gate.revoke(self.generation);
            return Ok(());
        }
        wait_with_cancellation(
            3_000,
            || self.now_ms.get(),
            |slice| {
                assert!(slice <= 50);
                self.now_ms.set(self.now_ms.get() + slice);
            },
            || {
                self.gate.check_deadline(self.now_ms.get());
                self.gate
                    .permits(Some(self.generation))
                    .then_some(())
                    .ok_or("revoked")
            },
        )
    }

    fn execute_safe_shutdown_step(&mut self, step: SafeShutdownStep) -> Result<(), Self::Error> {
        self.shutdown.push((step, self.now_ms.get()));
        if step == SafeShutdownStep::WaitForFreshTemperatureAtOrBelow45C {
            self.now_ms.set(self.now_ms.get() + 120_000);
        }
        Ok(())
    }
}

fn profile() -> MiningHardwareProfile {
    MiningHardwareProfile::ultra_205_bm1366(400, 1_100, 100).expect("closed profile")
}

fn assert_cancelled_at(step: PreparationStep) {
    // Arrange
    let mut backend = Backend::new(step, false);
    // Act
    let outcome = execute_preparation(&mut backend, profile());
    // Assert
    assert!(outcome.is_err());
    assert_eq!(backend.preparation.last(), Some(&step));
    assert_eq!(
        backend.shutdown.first(),
        Some(&(SafeShutdownStep::StopDispatch, 2_800))
    );
    assert!(!backend.gate.permits(Some(backend.generation)));
    assert!(backend.now_ms.get() >= 120_000);
}

#[test]
fn heartbeat_loss_during_fan_proof_initiates_stop_before_cooling() {
    assert_cancelled_at(PreparationStep::RequireFreshNonzeroFanRpm);
}

#[test]
fn heartbeat_loss_during_voltage_stabilization_initiates_stop_before_cooling() {
    assert_cancelled_at(PreparationStep::WaitForCoreVoltageStabilization500Ms);
}

#[test]
fn heartbeat_loss_during_frequency_ramp_initiates_stop_before_cooling() {
    assert_cancelled_at(PreparationStep::InitializeMiningReadyWithFrequencyRamp(
        profile().frequency(),
    ));
}

#[test]
fn successful_late_preparation_cannot_publish_ready_after_revocation() {
    // Arrange
    let mut backend = Backend::new(PreparationStep::RetainProductionUart, true);
    // Act
    let outcome = execute_preparation(&mut backend, profile());
    // Assert
    assert!(outcome.is_err());
    assert_eq!(
        backend.shutdown.first(),
        Some(&(SafeShutdownStep::StopDispatch, 2_500))
    );
}
