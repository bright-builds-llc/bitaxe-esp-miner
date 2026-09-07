#![allow(dead_code)]
#[path = "production_mining_session/cooling_core.rs"]
mod cooling_core;
use bitaxe_api::TelemetryObservations;
use bitaxe_safety::observation::Observation;
use bitaxe_safety::observation::{BootSessionId, MonotonicMillis, ObservationSequence};
use cooling_core::*;
fn observations(rpm: u16, sequence: u64, at: u64, temperature: f64) -> TelemetryObservations {
    fn sample<T>(v: T, sequence: u64, at: u64) -> Observation<T> {
        Observation::record_success(
            v,
            BootSessionId::new(7),
            ObservationSequence::new(sequence),
            MonotonicMillis::new(at),
        )
        .expect("synthetic observation")
        .0
    }
    TelemetryObservations {
        power_watts: sample(0.5, sequence, at),
        bus_voltage_volts: sample(5.0, sequence, at),
        current_amps: sample(0.1, sequence, at),
        core_voltage_actual_mv: sample(0.0, sequence, at),
        chip_temp_celsius: sample(temperature, sequence, at),
        vr_temp_celsius: sample(temperature, sequence, at),
        fan_rpm: sample(rpm, sequence, at),
    }
}
struct Backend {
    now: u64,
    rpm: u16,
    advance_sample: bool,
    cancel: bool,
    pending: bool,
    maybe_fixed_feedback_at: Option<u64>,
    acknowledge_at: u64,
    temperature: f64,
    commands: Vec<FanCommand>,
}
impl Default for Backend {
    fn default() -> Self {
        Self {
            now: 1_000,
            rpm: 2_000,
            advance_sample: true,
            cancel: false,
            pending: false,
            maybe_fixed_feedback_at: None,
            acknowledge_at: 1_000,
            temperature: 30.0,
            commands: vec![],
        }
    }
}
impl CoolingBackend for Backend {
    fn now_ms(&self) -> u64 {
        self.now
    }
    fn admitted(&self) -> bool {
        !self.cancel || self.now == 1_000
    }
    fn observations(&self) -> TelemetryObservations {
        if self.advance_sample {
            let acquired = if self.now > 1_000 {
                self.maybe_fixed_feedback_at.unwrap_or(self.now)
            } else {
                self.now
            };
            observations(
                if self.now == 1_000 { 0 } else { self.rpm },
                acquired,
                acquired,
                self.temperature,
            )
        } else {
            observations(self.rpm, 1, 1_000, self.temperature)
        }
    }
    fn command(&mut self, c: FanCommand) -> Result<(), CoolingError> {
        self.commands.push(c);
        Ok(())
    }
    fn poll(&mut self) -> Result<FanReply, CoolingError> {
        Ok(if self.pending || self.now < self.acknowledge_at {
            FanReply::Pending
        } else {
            FanReply::Applied
        })
    }
    fn wait(&mut self) {
        self.now += 50;
    }
}
#[test]
fn never_prepared_worker_can_reach_fan_preparation_from_fresh_zero_rpm() {
    assert!(preparation_safety(true, false, true));
}
#[test]
fn ordinary_and_prepared_worker_still_require_nonzero_rpm() {
    assert!(!preparation_safety(true, false, false));
}
#[test]
fn stale_or_unsafe_observations_never_admit_preparation() {
    assert!(!preparation_safety(false, true, true));
}
#[test]
fn fan_only_success_commands_only_full_fan() {
    let mut backend = Backend::default();
    assert_eq!(qualify(&mut backend), Ok(2_000));
    assert_eq!(backend.commands, vec![FanCommand::Full]);
}
#[test]
fn zero_feedback_never_completes_fan_proof() {
    let mut backend = Backend {
        rpm: 0,
        ..Backend::default()
    };
    assert_eq!(qualify(&mut backend), Err(CoolingError::TimedOut));
    assert_eq!(backend.commands, vec![FanCommand::Full]);
}
#[test]
fn precommand_feedback_never_completes_fan_proof() {
    let mut backend = Backend {
        advance_sample: false,
        ..Backend::default()
    };
    assert!(qualify(&mut backend).is_err());
    assert_eq!(backend.commands, vec![FanCommand::Full]);
}
#[test]
fn cancellation_cannot_produce_fan_proof() {
    let mut backend = Backend {
        cancel: true,
        ..Backend::default()
    };
    assert_eq!(qualify(&mut backend), Err(CoolingError::Rejected));
    assert_eq!(backend.commands, vec![FanCommand::Full]);
}
#[test]
fn hot_baseline_cannot_lower_fan() {
    let mut backend = Backend {
        temperature: 46.0,
        ..Backend::default()
    };
    assert_eq!(restore(&mut backend), Err(CoolingError::Rejected));
    assert!(backend.commands.is_empty());
}
#[test]
fn cool_idle_baseline_can_lower_fan() {
    let mut backend = Backend::default();
    assert_eq!(restore(&mut backend), Ok(()));
    assert_eq!(backend.commands, vec![FanCommand::RestoreBaseline]);
}
#[test]
fn new_boot_or_nonadvancing_sequence_is_not_postcommand_proof() {
    let baseline = FanStamp {
        boot_session: 7,
        sequence: 3,
        acquired_at_ms: 10,
    };
    assert!(!post_command_fan(
        FanStamp {
            boot_session: 8,
            sequence: 4,
            acquired_at_ms: 30
        },
        Some(baseline),
        20
    ));
    assert!(!post_command_fan(
        FanStamp {
            boot_session: 7,
            sequence: 3,
            acquired_at_ms: 30
        },
        Some(baseline),
        20
    ));
}

