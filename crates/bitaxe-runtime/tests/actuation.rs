use bitaxe_runtime::clock::{wait_with_clock, Clock};
use bitaxe_runtime::mining_actuation::{
    execute_preparation, preparation_plan, safe_shutdown_plan, MiningActuationBackend,
    PreparationStep, SafeShutdownStep,
};
use bitaxe_runtime::revocation::{GenerationGate, RevocationReason, WorkerGeneration};
use bitaxe_stratum::v1::production_session::MiningHardwareProfile;

struct Adapter {
    gate: GenerationGate,
    generation: WorkerGeneration,
    now_ms: u64,
    cancel_at: PreparationStep,
    shutdown: Vec<SafeShutdownStep>,
    rollback_failed: bool,
}

impl Adapter {
    fn new(cancel_at: PreparationStep) -> Self {
        let gate = GenerationGate::new();
        let generation = gate.begin_link(0).expect("fresh generation");
        assert!(gate.admit_budget(generation, 180_000));
        assert!(gate.activate(generation));
        Self {
            gate,
            generation,
            now_ms: 0,
            cancel_at,
            shutdown: Vec::new(),
            rollback_failed: false,
        }
    }
}

impl Clock for Adapter {
    fn now_ms(&self) -> u64 {
        self.now_ms
    }
    fn sleep_ms(&mut self, duration_ms: u64) {
        self.now_ms += duration_ms;
    }
}

impl MiningActuationBackend for Adapter {
    type Error = &'static str;
    fn check_preparation_admission(&mut self) -> Result<(), Self::Error> {
        self.gate.check_deadline(self.now_ms);
        self.gate
            .permits(Some(self.generation))
            .then_some(())
            .ok_or("revoked")
    }
    fn execute_preparation_step(&mut self, step: PreparationStep) -> Result<(), Self::Error> {
        if step == self.cancel_at {
            self.gate.revoke_reason_at(
                self.generation,
                self.now_ms,
                RevocationReason::UnsafeObservation,
            );
        }
        Ok(()) // A late successful adapter reply cannot re-open the gate.
    }
    fn execute_safe_shutdown_step(&mut self, step: SafeShutdownStep) -> Result<(), Self::Error> {
        self.shutdown.push(step);
        if self.rollback_failed && step == SafeShutdownStep::HoldResetLow {
            Err("rollback")
        } else {
            Ok(())
        }
    }
}

fn profile() -> MiningHardwareProfile {
    MiningHardwareProfile::ultra_205_bm1366(400, 1_100, 100).expect("production profile")
}

#[test]
fn every_late_preparation_success_rolls_back_after_revocation() {
    // Arrange
    for step in preparation_plan(profile()) {
        let mut adapter = Adapter::new(step);
        // Act
        let failure = execute_preparation(&mut adapter, profile()).expect_err("revoked");
        // Assert
        assert_eq!(failure.original().step(), step);
        assert_eq!(*failure.original().source(), "revoked");
        assert_eq!(adapter.shutdown, safe_shutdown_plan());
        assert!(!adapter.gate.permits(Some(adapter.generation)));
    }
}

#[test]
fn rollback_failure_does_not_replace_actual_authority_rejection() {
    // Arrange
    let step = PreparationStep::WaitForCoreVoltageStabilization500Ms;
    let mut adapter = Adapter::new(step);
    adapter.rollback_failed = true;
    // Act
    let failure = execute_preparation(&mut adapter, profile()).expect_err("revoked");
    // Assert
    assert_eq!(failure.original().step(), step);
    assert_eq!(*failure.original().source(), "revoked");
    assert_eq!(
        failure
            .maybe_safe_shutdown_failure()
            .expect("secondary rollback")
            .step(),
        SafeShutdownStep::HoldResetLow
    );
    assert_eq!(
        adapter.shutdown.last(),
        Some(&SafeShutdownStep::SetFanDutyTo30Percent)
    );
}

#[test]
fn virtual_wait_obeys_real_heartbeat_deadline() {
    // Arrange
    let mut adapter = Adapter::new(PreparationStep::EnableAsic);
    let gate = GenerationGate::new();
    let generation = gate.begin_link(0).expect("link");
    assert!(gate.admit_budget(generation, 180_000));
    assert!(gate.activate(generation));
    // Act
    let result = wait_with_clock(&mut adapter, 3_000, |now| {
        gate.check_deadline(now);
        gate.permits(Some(generation))
            .then_some(())
            .ok_or("revoked")
    });
    // Assert
    assert_eq!(result, Err("revoked"));
    assert_eq!(adapter.now_ms, 2_800);
    assert_eq!(
        gate.timing(2_800).expect("timing").revocation_reason,
        RevocationReason::HeartbeatTimeout
    );
}
