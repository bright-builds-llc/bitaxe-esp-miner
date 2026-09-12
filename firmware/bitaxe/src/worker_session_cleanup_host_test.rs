#![allow(dead_code)]
//! Actual session cleanup and durable budget adapter with synthetic NVS/owner boundaries.
#[path = "production_mining_session/admission_diagnostics.rs"]
mod admission_diagnostics;
#[path = "bwg_worker_session.rs"]
mod bwg_worker_session;
#[path = "production_mining_session/revocation.rs"]
mod revocation;
#[path = "production_mining_session/shutdown_budget.rs"]
mod shutdown_budget;
#[path = "worker_acceptance_budget.rs"]
mod worker_acceptance_budget;
#[path = "worker_qualification_budget.rs"]
mod worker_qualification_budget;

use bitaxe_api::acceptance_budget::AcceptanceBudget;
use bitaxe_worker_control::{
    RestorationReason, WorkerLeaseGrant, WorkerSession, WorkerSessionError,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard};
static TEST_LOCK: Mutex<()> = Mutex::new(());
static LEDGER: Mutex<Option<AcceptanceBudget>> = Mutex::new(None);
static QUAL_LEDGER: Mutex<Option<bitaxe_worker_control::QualificationLedger>> = Mutex::new(None);
static FAIL_WRITE: AtomicBool = AtomicBool::new(false);
static INTERRUPT_WRITE: AtomicBool = AtomicBool::new(false);
static CLEANUP_WAS_BUSY: AtomicBool = AtomicBool::new(false);
static WRITE_GENERATION: Mutex<Option<revocation::WorkerGeneration>> = Mutex::new(None);
static FAIL_AFTER_WRITE: AtomicBool = AtomicBool::new(false);
static FAIL_OWNER_STOP: AtomicBool = AtomicBool::new(false);
const CAMPAIGN: &str = "synthetic_campaign_001";

mod runtime_uptime {
    pub fn millis() -> u64 {
        1_000
    }
}
// This fixture replaces USB ownership while exercising the real durable cleanup adapter.
mod bwg_worker_usb {
    pub(crate) mod trace {
        use bitaxe_worker_control::serial::trace::{SerialTrace, SerialTraceSnapshot};
        static TRACE: SerialTrace = SerialTrace::new();
        pub(crate) fn snapshot() -> SerialTraceSnapshot {
            TRACE.snapshot()
        }
    }
}
mod startup {
    pub struct BootMiningBaselineConfirmed;
}
mod settings_adapter {
    pub mod preservation {
        pub fn read() -> Result<bitaxe_worker_control::SettingsPreservation, ()> {
            Err(())
        }
    }
}
mod bwg_worker_nvs {
    use super::*;
    pub struct BwgWorkerNvs;
    impl BwgWorkerNvs {
        pub fn open() -> Result<Self, ()> {
            Ok(Self)
        }
        pub fn qualification_ledger(
            &self,
        ) -> anyhow::Result<bitaxe_worker_control::QualificationLedger> {
            Ok(QUAL_LEDGER
                .lock()
                .expect("test ledger")
                .clone()
                .unwrap_or_default())
        }
        pub fn store_qualification_ledger(
            &mut self,
            ledger: &bitaxe_worker_control::QualificationLedger,
        ) -> anyhow::Result<()> {
            if FAIL_WRITE.load(Ordering::SeqCst) {
                anyhow::bail!("synthetic write failure");
            }
            if INTERRUPT_WRITE.swap(false, Ordering::SeqCst) {
                let generation = WRITE_GENERATION
                    .lock()
                    .expect("test generation")
                    .expect("generation");
                revocation::revoke_at(generation, 1000);
                CLEANUP_WAS_BUSY.store(
                    worker_acceptance_budget::finish(generation).is_err(),
                    Ordering::SeqCst,
                );
            }
            *QUAL_LEDGER.lock().expect("test ledger") = Some(ledger.clone());
            if FAIL_AFTER_WRITE.load(Ordering::SeqCst) {
                anyhow::bail!("synthetic readback failure");
            }
            Ok(())
        }
        pub fn maybe_acceptance_budget(&self) -> anyhow::Result<Option<AcceptanceBudget>> {
            Ok(LEDGER.lock().expect("test ledger").clone())
        }
        pub fn store_acceptance_budget(&mut self, budget: &AcceptanceBudget) -> anyhow::Result<()> {
            if FAIL_WRITE.load(Ordering::SeqCst) {
                anyhow::bail!("synthetic storage failure");
            }
            if INTERRUPT_WRITE.swap(false, Ordering::SeqCst) {
                let generation = WRITE_GENERATION
                    .lock()
                    .expect("test generation")
                    .expect("reserved generation");
                revocation::revoke_at(generation, 1_000);
                CLEANUP_WAS_BUSY.store(
                    worker_acceptance_budget::finish(generation).is_err(),
                    Ordering::SeqCst,
                );
            }
            *LEDGER.lock().expect("test ledger") = Some(budget.clone());
            if FAIL_AFTER_WRITE.load(Ordering::SeqCst) {
                anyhow::bail!("synthetic readback failure");
            }
            Ok(())
        }
    }
}
mod production_mining_session {
    pub(crate) fn bwg_cooling(
        _generation: crate::revocation::WorkerGeneration,
        _restore: bool,
    ) -> Result<serde_json::Value, ()> {
        Err(())
    }