#[path = "mining_actuation.rs"]
mod mining_actuation;
#[derive(Default)]
struct PreparationBackend {
    steps: Vec<mining_actuation::PreparationStep>,
    fail_fan_proof: bool,
}
impl mining_actuation::MiningActuationBackend for PreparationBackend {
    type Error = ();
    fn execute_preparation_step(
        &mut self,
        step: mining_actuation::PreparationStep,
    ) -> Result<(), ()> {
        self.steps.push(step);
        if self.fail_fan_proof
            && step == mining_actuation::PreparationStep::RequireFreshNonzeroFanRpm
        {
            Err(())
        } else {
            Ok(())
        }
    }
    fn execute_safe_shutdown_step(
        &mut self,
        _: mining_actuation::SafeShutdownStep,
    ) -> Result<(), ()> {
        Ok(())
    }
}
#[test]
fn fresh_zero_worker_reaches_fan_proof_and_cannot_energize_on_failed_proof() {
    // Arrange
    use bitaxe_stratum::v1::production_session::*;
    let profile = MiningHardwareProfilePreset::Conservative.profile();
    let lease = MiningCampaignLease::new(
        MiningCampaignLeaseId::new(1).expect("lease"),
        profile,
        MiningCampaignStopCondition::MonotonicDeadline {
            deadline: MiningCampaignMonotonicDeadline::new(60_000).expect("deadline"),
        },
    );
    let observations = observations(0, 1, 1_000, 30.0);
    let ready = ProductionReadiness {
        operator_intent: bitaxe_stratum::v1::state::MiningOperatorIntent::Run,
        network_ready: true,
        stratum_v1_supported: true,
        safety_prerequisites_fresh: preparation_safety(
            observations.is_ultra_205_mining_safe_at(MonotonicMillis::new(1_000)),
            false,
            true,
        ),
        maybe_campaign_lease: Some(lease),
        actuation_qualified: true,
    };
    let mut engine = ProductionMiningSession::new();
    let mut backend = PreparationBackend {
        fail_fan_proof: true,
        ..PreparationBackend::default()
    };
    // Act
    let effects = engine
        .handle(ProductionSessionEvent::Wake {
            wakeup: None,
            readiness: ready,
            now_ms: 1_000,
        })
        .expect("readiness");
    assert!(effects
        .iter()
        .any(|effect| matches!(effect, ProductionSessionEffect::PrepareHardware { .. })));
    let result = mining_actuation::execute_preparation(&mut backend, profile);
    // Assert
    assert!(result.is_err());
    assert_eq!(
        backend.steps,
        vec![
            mining_actuation::PreparationStep::RequireFreshSafetyObservations,
            mining_actuation::PreparationStep::SetFanDutyTo100Percent,
            mining_actuation::PreparationStep::RequireFreshNonzeroFanRpm
        ]
    );
}
#[test]
fn successful_fan_proof_precedes_any_preparation_power_effect() {
    // Arrange
    let mut backend = PreparationBackend::default();
    // Act
    mining_actuation::execute_preparation(
        &mut backend,
        bitaxe_stratum::v1::production_session::MiningHardwareProfilePreset::Conservative.profile(),
    )
    .expect("successful ordered preparation");
    // Assert
    assert!(matches!(
        &backend.steps[..4],
        [
            mining_actuation::PreparationStep::RequireFreshSafetyObservations,
            mining_actuation::PreparationStep::SetFanDutyTo100Percent,
            mining_actuation::PreparationStep::RequireFreshNonzeroFanRpm,
            mining_actuation::PreparationStep::SetCoreVoltage(_)
        ]
    ));
}

#[test]
fn fresh_nonzero_feedback_without_command_acknowledgement_is_not_proof() {
    // Arrange
    let mut backend = Backend {
        pending: true,
        ..Backend::default()
    };
    // Act
    let result = qualify(&mut backend);
    // Assert
    assert_eq!(result, Err(CoolingError::TimedOut));
    assert_eq!(backend.commands, vec![FanCommand::Full]);
}

#[test]
fn feedback_before_full_fan_acknowledgement_is_not_postcommand_proof() {
    // Arrange: feedback advances after enqueue but predates the command acknowledgement.
    let mut backend = Backend {
        maybe_fixed_feedback_at: Some(1_100),
        acknowledge_at: 1_500,
        ..Backend::default()
    };
    // Act
    let result = qualify(&mut backend);
    // Assert
    assert!(result.is_err());
    assert_eq!(backend.commands, vec![FanCommand::Full]);
}
