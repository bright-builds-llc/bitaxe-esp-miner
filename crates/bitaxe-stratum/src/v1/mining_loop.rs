//! Fail-closed Stratum v1 mining-loop gate.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/tasks/protocol_coordinator.c`
//! - `reference/esp-miner/main/work_queue.c`
//! - `crates/bitaxe-asic/src/bm1366/init_plan.rs`
//! - `crates/bitaxe-asic/src/bm1366/observation.rs`
//! - Parity checklist rows `STR-006`, `STR-007`, and `STAT-004`

use bitaxe_asic::bm1366::{
    command::Bm1366Command, result::Bm1366NonceResult, work::Bm1366WorkFields,
};
use bitaxe_config::Ultra205Defaults;

use crate::error::StratumV1Error;
use crate::v1::mining::ShareSubmission;
use crate::v1::queue::MiningWorkQueue;
use crate::v1::state::{
    MiningActivityStatus, MiningRuntimeState, PoolLifecycleStatus, WorkSubmissionGate,
};

pub const HARDWARE_EVIDENCE_ACK_MISSING: &str = "hardware_evidence_ack_missing";
pub const ASIC_INITIALIZED_GATE_MISSING: &str = "asic_initialized_gate_missing";
pub const SAFETY_PREFLIGHT_EVIDENCE_MISSING: &str = "safety_preflight_evidence_missing";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct MiningLoopGate {
    pub asic_initialized: bool,
    pub safety_evidence: bool,
    pub hardware_evidence_ack: bool,
}

impl MiningLoopGate {
    #[must_use]
    pub const fn decision(self) -> MiningLoopDecision {
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

        if !self.safety_evidence {
            return MiningLoopDecision::Blocked {
                reason: SAFETY_PREFLIGHT_EVIDENCE_MISSING,
            };
        }

        MiningLoopDecision::Ready
    }