    pub(crate) use super::{admission_diagnostics, revocation};
    pub fn status_evidence(_: Option<revocation::WorkerGeneration>) -> Option<serde_json::Value> {
        None
    }
    pub fn bwg_start(
        _: &bitaxe_worker_control::WorkerLeaseGrant,
        _: bitaxe_worker_control::LeaseDeadlines,
        _: revocation::WorkerGeneration,
    ) -> Result<(), ()> {
        Err(())
    }
    pub fn bwg_renew(
        _: &bitaxe_worker_control::WorkerLeaseRenewal,
        _: bitaxe_worker_control::LeaseDeadlines,
        _: revocation::WorkerGeneration,
    ) -> Result<(), ()> {
        Err(())
    }
    pub fn bwg_safe_stop() -> Result<(), ()> {
        if super::FAIL_OWNER_STOP.load(super::Ordering::SeqCst) {
            Err(())
        } else {
            Ok(())
        }
    }
}
struct Scope {
    _lock: MutexGuard<'static, ()>,
    generation: revocation::WorkerGeneration,
}
impl Scope {
    fn new() -> Self {
        let lock = TEST_LOCK.lock().unwrap_or_else(|error| error.into_inner());
        FAIL_WRITE.store(false, Ordering::SeqCst);
        FAIL_AFTER_WRITE.store(false, Ordering::SeqCst);
        FAIL_OWNER_STOP.store(false, Ordering::SeqCst);
        *LEDGER.lock().expect("test ledger") = None;
        *QUAL_LEDGER.lock().expect("test ledger") = None;
        let generation = revocation::begin_link(1_000).expect("previous test cleaned up");
        *WRITE_GENERATION.lock().expect("test generation") = Some(generation);
        INTERRUPT_WRITE.store(false, Ordering::SeqCst);
        CLEANUP_WAS_BUSY.store(false, Ordering::SeqCst);
        Self {
            _lock: lock,
            generation,
        }
    }
    fn reserved(&self) -> bwg_worker_session::ProductionWorkerSession {
        let grant = synthetic_grant();
        worker_acceptance_budget::admit(self.generation, &grant).expect("reserve test window");
        let mut session = bwg_worker_session::ProductionWorkerSession::default();
        session.set_generation(self.generation);
        session
    }
}
impl Drop for Scope {
    fn drop(&mut self) {
        FAIL_WRITE.store(false, Ordering::SeqCst);
        FAIL_AFTER_WRITE.store(false, Ordering::SeqCst);
        revocation::revoke_at(self.generation, 1_000);
        worker_acceptance_budget::finish(self.generation).expect("test cleanup");
        revocation::finish_shutdown(self.generation);
    }
}
fn ledger() -> AcceptanceBudget {
    LEDGER
        .lock()
        .expect("test ledger")
        .clone()
        .expect("reserved ledger")
}

