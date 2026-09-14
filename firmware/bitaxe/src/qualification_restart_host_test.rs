//! Runs the actual restart adapter with synthetic boot, USB and CPU-reset boundaries.
extern crate self as esp_idf_svc;
#[path = "qualification_restart.rs"]
mod qualification_restart;

use bitaxe_worker_control::QualificationRestartContext;
use production_mining_session::revocation::WorkerGeneration;
use std::cell::RefCell;

#[derive(Clone)]
struct State {
    startup: bool,
    mine_on_boot: bool,
    maybe_epoch: Option<u32>,
    idle: bool,
    claim_allowed: bool,
    epoch_claim_allowed: bool,
    boot: u64,
    now: u64,
    resets: u32,
}
impl Default for State {
    fn default() -> Self {
        Self {
            startup: true,
            mine_on_boot: false,
            maybe_epoch: Some(1),
            idle: true,
            claim_allowed: true,
            epoch_claim_allowed: true,
            boot: 2,
            now: 1000,
            resets: 0,
        }
    }
}
thread_local! { static STATE: RefCell<State> = RefCell::new(State::default()); }
fn read<T>(operation: impl FnOnce(&State) -> T) -> T {
    STATE.with_borrow(operation)
}
fn change(operation: impl FnOnce(&mut State)) {
    STATE.with_borrow_mut(operation);
}
fn context() -> QualificationRestartContext {
    QualificationRestartContext {
        boot_ordinal: 2,
        worker_generation: 7,
        transport_epoch: 1,
    }
}
mod runtime_uptime {
    pub fn millis() -> u64 {
        super::read(|s| s.now)
    }
}
mod boot_evidence {
    pub fn operator_snapshot_boot_ordinal() -> u64 {
        super::read(|s| s.boot)
    }
}
mod settings_adapter {
    pub fn start_mining_on_boot() -> bool {
        super::read(|s| s.mine_on_boot)
    }
}
mod bwg_worker_usb {
    pub fn maybe_authenticated_epoch() -> Option<u32> {
        super::read(|s| s.maybe_epoch)
    }
    pub fn claim_restart_epoch(epoch: u32) -> bool {
        let claimed = super::read(|s| s.epoch_claim_allowed && s.maybe_epoch == Some(epoch));
        if claimed {
            super::change(|s| s.maybe_epoch = None);
        }
        claimed
    }
    pub mod startup_diagnostics {
        pub struct Progress;
        pub static PROGRESS: Progress = Progress;
        impl Progress {
            pub fn successful(&self) -> bool {
                super::super::read(|s| s.startup)
            }
        }
    }
}
mod production_mining_session {
    pub mod revocation {
        #[derive(Clone, Copy)]
        pub struct WorkerGeneration(pub u32);
        impl WorkerGeneration {
            pub fn raw(self) -> u32 {
                self.0
            }
        }
        pub fn is_idle(generation: WorkerGeneration) -> bool {
            super::super::read(|s| s.idle && generation.0 == 7)
        }
        pub fn abort_idle_restart(_generation: WorkerGeneration) -> bool {
            super::super::change(|s| s.idle = true);
            true
        }
        pub fn claim_idle_restart(generation: WorkerGeneration) -> bool {
            let claimed = is_idle(generation) && super::super::read(|s| s.claim_allowed);
            if claimed {
                super::super::change(|s| s.idle = false);
            }
            claimed
        }
    }
}
pub mod sys {
    /// Synthetic non-returning ESP boundary, caught only by the positive host test.
    /// # Safety
    /// Only called through the real restart adapter after its final idle claim.
    pub unsafe fn esp_restart() -> ! {
        super::change(|s| s.resets += 1);
        panic!("synthetic CPU restart");
    }
}

#[test]
fn readiness_requires_completed_startup_and_disabled_boot_mining() {
    // Arrange
    change(|s| s.startup = false);
    // Act / Assert
    assert_eq!(
        qualification_restart::context(WorkerGeneration(7)).expect("read"),
        None
    );
    change(|s| {
        s.startup = true;
        s.mine_on_boot = true;
    });
    assert_eq!(
        qualification_restart::context(WorkerGeneration(7)).expect("read"),
        None
    );
    assert_eq!(read(|s| s.resets), 0);
}

#[test]
fn readiness_requires_authenticated_epoch_and_exact_idle_generation() {
    // Arrange
    change(|s| s.maybe_epoch = None);
    // Act / Assert
    assert_eq!(
        qualification_restart::context(WorkerGeneration(7)).expect("read"),
        None
    );
    change(|s| s.maybe_epoch = Some(1));
    assert_eq!(
        qualification_restart::context(WorkerGeneration(8)).expect("read"),
        None
    );
    change(|s| s.idle = false);
    assert_eq!(
        qualification_restart::context(WorkerGeneration(7)).expect("read"),
        None
    );
}

#[test]
fn fresh_clock_expiry_cannot_reach_the_cpu_reset_boundary() {
    // Arrange
    change(|s| s.now = 2000);
    // Act
    let result = qualification_restart::restart(WorkerGeneration(7), context(), 2000);
    // Assert
    assert!(result.is_err());
    assert!(read(|s| s.idle));
    assert_eq!(read(|s| s.resets), 0);
}

#[test]
fn context_change_after_ack_cannot_reach_the_cpu_reset_boundary() {
    // Arrange
    change(|s| s.maybe_epoch = Some(2));
    // Act
    let result = qualification_restart::restart(WorkerGeneration(7), context(), 2000);
    // Assert
    assert!(result.is_err());
    assert!(read(|s| s.idle));
    assert_eq!(read(|s| s.resets), 0);
}

#[test]
fn cancellation_at_final_idle_claim_cannot_reach_the_cpu_reset_boundary() {
    // Arrange
    change(|s| s.claim_allowed = false);
    // Act
    let result = qualification_restart::restart(WorkerGeneration(7), context(), 2000);
    // Assert
    assert!(result.is_err());
    assert_eq!(read(|s| s.resets), 0);
}

#[test]
fn confirmed_restart_claims_idle_before_calling_the_nonreturning_cpu_boundary() {
    // Arrange
    assert_eq!(
        qualification_restart::context(WorkerGeneration(7)).expect("read"),
        Some(context())
    );
    // Act
    let result = std::panic::catch_unwind(|| {
        qualification_restart::restart(WorkerGeneration(7), context(), 2000)
    });
    // Assert
    assert!(result.is_err());
    assert_eq!(read(|s| s.resets), 1);
    assert!(!read(|s| s.idle));
    assert!(qualification_restart::restart(WorkerGeneration(7), context(), 2000).is_err());
    assert_eq!(read(|s| s.resets), 1);
}

#[test]
fn native_cancellation_winning_the_commit_cas_releases_idle_without_reset() {
    // Arrange
    change(|s| s.epoch_claim_allowed = false);
    // Act
    let result = qualification_restart::restart(WorkerGeneration(7), context(), 2000);
    // Assert
    assert!(result.is_err());
    assert!(read(|s| s.idle));
    assert_eq!(read(|s| s.resets), 0);
}
