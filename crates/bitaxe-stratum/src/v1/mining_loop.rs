//! Fail-closed Stratum v1 mining-loop gate.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/tasks/protocol_coordinator.c`
//! - `reference/esp-miner/main/work_queue.c`
//! - `crates/bitaxe-asic/src/bm1366/init_plan.rs`
//! - `crates/bitaxe-asic/src/bm1366/observation.rs`
//! - Parity checklist rows `STR-006`, `STR-007`, and `STAT-004`

use bitaxe_asic::bm1366::production::Bm1366ProductionCommand;
use bitaxe_config::Ultra205Defaults;
use bitaxe_safety::{
    evidence::SafetyCriticalEvidence, mining_preconditions::ProductionMiningPreconditionDecision,
    power::PowerEvidenceToken, status::SafetyStatus, thermal::ThermalEvidenceToken,
};

use crate::error::StratumV1Error;
use crate::v1::production_work::{
    CorrelationOutcome, ProductionNonceObservation, ProductionWorkRegistry, SubmitIntent,
};
use crate::v1::state::{MiningActivityStatus, MiningRuntimeState, PoolLifecycleStatus};

pub const HARDWARE_EVIDENCE_ACK_MISSING: &str = "hardware_evidence_ack_missing";
pub const ASIC_INITIALIZED_GATE_MISSING: &str = "asic_initialized_gate_missing";
pub const SAFETY_PREFLIGHT_EVIDENCE_MISSING: &str = "safety_preflight_evidence_missing";
pub const POWER_PREFLIGHT_EVIDENCE_MISSING: &str = "power_preflight_evidence_missing";
pub const THERMAL_PREFLIGHT_EVIDENCE_MISSING: &str = "thermal_preflight_evidence_missing";

#[derive(Debug, Clone, PartialEq)]
pub struct MiningLoopGate {
    pub production_preconditions: ProductionMiningPreconditionDecision,
    pub asic_initialized: bool,
    pub maybe_power_evidence: Option<PowerEvidenceToken>,
    pub maybe_thermal_evidence: Option<ThermalEvidenceToken>,
    pub maybe_safety_evidence: Option<SafetyCriticalEvidence>,
    pub safety_status: SafetyStatus,
    pub hardware_evidence_ack: bool,
}

impl Default for MiningLoopGate {
    fn default() -> Self {
        Self {
            production_preconditions: ProductionMiningPreconditionDecision::Ready,
            asic_initialized: false,
            maybe_power_evidence: None,
            maybe_thermal_evidence: None,
            maybe_safety_evidence: None,
            safety_status: SafetyStatus::SafeBlocked {
                reason: SAFETY_PREFLIGHT_EVIDENCE_MISSING,
            },
            hardware_evidence_ack: false,
        }
    }
}

impl MiningLoopGate {
    #[must_use]
    pub fn decision(&self) -> MiningLoopDecision {
        if let ProductionMiningPreconditionDecision::Blocked { reason, .. } =
            &self.production_preconditions
        {
            return MiningLoopDecision::Blocked { reason: *reason };
        }

        if self.maybe_power_evidence.is_none() {
            return MiningLoopDecision::Blocked {
                reason: POWER_PREFLIGHT_EVIDENCE_MISSING,
            };
        }

        if !self.thermal_evidence_ready() {
            return MiningLoopDecision::Blocked {
                reason: THERMAL_PREFLIGHT_EVIDENCE_MISSING,
            };
        }

        if !self.safety_evidence_ready() {
            return MiningLoopDecision::Blocked {
                reason: SAFETY_PREFLIGHT_EVIDENCE_MISSING,
            };
        }

        if !self.hardware_evidence_ack {
            return MiningLoopDecision::Blocked {
                reason: HARDWARE_EVIDENCE_ACK_MISSING,
            };
        }

        if !self.asic_initialized {
            return MiningLoopDecision::Blocked {
                reason: ASIC_INITIALIZED_GATE_MISSING,
            };
        }

        MiningLoopDecision::Ready
    }

    fn thermal_evidence_ready(&self) -> bool {
        let Some(thermal) = self.maybe_thermal_evidence else {
            return false;
        };

        thermal.evidence.is_hardware_verified()
    }