    pub fn apply_to_state(self, state: &mut MiningRuntimeState) {
        match self.decision() {
            MiningLoopDecision::Ready => {
                state.allow_work_submission();
                state.set_lifecycle(PoolLifecycleStatus::Active);
                state.set_mining_activity(MiningActivityStatus::Active);
            }
            MiningLoopDecision::Blocked { .. } => {
                state.work_submission = WorkSubmissionGate::Blocked;
                state.set_lifecycle(PoolLifecycleStatus::Error);
                state.set_mining_activity(MiningActivityStatus::SafeBlocked);
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
    pub fields: Bm1366WorkFields,
    pub maybe_command: Option<Bm1366Command>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuardedMiningLoopInputs {
    pub gate: MiningLoopGate,
    pub pool_defaults: Ultra205Defaults,
    pub source: GuardedMiningLoopSource,
    pub work_queue: MiningWorkQueue,
    pub runtime_state: MiningRuntimeState,
    pub maybe_nonce_result: Option<Bm1366NonceResult>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GuardedMiningLoopPlan {
    pub source: GuardedMiningLoopSource,
    pub runtime_state: MiningRuntimeState,
    pub work_queue: MiningWorkQueue,
    pub maybe_dispatch: Option<GuardedBm1366DispatchPlan>,
    pub maybe_share_submission: Option<ShareSubmission>,
}

impl GuardedMiningLoopInputs {
    pub fn plan(mut self) -> Result<GuardedMiningLoopPlan, StratumV1Error> {
        self.gate.apply_to_state(&mut self.runtime_state);
        if self.gate.decision() != MiningLoopDecision::Ready {
            return Ok(GuardedMiningLoopPlan {
                source: self.source,
                runtime_state: self.runtime_state,
                work_queue: self.work_queue,
                maybe_dispatch: None,
                maybe_share_submission: None,
            });
        }

        self.runtime_state
            .set_pool_difficulty(crate::v1::messages::PoolDifficulty {
                difficulty: f64::from(self.pool_defaults.primary_pool().difficulty()),
            });

        let maybe_share_submission = self
            .maybe_nonce_result
            .and_then(|result| {
                self.work_queue
                    .maybe_active_work(result.job_id)
                    .map(|work| (work, result))
            })
            .map(|(work, result)| ShareSubmission::from_nonce_result(work, result))
            .transpose()?;

        if self.work_queue.is_empty() {
            return Ok(GuardedMiningLoopPlan {
                source: self.source,
                runtime_state: self.runtime_state,
                work_queue: self.work_queue,
                maybe_dispatch: None,
                maybe_share_submission,
            });
        }

        let work = self.work_queue.dequeue_work()?;
        if let Some(pool_difficulty) = work.maybe_pool_difficulty {
            self.runtime_state.set_pool_difficulty(pool_difficulty);
        }

        Ok(GuardedMiningLoopPlan {
            source: self.source,
            runtime_state: self.runtime_state,
            work_queue: self.work_queue,
            maybe_dispatch: Some(GuardedBm1366DispatchPlan {
                fields: work.fields,
                maybe_command: None,
            }),
            maybe_share_submission,
        })
    }
}

#[cfg(test)]
mod mining_loop_tests {
    use bitaxe_asic::bm1366::{
        command::Bm1366Command, result::Bm1366NonceResult, work::Bm1366JobId,
    };
    use bitaxe_config::ultra_205_defaults;

    use super::*;
    use crate::v1::messages::{ExtranonceAssignment, MiningNotify, PoolDifficulty};
    use crate::v1::mining::MiningWorkBuilder;
    use crate::v1::queue::MiningWorkQueue;
    use crate::v1::state::{MiningActivityStatus, MiningRuntimeState, WorkSubmissionGate};

    #[test]
    fn mining_loop_gate_defaults_to_hardware_evidence_block() {
        // Arrange
        let gate = MiningLoopGate::default();

        // Act
        let decision = gate.decision();

        // Assert
        assert_eq!(
            decision,
            MiningLoopDecision::Blocked {
                reason: HARDWARE_EVIDENCE_ACK_MISSING
            }
        );
    }

    #[test]
    fn mining_loop_gate_blocks_initialized_asic_without_safety_evidence() {
        // Arrange
        let gate = MiningLoopGate {
            asic_initialized: true,
            safety_evidence: false,
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
    fn mining_loop_gate_is_ready_when_all_evidence_is_present() {
        // Arrange
        let gate = MiningLoopGate {
            asic_initialized: true,
            safety_evidence: true,
            hardware_evidence_ack: true,
        };

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
    fn ready_mining_loop_turns_queued_work_into_typed_bm1366_dispatch() {
        // Arrange
        let mut queue = MiningWorkQueue::new();
        queue
            .enqueue_work(sample_work(Bm1366JobId::new(0x28)))
            .expect("sample work should enqueue");
        let inputs = GuardedMiningLoopInputs {
            gate: ready_gate(),
            pool_defaults: ultra_205_defaults(),
            source: GuardedMiningLoopSource::FakePool,
            work_queue: queue,
            runtime_state: MiningRuntimeState::default(),
            maybe_nonce_result: None,
        };

        // Act
        let plan = inputs.plan().expect("ready work should produce a plan");

        // Assert
        let dispatch = plan
            .maybe_dispatch
            .expect("ready queue should emit typed BM1366 work");
        assert_eq!(dispatch.fields.nbits, 0x1705_ae3a_u32.to_le_bytes());
        let maybe_command: Option<Bm1366Command> = dispatch.maybe_command;
        assert!(maybe_command.is_none());
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
    fn ready_mining_loop_converts_active_nonce_result_with_empty_pending_queue_into_share() {
        // Arrange
        let mut queue = MiningWorkQueue::new();
        let job_id = Bm1366JobId::new(0x28);
        queue
            .enqueue_work(sample_work(job_id))
            .expect("sample work should enqueue");
        let dispatch_plan = GuardedMiningLoopInputs {
            gate: ready_gate(),
            pool_defaults: ultra_205_defaults(),
            source: GuardedMiningLoopSource::Notify,
            work_queue: queue,
            runtime_state: MiningRuntimeState::default(),
            maybe_nonce_result: None,
        }
        .plan()
        .expect("queued work should dispatch first");
        let inputs = GuardedMiningLoopInputs {
            gate: ready_gate(),
            pool_defaults: ultra_205_defaults(),
            source: GuardedMiningLoopSource::Notify,
            work_queue: dispatch_plan.work_queue,
            runtime_state: MiningRuntimeState::default(),
            maybe_nonce_result: Some(sample_nonce_result(job_id)),
        };

        // Act
        let plan = inputs.plan().expect("valid result should produce a plan");

        // Assert
        assert!(plan.maybe_dispatch.is_none());
        let share = plan
            .maybe_share_submission
            .expect("valid tracked job should produce a share submission");
        assert_eq!(share.job_id, "job-40");
        assert_eq!(share.nonce, 0x1234_5678);
    }

    #[test]
    fn ready_mining_loop_converts_active_nonce_result_when_pending_front_job_differs() {
        // Arrange
        let active_job_id = Bm1366JobId::new(0x28);
        let pending_job_id = Bm1366JobId::new(0x30);
        let mut queue = MiningWorkQueue::new();
        queue
            .enqueue_work(sample_work(active_job_id))
            .expect("active sample work should enqueue");
        let mut queue = GuardedMiningLoopInputs {
            gate: ready_gate(),
            pool_defaults: ultra_205_defaults(),
            source: GuardedMiningLoopSource::Notify,
            work_queue: queue,
            runtime_state: MiningRuntimeState::default(),
            maybe_nonce_result: None,
        }
        .plan()
        .expect("active work should dispatch")
        .work_queue;
        queue
            .enqueue_work(sample_work(pending_job_id))
            .expect("pending sample work should enqueue");
        let inputs = GuardedMiningLoopInputs {
            gate: ready_gate(),
            pool_defaults: ultra_205_defaults(),
            source: GuardedMiningLoopSource::Notify,
            work_queue: queue,
            runtime_state: MiningRuntimeState::default(),
            maybe_nonce_result: Some(sample_nonce_result(active_job_id)),
        };

        // Act
        let plan = inputs.plan().expect("valid result should produce a plan");

        // Assert
        let share = plan
            .maybe_share_submission
            .expect("active work should produce a share submission");
        assert_eq!(share.job_id, "job-40");
        let dispatch = plan
            .maybe_dispatch
            .expect("pending work should still dispatch");
        assert_eq!(dispatch.fields.nbits, 0x1705_ae3a_u32.to_le_bytes());
    }

    #[test]
    fn ready_mining_loop_rejects_nonce_result_for_invalidated_job() {
        // Arrange
        let mut queue = MiningWorkQueue::new();
        let job_id = Bm1366JobId::new(0x28);
        queue
            .enqueue_work(sample_work(job_id))
            .expect("sample work should enqueue");
        queue.clear_jobs();
        let inputs = GuardedMiningLoopInputs {
            gate: ready_gate(),
            pool_defaults: ultra_205_defaults(),
            source: GuardedMiningLoopSource::Notify,
            work_queue: queue,
            runtime_state: MiningRuntimeState::default(),
            maybe_nonce_result: Some(sample_nonce_result(job_id)),
        };

        // Act
        let plan = inputs
            .plan()
            .expect("empty queue should still produce a plan");

        // Assert
        assert!(plan.maybe_dispatch.is_none());
        assert!(plan.maybe_share_submission.is_none());
    }

    fn ready_gate() -> MiningLoopGate {
        MiningLoopGate {
            asic_initialized: true,
            safety_evidence: true,
            hardware_evidence_ack: true,
        }
    }

    fn sample_work(job_id: Bm1366JobId) -> crate::v1::mining::MiningWork {
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
                clean_jobs: false,
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