#[test]
fn no_owner_stop_finalizes_reserved_window_without_refund() {
    // Arrange: reservation succeeded, but Start never became an owner session.
    let scope = Scope::new();
    let mut session = scope.reserved();
    let expected = ledger().finish().expect("expected terminal ledger");
    // Act
    let result = session.safe_stop(RestorationReason::ControlFailed);
    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(ledger(), expected);
    assert_eq!(ledger().charged_milliseconds(), 180_000);
}

#[test]
fn durable_cleanup_failure_cannot_acknowledge_restoration() {
    // Arrange
    let scope = Scope::new();
    let mut session = scope.reserved();
    FAIL_WRITE.store(true, Ordering::SeqCst);
    // Act
    let result = session.safe_stop(RestorationReason::ControlFailed);
    // Assert
    assert_eq!(result, Err(WorkerSessionError::SafeStopFailed));
    assert_ne!(ledger(), ledger().finish().expect("terminal ledger"));
}

#[test]
fn owner_stop_failure_keeps_reservation_pending() {
    // Arrange: acknowledgement is unavailable, so queued/active effects are not disproved.
    let scope = Scope::new();
    let mut session = scope.reserved();
    FAIL_OWNER_STOP.store(true, Ordering::SeqCst);
    let pending = ledger();
    // Act
    let result = session.safe_stop(RestorationReason::ControlFailed);
    // Assert
    assert_eq!(result, Err(WorkerSessionError::SafeStopFailed));
    assert_eq!(ledger(), pending);
}

fn synthetic_grant() -> WorkerLeaseGrant {
    serde_json::from_value(serde_json::json!({
            "protocolVersion":"0.4", "leaseId":"synthetic_lease", "challengeId":"synthetic_challenge",
            "authorization":"synthetic-non-production-authorization", "durationMilliseconds":60000,
            "renewAfterMilliseconds":20000,
            "stratum":{"endpoint":"stratum+tcp://example.invalid:3333/","username":"synthetic","password":"synthetic"},
            "acceptanceCampaign":{"id":CAMPAIGN,"maximumActiveMilliseconds":180000,"window":0}
        })).expect("synthetic grant")
}

#[test]
fn ambiguous_reservation_write_is_finalized_without_refund() {
    // Arrange
    let scope = Scope::new();
    FAIL_AFTER_WRITE.store(true, Ordering::SeqCst);
    assert!(worker_acceptance_budget::admit(scope.generation, &synthetic_grant()).is_err());
    let expected = ledger().finish().expect("terminal ledger");
    FAIL_AFTER_WRITE.store(false, Ordering::SeqCst);
    let mut session = bwg_worker_session::ProductionWorkerSession::default();
    session.set_generation(scope.generation);
    // Act
    let result = session.safe_stop(RestorationReason::ControlFailed);
    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(ledger(), expected);
    assert_eq!(ledger().charged_milliseconds(), 180_000);
}

#[test]
fn failed_reservation_write_conservatively_charges_window_on_cleanup() {
    // Arrange
    let scope = Scope::new();
    FAIL_WRITE.store(true, Ordering::SeqCst);
    assert!(worker_acceptance_budget::admit(scope.generation, &synthetic_grant()).is_err());
    FAIL_WRITE.store(false, Ordering::SeqCst);
    let mut session = bwg_worker_session::ProductionWorkerSession::default();
    session.set_generation(scope.generation);
    // Act
    let result = session.safe_stop(RestorationReason::ControlFailed);
    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(ledger(), ledger().finish().expect("terminal ledger"));
    assert_eq!(ledger().charged_milliseconds(), 180_000);
}

#[test]
fn durable_cleanup_failure_blocks_new_link_until_retry_completes() {
    // Arrange
    let scope = Scope::new();
    let mut session = scope.reserved();
    FAIL_WRITE.store(true, Ordering::SeqCst);
    // Act
    assert_eq!(
        session.safe_stop(RestorationReason::ControlFailed),
        Err(WorkerSessionError::SafeStopFailed)
    );
    // Assert
    assert!(revocation::begin_link(1_000).is_none());
    FAIL_WRITE.store(false, Ordering::SeqCst);
    assert_eq!(session.safe_stop(RestorationReason::ControlFailed), Ok(()));
    let fresh = revocation::begin_link(1_000).expect("durable cleanup permits fresh link");
    revocation::revoke_at(fresh, 1_000);
}

