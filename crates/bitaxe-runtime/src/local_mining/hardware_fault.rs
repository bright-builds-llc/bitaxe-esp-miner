//! Upstream's `hardware_fault` for local mining (ADR-0038). Upstream's fan controller
//! flags it when a fan write fails, and its power management task then stops mining
//! until reboot. Here the latch covers local mining only (owner decision
//! 2026-10-17): a Gate lease's failed write follows the lease's own stops.

use crate::revocation::{FanControllerOwner, WorkerGeneration};

/// The local run a failed fan write by `owner` must stop and latch; `None` for any
/// other owner.
#[must_use]
pub const fn faulted_local_generation(owner: FanControllerOwner) -> Option<WorkerGeneration> {
    match owner {
        FanControllerOwner::Local(generation) => Some(generation),
        FanControllerOwner::Campaign
        | FanControllerOwner::Lease(_)
        | FanControllerOwner::Idle(_) => None,
    }
}

/// Upstream's message for the fan controller `context` whose write failed.
#[must_use]
pub fn hardware_fault_message(context: &str) -> String {
    format!("Fan Control Failed ({context})")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::revocation::GenerationGate;

    #[test]
    fn a_local_runs_failed_write_names_its_generation() {
        // Arrange
        let gate = GenerationGate::new();
        let generation = gate.begin_local().expect("local generation");
        // Act / Assert
        assert_eq!(
            faulted_local_generation(FanControllerOwner::Local(generation)),
            Some(generation)
        );
    }

    #[test]
    fn a_leases_failed_write_latches_nothing() {
        // Arrange
        let gate = GenerationGate::new();
        let generation = gate.begin_link(0).expect("link");
        // Act / Assert
        assert_eq!(
            faulted_local_generation(FanControllerOwner::Lease(generation)),
            None
        );
    }

    #[test]
    fn campaign_and_idle_writes_latch_nothing() {
        // Arrange
        let gate = GenerationGate::new();
        let idle = FanControllerOwner::Idle(gate.grant_idle_fan());
        // Act / Assert
        assert_eq!(faulted_local_generation(FanControllerOwner::Campaign), None);
        assert_eq!(faulted_local_generation(idle), None);
    }

    #[test]
    fn the_message_matches_upstream() {
        // Arrange / Act / Assert
        assert_eq!(hardware_fault_message("Auto"), "Fan Control Failed (Auto)");
    }
}