    fn safety_evidence_ready(&self) -> bool {
        let Some(evidence) = self.maybe_safety_evidence else {
            return false;
        };

        evidence.is_hardware_verified() && matches!(self.safety_status, SafetyStatus::Normal)
    }

    pub fn apply_to_state(&self, state: &mut MiningRuntimeState) {
        match self.decision() {
            MiningLoopDecision::Ready => {
                state.allow_work_submission();
                state.set_lifecycle(PoolLifecycleStatus::Active);
                state.set_mining_activity(MiningActivityStatus::Active);
            }
            MiningLoopDecision::Blocked { reason } => {
                state.block_work_submission(reason);
                state.set_lifecycle(PoolLifecycleStatus::Error);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiningLoopDecision {
    Ready,
    Blocked { reason: &'static str },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuardedMiningLoopSource {
    FakePool,
    Notify,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuardedBm1366DispatchPlan {
    pub maybe_production_command: Option<Bm1366ProductionCommand>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuardedMiningLoopInputs {
    pub gate: MiningLoopGate,
    pub pool_defaults: Ultra205Defaults,
    pub source: GuardedMiningLoopSource,
    pub production_registry: ProductionWorkRegistry,
    pub runtime_state: MiningRuntimeState,
    pub maybe_nonce_observation: Option<ProductionNonceObservation>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuardedMiningLoopPlan {
    pub source: GuardedMiningLoopSource,
    pub runtime_state: MiningRuntimeState,
    pub production_registry: ProductionWorkRegistry,
    pub maybe_dispatch: Option<GuardedBm1366DispatchPlan>,
    pub maybe_submit_intent: Option<SubmitIntent>,
}

impl GuardedMiningLoopInputs {
    pub fn plan(mut self) -> Result<GuardedMiningLoopPlan, StratumV1Error> {
        self.gate.apply_to_state(&mut self.runtime_state);
        if self.gate.decision() != MiningLoopDecision::Ready {
            return Ok(GuardedMiningLoopPlan {
                source: self.source,
                runtime_state: self.runtime_state,
                production_registry: self.production_registry,
                maybe_dispatch: None,
                maybe_submit_intent: None,
            });
        }

        self.runtime_state
            .set_pool_difficulty(crate::v1::messages::PoolDifficulty {
                difficulty: f64::from(self.pool_defaults.primary_pool().difficulty()),
            });

        let maybe_submit_intent = if let Some(observation) = self.maybe_nonce_observation {
            match self.production_registry.correlate_nonce_result(observation) {
                CorrelationOutcome::SubmitIntent(intent) => Some(intent),
                CorrelationOutcome::Blocked { reason } => {
                    self.runtime_state.block_work_submission(reason.as_str());
                    return Ok(GuardedMiningLoopPlan {
                        source: self.source,
                        runtime_state: self.runtime_state,
                        production_registry: self.production_registry,
                        maybe_dispatch: None,
                        maybe_submit_intent: None,
                    });
                }
            }
        } else {
            None
        };

        let dispatch = match self.production_registry.dispatch_next() {
            Ok(dispatch) => dispatch,
            Err(StratumV1Error::QueueEmpty) => {
                return Ok(GuardedMiningLoopPlan {
                    source: self.source,
                    runtime_state: self.runtime_state,
                    production_registry: self.production_registry,
                    maybe_dispatch: None,
                    maybe_submit_intent,
                });
            }
            Err(error) => return Err(error),
        };
        if let Some(pool_difficulty) = dispatch.work.maybe_pool_difficulty {
            self.runtime_state.set_pool_difficulty(pool_difficulty);
        }

        Ok(GuardedMiningLoopPlan {
            source: self.source,
            runtime_state: self.runtime_state,
            production_registry: self.production_registry,
            maybe_dispatch: Some(GuardedBm1366DispatchPlan {
                maybe_production_command: Some(Bm1366ProductionCommand::SendProductionWork(
                    dispatch.work_payload,
                )),
            }),
            maybe_submit_intent,
        })
    }
}

#[cfg(test)]
mod mining_loop_tests {
    use bitaxe_asic::bm1366::{
        production::Bm1366ProductionCommand, result::Bm1366NonceResult, work::Bm1366JobId,
    };
    use bitaxe_config::ultra_205_defaults;
    use bitaxe_safety::{
        evidence::SafetyCriticalEvidence,
        mining_preconditions::ProductionMiningPreconditionDecision, power::PowerEvidenceToken,
        status::SafetyStatus, thermal::ThermalEvidenceToken,
    };

    use super::*;
    use crate::v1::messages::{ExtranonceAssignment, MiningNotify, PoolDifficulty};
    use crate::v1::mining::MiningWorkBuilder;
    use crate::v1::production_work::{PoolSessionGeneration, ProductionWorkRegistry};
    use crate::v1::state::{MiningActivityStatus, MiningRuntimeState, WorkSubmissionGate};

    #[test]
    fn mining_loop_gate_missing_power_blocks_before_safety_evidence() {
        // Arrange
        let gate = MiningLoopGate {
            production_preconditions: ProductionMiningPreconditionDecision::Ready,
            ..MiningLoopGate::default()
        };

        // Act
        let decision = gate.decision();

        // Assert
        assert_eq!(
            decision,
            MiningLoopDecision::Blocked {
                reason: POWER_PREFLIGHT_EVIDENCE_MISSING
            }
        );
    }

    #[test]
    fn mining_loop_gate_missing_thermal_blocks_before_safety_evidence() {
        // Arrange
        let gate = MiningLoopGate {
            production_preconditions: ProductionMiningPreconditionDecision::Ready,
            asic_initialized: true,
            maybe_power_evidence: Some(power_evidence()),
            maybe_thermal_evidence: None,
            maybe_safety_evidence: Some(SafetyCriticalEvidence::hardware_smoke(
                "phase-06-mining-safety-smoke",
            )),
            safety_status: SafetyStatus::Normal,
            hardware_evidence_ack: true,
        };

        // Act
        let decision = gate.decision();

        // Assert
        assert_eq!(
            decision,
            MiningLoopDecision::Blocked {
                reason: THERMAL_PREFLIGHT_EVIDENCE_MISSING
            }
        );
    }

    #[test]
    fn mining_loop_gate_blocks_initialized_asic_without_safety_evidence() {
        // Arrange
        let gate = MiningLoopGate {
            production_preconditions: ProductionMiningPreconditionDecision::Ready,
            asic_initialized: true,
            maybe_power_evidence: Some(power_evidence()),
            maybe_thermal_evidence: Some(thermal_evidence()),
            maybe_safety_evidence: None,
            safety_status: SafetyStatus::Normal,
            hardware_evidence_ack: true,
        };

        // Act
        let decision = gate.decision();

        // Assert
        assert_eq!(
            decision,
            MiningLoopDecision::Blocked {
                reason: SAFETY_PREFLIGHT_EVIDENCE_MISSING
            }
        );
    }

    #[test]
    fn mining_loop_gate_blocks_faulted_safety_or_missing_hardware_ack() {
        // Arrange
        let faulted_safety = MiningLoopGate {
            production_preconditions: ProductionMiningPreconditionDecision::Ready,
            safety_status: SafetyStatus::PowerFault {
                reason: "power_fault",
            },
            ..ready_gate()
        };
        let missing_ack = MiningLoopGate {
            production_preconditions: ProductionMiningPreconditionDecision::Ready,
            hardware_evidence_ack: false,
            ..ready_gate()
        };

        // Act
        let faulted_decision = faulted_safety.decision();
        let missing_ack_decision = missing_ack.decision();

        // Assert
        assert_eq!(
            faulted_decision,
            MiningLoopDecision::Blocked {
                reason: SAFETY_PREFLIGHT_EVIDENCE_MISSING
            }
        );
        assert_eq!(
            missing_ack_decision,
            MiningLoopDecision::Blocked {
                reason: HARDWARE_EVIDENCE_ACK_MISSING
            }
        );
    }

    #[test]
    fn mining_loop_gate_is_ready_when_all_evidence_is_present() {
        // Arrange
        let gate = ready_gate();

        // Act
        let decision = gate.decision();

        // Assert
        assert_eq!(decision, MiningLoopDecision::Ready);
    }

    #[test]
    fn blocked_mining_loop_keeps_runtime_state_safe_blocked() {
        // Arrange
        let gate = MiningLoopGate::default();
        let mut state = MiningRuntimeState::default();

        // Act
        gate.apply_to_state(&mut state);

        // Assert
        assert_eq!(state.work_submission, WorkSubmissionGate::Blocked);
        assert_eq!(state.mining_activity, MiningActivityStatus::SafeBlocked);
    }

    #[test]
    fn blocked_production_precondition_stores_exact_reason_before_dispatch() {
        // Arrange
        let mut registry = ProductionWorkRegistry::new();
        registry
            .enqueue_pool_work(sample_work(Bm1366JobId::new(0x28)))
            .expect("sample work should enqueue");
        let gate = MiningLoopGate {
            production_preconditions: ProductionMiningPreconditionDecision::blocked(
                "voltage_observation_stale",
            ),
            ..ready_gate()
        };
        let inputs = GuardedMiningLoopInputs {
            gate,
            pool_defaults: ultra_205_defaults(),
            source: GuardedMiningLoopSource::Notify,
            production_registry: registry,
            runtime_state: MiningRuntimeState::default(),
            maybe_nonce_observation: Some(sample_nonce_observation(
                ProductionWorkRegistry::new().generation(),
                Bm1366JobId::new(0x28),
            )),
        };

        // Act
        let plan = inputs
            .plan()
            .expect("blocked precondition should still produce a safe plan");

        // Assert
        assert_eq!(
            plan.runtime_state.maybe_blocked_reason,
            Some("voltage_observation_stale")
        );
        assert_eq!(
            plan.runtime_state.work_submission,
            WorkSubmissionGate::Blocked
        );
        assert!(plan.maybe_dispatch.is_none());
        assert!(plan.maybe_submit_intent.is_none());
    }

    #[test]
    fn ready_mining_loop_dispatches_production_work_command() {
        // Arrange
        let mut registry = ProductionWorkRegistry::new();
        registry
            .enqueue_pool_work(sample_work(Bm1366JobId::new(0x28)))
            .expect("sample work should enqueue");
        let inputs = GuardedMiningLoopInputs {
            gate: ready_gate(),
            pool_defaults: ultra_205_defaults(),
            source: GuardedMiningLoopSource::FakePool,
            production_registry: registry,
            runtime_state: MiningRuntimeState::default(),
            maybe_nonce_observation: None,
        };

        // Act
        let plan = inputs.plan().expect("ready work should produce a plan");

        // Assert
        let dispatch = plan
            .maybe_dispatch
            .expect("ready queue should emit typed BM1366 work");
        let maybe_command: Option<Bm1366ProductionCommand> = dispatch.maybe_production_command;
        assert!(matches!(
            maybe_command,
            Some(Bm1366ProductionCommand::SendProductionWork(payload))
                if payload.job_id() == Bm1366JobId::new(0x28)
        ));
        assert_eq!(
            plan.runtime_state.work_submission,
            WorkSubmissionGate::Ready
        );
        assert_eq!(
            plan.runtime_state.mining_activity,
            MiningActivityStatus::Active
        );
    }

    #[test]
    fn ready_mining_loop_correlates_active_nonce_result_into_submit_intent() {
        // Arrange
        let mut registry = ProductionWorkRegistry::new();
        let job_id = Bm1366JobId::new(0x28);
        registry
            .enqueue_pool_work(sample_work(job_id))
            .expect("sample work should enqueue");
        let dispatch_plan = GuardedMiningLoopInputs {
            gate: ready_gate(),
            pool_defaults: ultra_205_defaults(),
            source: GuardedMiningLoopSource::Notify,
            production_registry: registry,
            runtime_state: MiningRuntimeState::default(),
            maybe_nonce_observation: None,
        }
        .plan()
        .expect("queued work should dispatch first");
        let observed_generation = dispatch_plan.production_registry.generation();
        let inputs = GuardedMiningLoopInputs {
            gate: ready_gate(),
            pool_defaults: ultra_205_defaults(),
            source: GuardedMiningLoopSource::Notify,
            production_registry: dispatch_plan.production_registry,
            runtime_state: MiningRuntimeState::default(),
            maybe_nonce_observation: Some(sample_nonce_observation(observed_generation, job_id)),
        };

        // Act
        let plan = inputs.plan().expect("valid result should produce a plan");

        // Assert
        assert!(plan.maybe_dispatch.is_none());
        let submit_intent = plan
            .maybe_submit_intent
            .expect("valid tracked job should produce a submit intent");
        assert_eq!(submit_intent.submission().job_id, "job-40");
        assert_eq!(submit_intent.submission().nonce, 0x1234_5678);
    }

    #[test]
    fn ready_mining_loop_correlates_active_result_and_dispatches_pending_work() {
        // Arrange
        let active_job_id = Bm1366JobId::new(0x28);
        let pending_job_id = Bm1366JobId::new(0x30);
        let mut registry = ProductionWorkRegistry::new();
        registry
            .enqueue_pool_work(sample_work(active_job_id))
            .expect("active sample work should enqueue");
        let mut registry = GuardedMiningLoopInputs {
            gate: ready_gate(),
            pool_defaults: ultra_205_defaults(),
            source: GuardedMiningLoopSource::Notify,
            production_registry: registry,
            runtime_state: MiningRuntimeState::default(),
            maybe_nonce_observation: None,
        }
        .plan()
        .expect("active work should dispatch")
        .production_registry;
        registry
            .enqueue_pool_work(sample_work(pending_job_id))
            .expect("pending sample work should enqueue");
        let observed_generation = registry.generation();
        let inputs = GuardedMiningLoopInputs {
            gate: ready_gate(),
            pool_defaults: ultra_205_defaults(),
            source: GuardedMiningLoopSource::Notify,
            production_registry: registry,
            runtime_state: MiningRuntimeState::default(),
            maybe_nonce_observation: Some(sample_nonce_observation(
                observed_generation,
                active_job_id,
            )),
        };

        // Act
        let plan = inputs.plan().expect("valid result should produce a plan");

        // Assert
        let submit_intent = plan
            .maybe_submit_intent
            .expect("active work should produce a submit intent");
        assert_eq!(submit_intent.submission().job_id, "job-40");
        let dispatch = plan
            .maybe_dispatch
            .expect("pending work should still dispatch");
        assert!(matches!(
            dispatch.maybe_production_command,
            Some(Bm1366ProductionCommand::SendProductionWork(payload))
                if payload.job_id() == pending_job_id
        ));
    }

    #[test]
    fn ready_mining_loop_blocks_uncorrelated_nonce_result_reason() {
        // Arrange
        let registry = ProductionWorkRegistry::new();
        let job_id = Bm1366JobId::new(0x28);
        let inputs = GuardedMiningLoopInputs {
            gate: ready_gate(),
            pool_defaults: ultra_205_defaults(),
            source: GuardedMiningLoopSource::Notify,
            production_registry: registry,
            runtime_state: MiningRuntimeState::default(),
            maybe_nonce_observation: Some(sample_nonce_observation(
                ProductionWorkRegistry::new().generation(),
                job_id,
            )),
        };

        // Act
        let plan = inputs
            .plan()
            .expect("empty queue should still produce a plan");

        // Assert
        assert!(plan.maybe_dispatch.is_none());
        assert!(plan.maybe_submit_intent.is_none());
        assert_eq!(
            plan.runtime_state.maybe_blocked_reason,
            Some("production_job_uncorrelated")
        );
        assert_eq!(
            plan.runtime_state.work_submission,
            WorkSubmissionGate::Blocked
        );
    }

    #[test]
    fn ready_mining_loop_rejects_stale_generation_after_clean_jobs_reuses_lookup() {
        // Arrange
        let job_id = Bm1366JobId::new(0x28);
        let mut registry = ProductionWorkRegistry::new();
        registry
            .enqueue_pool_work(sample_work(job_id))
            .expect("generation zero work should enqueue");
        let generation_zero_plan = GuardedMiningLoopInputs {
            gate: ready_gate(),
            pool_defaults: ultra_205_defaults(),
            source: GuardedMiningLoopSource::Notify,
            production_registry: registry,
            runtime_state: MiningRuntimeState::default(),
            maybe_nonce_observation: None,
        }
        .plan()
        .expect("generation zero work should dispatch");
        let stale_observation = sample_nonce_observation(
            generation_zero_plan.production_registry.generation(),
            job_id,
        );
        let mut registry = generation_zero_plan.production_registry;
        registry
            .enqueue_pool_work(sample_clean_jobs_work(job_id))
            .expect("clean-jobs work should replace active work");
        let generation_one_plan = GuardedMiningLoopInputs {
            gate: ready_gate(),
            pool_defaults: ultra_205_defaults(),
            source: GuardedMiningLoopSource::Notify,
            production_registry: registry,
            runtime_state: MiningRuntimeState::default(),
            maybe_nonce_observation: None,
        }
        .plan()
        .expect("generation one work should dispatch with reused lookup");
        let inputs = GuardedMiningLoopInputs {
            gate: ready_gate(),
            pool_defaults: ultra_205_defaults(),
            source: GuardedMiningLoopSource::Notify,
            production_registry: generation_one_plan.production_registry,
            runtime_state: MiningRuntimeState::default(),
            maybe_nonce_observation: Some(stale_observation),
        };

        // Act
        let plan = inputs
            .plan()
            .expect("stale generation should produce a fail-closed plan");

        // Assert
        assert!(plan.maybe_dispatch.is_none());
        assert!(plan.maybe_submit_intent.is_none());
        assert_eq!(
            plan.runtime_state.maybe_blocked_reason,
            Some("production_wrong_session")
        );
        assert_eq!(
            plan.runtime_state.work_submission,
            WorkSubmissionGate::Blocked
        );
    }

    fn ready_gate() -> MiningLoopGate {
        MiningLoopGate {
            production_preconditions: ProductionMiningPreconditionDecision::Ready,
            asic_initialized: true,
            maybe_power_evidence: Some(power_evidence()),
            maybe_thermal_evidence: Some(thermal_evidence()),
            maybe_safety_evidence: Some(SafetyCriticalEvidence::hardware_smoke(
                "phase-06-mining-safety-smoke",
            )),
            safety_status: SafetyStatus::Normal,
            hardware_evidence_ack: true,
        }
    }

    fn power_evidence() -> PowerEvidenceToken {
        PowerEvidenceToken {
            bus_voltage_volts: 5.0,
            current_amps: 2.5,
            power_watts: 12.5,
        }
    }

    fn thermal_evidence() -> ThermalEvidenceToken {
        ThermalEvidenceToken {
            chip_temp_celsius: 55.0,
            evidence: SafetyCriticalEvidence::hardware_smoke("phase-06-mining-thermal-smoke"),
        }
    }

    fn sample_work(job_id: Bm1366JobId) -> crate::v1::mining::MiningWork {
        sample_work_with_clean_jobs(job_id, false)
    }

    fn sample_clean_jobs_work(job_id: Bm1366JobId) -> crate::v1::mining::MiningWork {
        sample_work_with_clean_jobs(job_id, true)
    }

    fn sample_work_with_clean_jobs(
        job_id: Bm1366JobId,
        clean_jobs: bool,
    ) -> crate::v1::mining::MiningWork {
        MiningWorkBuilder::new(
            MiningNotify {
                job_id: format!("job-{}", job_id.raw()),
                prev_block_hash: "00".repeat(32),
                coinbase_1: "0200000001".to_owned(),
                coinbase_2: "ffffffff".to_owned(),
                merkle_branches: Vec::new(),
                version: 0x2000_0004,
                nbits: 0x1705_ae3a,
                ntime: 0x6470_25b5,
                clean_jobs,
            },
            ExtranonceAssignment {
                extranonce1: "4de05269".to_owned(),
                extranonce2_len: 4,
            },
        )
        .with_pool_difficulty(PoolDifficulty { difficulty: 1000.0 })
        .build(job_id)
        .expect("sample work should build")
    }

    fn sample_nonce_observation(
        observed_generation: PoolSessionGeneration,
        job_id: Bm1366JobId,
    ) -> ProductionNonceObservation {
        ProductionNonceObservation {
            observed_generation,
            result: sample_nonce_result(job_id),
        }
    }

    fn sample_nonce_result(job_id: Bm1366JobId) -> Bm1366NonceResult {
        Bm1366NonceResult {
            job_id,
            nonce: 0x1234_5678,
            asic_index: 0,
            core_id: 1,
            small_core_id: 0,
            version_bits: 0x0000_2000,
        }
    }
}