#[test]
fn revoked_during_reservation_write_cannot_finish_in_flight_admission() {
    // Arrange
    let scope = Scope::new();
    INTERRUPT_WRITE.store(true, Ordering::SeqCst);
    // Act
    let result = worker_acceptance_budget::admit(scope.generation, &synthetic_grant());
    // Assert
    assert!(result.is_err());
    assert!(CLEANUP_WAS_BUSY.load(Ordering::SeqCst));
    assert!(revocation::begin_link(1_000).is_none());
    let mut session = bwg_worker_session::ProductionWorkerSession::default();
    session.set_generation(scope.generation);
    assert_eq!(session.safe_stop(RestorationReason::ControlFailed), Ok(()));
    assert_eq!(ledger(), ledger().finish().expect("terminal ledger"));
    assert_eq!(ledger().charged_milliseconds(), 180_000);
}

#[test]
fn repeated_cleanup_preserves_terminal_reservation() {
    // Arrange
    let scope = Scope::new();
    let mut session = scope.reserved();
    assert_eq!(session.safe_stop(RestorationReason::ControlFailed), Ok(()));
    let expected = ledger();
    // Act
    let result = session.safe_stop(RestorationReason::ControlFailed);
    // Assert
    assert_eq!(result, Ok(()));
    assert_eq!(ledger(), expected);
}

#[test]
fn budget_review_does_not_create_an_absent_ledger() {
    // Arrange
    let _scope = Scope::new();
    // Act
    let review = worker_acceptance_budget::review(CAMPAIGN).expect("read-only review");
    // Assert
    assert_eq!(
        review,
        serde_json::json!({
            "schema":"worker-budget-review-v1", "campaign_match":false,
            "reserved_mask":0,"completed_mask":0,"charged_ms":0,"pending":false
        })
    );
    assert!(LEDGER.lock().expect("test ledger").is_none());
}

#[test]
fn budget_review_reports_pending_charge_without_finalizing_it() {
    // Arrange
    let scope = Scope::new();
    let session = scope.reserved();
    let before = ledger();
    // Act
    let review = session
        .acceptance_budget_review(CAMPAIGN)
        .expect("read-only review")
        .expect("supported");
    // Assert
    assert_eq!(
        review,
        serde_json::json!({
            "schema":"worker-budget-review-v1", "campaign_match":true,
            "reserved_mask":1,"completed_mask":0,"charged_ms":180000,"pending":true
        })
    );
    assert_eq!(ledger(), before);
}

#[test]
fn budget_review_does_not_echo_mismatched_campaign() {
    // Arrange
    let scope = Scope::new();
    let _session = scope.reserved();
    // Act
    let review =
        worker_acceptance_budget::review("other_synthetic_campaign").expect("read-only review");
    // Assert
    assert_eq!(review["campaign_match"], false);
    assert!(!review.to_string().contains(CAMPAIGN));
    assert!(!review.to_string().contains("other_synthetic_campaign"));
}

#[test]
fn budget_review_rejects_malformed_ledger_without_repair() {
    // Arrange
    let scope = Scope::new();
    let _session = scope.reserved();
    let before = ledger();
    let mut corrupt = serde_json::to_value(&before).expect("synthetic ledger");
    corrupt["charged_ms"] = serde_json::json!(1);
    *LEDGER.lock().expect("test ledger") =
        Some(serde_json::from_value(corrupt).expect("synthetic invalid ledger"));
    // Act
    let result = worker_acceptance_budget::review(CAMPAIGN);
    let after = ledger();
    *LEDGER.lock().expect("test ledger") = Some(before);
    // Assert
    assert!(result.is_err());
    assert_eq!(after.charged_milliseconds(), 1);
}

#[path = "worker_session_cleanup_host_test/qualification.rs"]
mod qualification_tests;
