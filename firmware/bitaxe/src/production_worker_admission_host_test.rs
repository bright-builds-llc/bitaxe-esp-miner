#![allow(dead_code)]

//! Runs the production BWG owner mapping with synthetic readiness and no hardware.

#[path = "production_mining_session/admission_diagnostics.rs"]
mod admission_diagnostics;
#[path = "production_mining_session/bwg.rs"]
mod bwg;
#[path = "production_mining_session/revocation.rs"]
mod revocation;
#[path = "production_mining_session/shutdown_budget.rs"]
mod shutdown_budget;

use bitaxe_stratum::v1::production_session::{
    MiningCampaignLease, MiningCampaignLeaseId, MiningCampaignMonotonicDeadline,
    MiningCampaignState, MiningCampaignStopCondition, MiningHardwareProfilePreset,
    MiningHardwareState, ProductionMiningSession, ProductionPoolSet, ProductionReadiness,
    ProductionSessionEvent, ProductionSessionSnapshot, ProductionSessionWakeup,
};
use bitaxe_stratum::v1::state::MiningOperatorIntent;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::sync::{Mutex, MutexGuard, OnceLock};
use std::time::Duration;

static NOTIFICATIONS: OnceLock<SyncSender<OwnerInboxMessage>> = OnceLock::new();
static FAN_CONTROLLER_ACTUATION_QUALIFIED: AtomicBool = AtomicBool::new(false);
static TEST_LOCK: Mutex<()> = Mutex::new(());

enum OwnerInboxMessage {
    Bwg(bwg::OwnerCommand),
}

fn semantic_version() -> &'static str {
    "synthetic-test"
}

mod runtime_uptime {
    pub(crate) fn millis() -> u64 {
        1_000
    }
}

mod worker_acceptance_budget {
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

    pub(crate) static FAIL_FINISH: AtomicBool = AtomicBool::new(false);
    pub(crate) static FINISH_CALLS: AtomicU32 = AtomicU32::new(0);

    pub(crate) fn diagnostic_snapshot() -> (u32, bool) {
        (0, false)
    }

    pub(crate) fn finish(_generation: super::revocation::WorkerGeneration) -> Result<(), ()> {
        // The actual durable ledger is outside this owner-admission test boundary.
        FINISH_CALLS.fetch_add(1, Ordering::SeqCst);
        if FAIL_FINISH.load(Ordering::SeqCst) {
            return Err(());
        }
        Ok(())
    }
}

struct OrdinaryEspProductionSessionAdapter {
    maybe_bwg_session: Option<bwg::OwnerSession>,
    maybe_bwg_reply: Option<bwg::PendingReply>,
    maybe_cooling_generation: Option<revocation::WorkerGeneration>,
    readiness: ProductionReadiness,
}

impl OrdinaryEspProductionSessionAdapter {
    fn wake_event(
        &mut self,
        wakeup: Option<ProductionSessionWakeup>,
        now_ms: u64,
        _snapshot: &ProductionSessionSnapshot,
        _pending_observation_recovered: bool,
    ) -> ProductionSessionEvent {
        ProductionSessionEvent::Wake {
            wakeup,
            now_ms,
            readiness: ProductionReadiness {
                maybe_campaign_lease: self.maybe_bwg_session.as_ref().map(|session| session.lease),
                ..self.readiness
            },
        }
    }
}

struct TestScope {
    _lock: MutexGuard<'static, ()>,
    generations: Vec<revocation::WorkerGeneration>,
}

impl TestScope {
    fn new() -> Self {
        let lock = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        worker_acceptance_budget::FAIL_FINISH.store(false, Ordering::SeqCst);
        cooling::FAIL_RESTORE.store(false, Ordering::SeqCst);
        worker_acceptance_budget::FINISH_CALLS.store(0, Ordering::SeqCst);
        Self {
            _lock: lock,
            generations: Vec::new(),
        }
    }

    fn link(&mut self) -> revocation::WorkerGeneration {
        let generation = revocation::begin_link(1_000).expect("prior test released its gate");
        self.generations.push(generation);
        generation
    }
}

impl Drop for TestScope {
    fn drop(&mut self) {
        // Also runs on assertion failure, without altering production gate internals.
        for generation in &self.generations {
            revocation::revoke_at(*generation, 1_000);
            revocation::finish_shutdown(*generation);
        }
    }
}

fn readiness() -> ProductionReadiness {
    ProductionReadiness {
        operator_intent: MiningOperatorIntent::Run,
        network_ready: true,
        stratum_v1_supported: true,
        safety_prerequisites_fresh: true,
        maybe_campaign_lease: None,
        actuation_qualified: true,
    }
}

fn blocked_adapter() -> OrdinaryEspProductionSessionAdapter {
    OrdinaryEspProductionSessionAdapter {
        maybe_bwg_session: None,
        maybe_bwg_reply: None,
        maybe_cooling_generation: None,
        readiness: ProductionReadiness {
            safety_prerequisites_fresh: false,
            ..readiness()
        },
    }
}

fn start(
    scope: &mut TestScope,
    adapter: &mut OrdinaryEspProductionSessionAdapter,
    core: &mut ProductionMiningSession,
) -> (
    revocation::WorkerGeneration,
    Receiver<Result<(), bwg::Error>>,
) {
    let generation = scope.link();
    assert!(revocation::admit_budget(generation, 180_000));
    let (reply, receiver) = mpsc::sync_channel(1);
    let event = adapter.event(
        bwg::OwnerCommand::Start {
            generation,
            worker_lease_id: "synthetic-lease".to_owned(),
            deadline: MiningCampaignMonotonicDeadline::new(61_000).expect("valid deadline"),
            pools: ProductionPoolSet {
                primary: None,
                fallback: None,
                prefer_fallback: false,
            },
            reply,
        },
        1_000,
        &core.snapshot(),
        core.next_campaign_lease_id(),
    );
    let effects = core.handle(event).expect("synthetic readiness event");
    for effect in effects {
        if matches!(
            effect,
            bitaxe_stratum::v1::production_session::ProductionSessionEffect::PrepareHardware { .. }
        ) {
            adapter.note_worker_preparation_started();
        }
    }
    adapter.complete_reply(&core.snapshot());
    (generation, receiver)
}

#[test]
fn blocked_initial_readiness_rejects_start_without_waiting_for_rpc_timeout() {
    // Arrange
    let mut scope = TestScope::new();
    let mut adapter = blocked_adapter();
    let mut core = ProductionMiningSession::new();

    // Act
    let (_, reply) = start(&mut scope, &mut adapter, &mut core);

    // Assert
    assert_eq!(reply.try_recv(), Ok(Err(bwg::Error::Rejected)));
}

#[test]
fn blocked_initial_start_can_retire_and_admit_a_fresh_link() {
    // Arrange
    let mut scope = TestScope::new();
    let mut adapter = blocked_adapter();
    let mut core = ProductionMiningSession::new();
    let (generation, _reply) = start(&mut scope, &mut adapter, &mut core);

    // Act
    revocation::revoke_at(generation, 1_100);
    core.handle(ProductionSessionEvent::CampaignLeaseRevoked)
        .expect("revocation event");
    adapter.complete_reply(&core.snapshot());
    let maybe_fresh = revocation::begin_link(1_200);
    if let Some(fresh) = maybe_fresh {
        scope.generations.push(fresh);
    }

    // Assert
    assert!(
        maybe_fresh.is_some(),
        "blocked preparation must not strand recovery admission"
    );
}

#[test]
fn blocked_start_finalizes_its_reserved_budget_before_releasing_ownership() {
    // Arrange
    let mut scope = TestScope::new();
    let mut adapter = blocked_adapter();
    let mut core = ProductionMiningSession::new();

    // Act
    let (_, _reply) = start(&mut scope, &mut adapter, &mut core);

    // Assert
    assert!(worker_acceptance_budget::FINISH_CALLS.load(Ordering::SeqCst) > 0);
}

#[test]
fn failed_budget_finalization_blocks_recovery_until_cleanup_succeeds() {
    // Arrange
    let mut scope = TestScope::new();
    let mut adapter = blocked_adapter();
    let mut core = ProductionMiningSession::new();
    worker_acceptance_budget::FAIL_FINISH.store(true, Ordering::SeqCst);
    let (generation, _reply) = start(&mut scope, &mut adapter, &mut core);
    revocation::revoke_at(generation, 1_100);
    core.handle(ProductionSessionEvent::CampaignLeaseRevoked)
        .expect("revocation event");
    adapter.complete_reply(&core.snapshot());
    let blocked = revocation::begin_link(1_200);
    if let Some(unexpected) = blocked {
        scope.generations.push(unexpected);
    }

    // Act
    worker_acceptance_budget::FAIL_FINISH.store(false, Ordering::SeqCst);
    adapter.complete_reply(&core.snapshot());
    let maybe_recovered = revocation::begin_link(1_300);
    if let Some(recovered) = maybe_recovered {
        scope.generations.push(recovered);
    }

    // Assert
    assert!(
        blocked.is_none(),
        "durable cleanup failure must retain ownership"
    );
    assert!(
        maybe_recovered.is_some(),
        "successful cleanup retry must release ownership"
    );
}

fn previously_consumed_core() -> ProductionMiningSession {
    let mut core = ProductionMiningSession::new();
    let lease_id = MiningCampaignLeaseId::new(1).expect("valid lease id");
    let lease = MiningCampaignLease::new(
        lease_id,
        MiningHardwareProfilePreset::Conservative.profile(),
        MiningCampaignStopCondition::MonotonicDeadline {
            deadline: MiningCampaignMonotonicDeadline::new(61_000).expect("valid deadline"),
        },
    );
    core.handle(ProductionSessionEvent::Wake {
        wakeup: None,
        readiness: ProductionReadiness {
            maybe_campaign_lease: Some(lease),
            ..readiness()
        },
        now_ms: 1_000,
    })
    .expect("prepare synthetic prior campaign");
    core.handle(ProductionSessionEvent::CampaignLeaseRevoked)
        .expect("revoke prior campaign");
    core.handle(ProductionSessionEvent::HardwareSafeStopConfirmed {
        lease_id,
        now_ms: 1_001,
    })
    .expect("confirm prior synthetic stop");
    assert_eq!(
        core.snapshot().campaign_state,
        MiningCampaignState::Consumed
    );
    assert_eq!(core.snapshot().hardware_state, MiningHardwareState::Stopped);
    core
}

#[test]
fn blocked_new_start_after_consumption_rejects_without_hardware_preparation() {
    // Arrange
    let mut scope = TestScope::new();
    let mut adapter = blocked_adapter();
    let mut core = previously_consumed_core();

    // Act
    let (generation, reply) = start(&mut scope, &mut adapter, &mut core);

    // Assert
    assert_eq!(reply.try_recv(), Ok(Err(bwg::Error::Rejected)));
    let timing = revocation::timing(1_100).expect("reserved generation tracked");
    assert_eq!(timing.generation, generation.raw());
    assert_eq!(timing.work_dispatched, 0);
    assert_eq!(timing.shutdown_stage, 0);
    assert_eq!(core.snapshot().hardware_state, MiningHardwareState::Stopped);
    assert!(!revocation::is_live(generation));
    assert!(adapter.maybe_bwg_session.is_none());
    assert_eq!(
        worker_acceptance_budget::FINISH_CALLS.load(Ordering::SeqCst),
        1
    );
}

#[test]
fn prepared_start_waits_for_stop_confirmation_despite_late_preparation_completion() {
    // Arrange
    let mut scope = TestScope::new();
    let mut adapter = blocked_adapter();
    adapter.readiness = readiness();
    let mut core = ProductionMiningSession::new();
    let (generation, reply) = start(&mut scope, &mut adapter, &mut core);
    let lease_id = adapter
        .maybe_bwg_session
        .as_ref()
        .expect("registered owner")
        .lease
        .id();
    assert_eq!(
        core.snapshot().hardware_state,
        MiningHardwareState::Preparing
    );
    assert_eq!(reply.try_recv(), Err(mpsc::TryRecvError::Empty));

    // Act
    revocation::revoke_at(generation, 1_100);
    core.handle(ProductionSessionEvent::CampaignLeaseRevoked)
        .expect("revoke preparation");
    core.handle(ProductionSessionEvent::HardwarePrepared {
        lease_id,
        now_ms: 1_101,
    })
    .expect("late preparation");
    adapter.complete_reply(&core.snapshot());

    // Assert
    assert_eq!(
        core.snapshot().hardware_state,
        MiningHardwareState::SafeStopping
    );
    assert!(!revocation::permits(Some(generation)));
    assert_eq!(reply.try_recv(), Err(mpsc::TryRecvError::Empty));
    assert!(revocation::begin_link(1_200).is_none());
    assert_eq!(
        worker_acceptance_budget::FINISH_CALLS.load(Ordering::SeqCst),
        0
    );
}

#[test]
fn safe_stop_ack_waits_for_durable_finalization_after_hardware_confirmation() {
    // Arrange
    let mut scope = TestScope::new();
    let mut adapter = blocked_adapter();
    adapter.readiness = readiness();
    let mut core = ProductionMiningSession::new();
    let (generation, _start_reply) = start(&mut scope, &mut adapter, &mut core);
    let lease_id = adapter
        .maybe_bwg_session
        .as_ref()
        .expect("registered owner")
        .lease
        .id();
    let (reply, receiver) = mpsc::sync_channel(1);
    let event = adapter.event(
        bwg::OwnerCommand::SafeStop { reply },
        1_100,
        &core.snapshot(),
        core.next_campaign_lease_id(),
    );
    core.handle(event).expect("ordered stop");
    core.handle(ProductionSessionEvent::HardwareSafeStopConfirmed {
        lease_id,
        now_ms: 1_200,
    })
    .expect("qualified synthetic stop");
    worker_acceptance_budget::FAIL_FINISH.store(true, Ordering::SeqCst);

    // Act
    adapter.complete_reply(&core.snapshot());
    let before_finalization = receiver.try_recv();
    let maybe_early_link = revocation::begin_link(1_201);
    if let Some(early) = maybe_early_link {
        scope.generations.push(early);
    }
    worker_acceptance_budget::FAIL_FINISH.store(false, Ordering::SeqCst);
    adapter.complete_reply(&core.snapshot());
    let after_finalization = receiver.try_recv();
    let maybe_fresh = revocation::begin_link(1_300);
    if let Some(fresh) = maybe_fresh {
        scope.generations.push(fresh);
    }

    // Assert
    assert_eq!(before_finalization, Err(mpsc::TryRecvError::Empty));
    assert!(maybe_early_link.is_none());
    assert_eq!(after_finalization, Ok(Ok(())));
    assert!(maybe_fresh.is_some());
    assert!(!revocation::permits(Some(generation)));
}

#[test]
fn network_unavailable_rejects_start_without_hardware_preparation() {
    // Arrange
    let mut scope = TestScope::new();
    let mut adapter = blocked_adapter();
    adapter.readiness = ProductionReadiness {
        network_ready: false,
        ..readiness()
    };
    let mut core = ProductionMiningSession::new();

    // Act
    let (generation, reply) = start(&mut scope, &mut adapter, &mut core);

    // Assert
    assert_eq!(reply.try_recv(), Ok(Err(bwg::Error::Rejected)));
    assert_eq!(
        core.snapshot().hardware_state,
        MiningHardwareState::Unprepared
    );
    assert!(!revocation::permits(Some(generation)));
}

#[test]
fn actuation_unavailable_rejects_start_without_hardware_preparation() {
    // Arrange
    let mut scope = TestScope::new();
    let mut adapter = blocked_adapter();
    adapter.readiness = ProductionReadiness {
        actuation_qualified: false,
        ..readiness()
    };
    let mut core = ProductionMiningSession::new();

    // Act
    let (generation, reply) = start(&mut scope, &mut adapter, &mut core);

    // Assert
    assert_eq!(reply.try_recv(), Ok(Err(bwg::Error::Rejected)));
    assert_eq!(
        core.snapshot().hardware_state,
        MiningHardwareState::Unprepared
    );
    assert!(!revocation::permits(Some(generation)));
}

#[test]
fn rejected_unprepared_start_does_not_acknowledge_stop_during_budget_failure() {
    // Arrange
    let mut scope = TestScope::new();
    let mut adapter = blocked_adapter();
    let mut core = ProductionMiningSession::new();
    worker_acceptance_budget::FAIL_FINISH.store(true, Ordering::SeqCst);
    let (_, start_reply) = start(&mut scope, &mut adapter, &mut core);
    assert_eq!(start_reply.try_recv(), Ok(Err(bwg::Error::Rejected)));
    let (reply, receiver) = mpsc::sync_channel(1);
    let event = adapter.event(
        bwg::OwnerCommand::SafeStop { reply },
        1_100,
        &core.snapshot(),
        core.next_campaign_lease_id(),
    );
    core.handle(event).expect("restore rejected candidate");

    // Act
    adapter.complete_reply(&core.snapshot());
    let pending = receiver.try_recv();
    worker_acceptance_budget::FAIL_FINISH.store(false, Ordering::SeqCst);
    adapter.complete_reply(&core.snapshot());

    // Assert
    assert_eq!(pending, Err(mpsc::TryRecvError::Empty));
    assert_eq!(receiver.try_recv(), Ok(Ok(())));
    assert!(adapter.maybe_bwg_session.is_none());
}

#[path = "production_worker_admission_host_test/start_boundary.rs"]
mod start_boundary;

#[path = "production_mining_session/cooling_core.rs"]
mod cooling_core;
mod cooling {
    use super::*;
    pub(crate) static FAIL_RESTORE: AtomicBool = AtomicBool::new(false);
    pub(crate) fn qualify(
        _: revocation::WorkerGeneration,
    ) -> Result<serde_json::Value, cooling_core::CoolingError> {
        Ok(serde_json::json!({"fan_duty_percent":100}))
    }
    pub(crate) fn restore(
        _: revocation::WorkerGeneration,
    ) -> Result<serde_json::Value, cooling_core::CoolingError> {
        if FAIL_RESTORE.load(Ordering::SeqCst) {
            Err(cooling_core::CoolingError::Rejected)
        } else {
            Ok(serde_json::json!({"fan_duty_percent":30}))
        }
    }
}

#[test]
fn cooling_owner_fences_link_without_reserving_mining_budget() {
    // Arrange
    let mut scope = TestScope::new();
    let generation = scope.link();
    let mut adapter = blocked_adapter();
    let snapshot = ProductionMiningSession::new().snapshot();
    let (reply, receiver) = mpsc::sync_channel(1);
    // Act
    adapter.event(
        bwg::OwnerCommand::Cooling {
            generation,
            restore: false,
            reply,
        },
        1_000,
        &snapshot,
        None,
    );
    // Assert
    assert!(receiver.try_recv().expect("cooling reply").is_ok());
    assert_eq!(adapter.maybe_cooling_generation, Some(generation));
    assert!(!revocation::permits(Some(generation)));
    assert_eq!(
        worker_acceptance_budget::FINISH_CALLS.load(Ordering::SeqCst),
        0
    );
    revocation::revoke_at(generation, 1_000);
    assert!(revocation::begin_link(1_000).is_none());
}

#[test]
fn no_session_safe_stop_cannot_acknowledge_failed_cooling_restore() {
    // Arrange
    let mut scope = TestScope::new();
    let generation = scope.link();
    assert!(revocation::begin_reservation(generation));
    let mut adapter = blocked_adapter();
    adapter.maybe_cooling_generation = Some(generation);
    cooling::FAIL_RESTORE.store(true, Ordering::SeqCst);
    revocation::revoke_at(generation, 1_000);
    let snapshot = ProductionMiningSession::new().snapshot();
    let (reply, receiver) = mpsc::sync_channel(1);
    // Act
    adapter.event(
        bwg::OwnerCommand::SafeStop { reply },
        1_000,
        &snapshot,
        None,
    );
    // Assert
    assert_eq!(
        receiver.try_recv().expect("stop reply"),
        Err(bwg::Error::Rejected)
    );
    assert_eq!(adapter.maybe_cooling_generation, Some(generation));
    assert!(revocation::begin_link(1_000).is_none());
}
