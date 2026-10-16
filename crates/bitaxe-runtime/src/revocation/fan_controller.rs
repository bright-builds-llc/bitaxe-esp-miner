//! Who may drive the fan controller's duty (ADR-0037 decision 6). While a Gate
//! lease mines, the controller follows the device's fan settings. Every write
//! names that lease's generation and is applied only while the generation is
//! active. Every lease safe-stop revokes the generation before its first fan
//! command, so a late write is refused and safe-stop and cooling own the fan.
//! The work epoch is deliberately not checked: pool reconnects and failovers
//! block work without ending the lease, and the fan must stay controlled.
use super::*;

impl GenerationGate {
    /// Whether a controller write of `duty_percent` may reach the fan now. The
    /// campaign path keeps the rule that a duty below 100% needs no live lease.
    pub fn permits_controller_fan(&self, owner: FanControllerOwner, duty_percent: u8) -> bool {
        match owner {
            FanControllerOwner::Lease(generation) => self.permits(Some(generation)),
            FanControllerOwner::Campaign => duty_percent >= 100 || self.permits(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn active_lease() -> (GenerationGate, WorkerGeneration) {
        let gate = GenerationGate::new();
        let worker = gate.begin_link(0).expect("link");
        assert!(gate.admit_budget(worker, 180_000));
        assert!(gate.activate_at(worker, 0));
        gate.heartbeat(worker, 100);
        (gate, worker)
    }

    #[test]
    fn an_active_lease_may_set_any_duty() {
        // Arrange
        let (gate, worker) = active_lease();
        // Act
        let permitted = gate.permits_controller_fan(FanControllerOwner::Lease(worker), 25);
        // Assert
        assert!(permitted);
    }

    #[test]
    fn a_revoked_lease_can_no_longer_set_the_fan() {
        // Arrange: every lease safe-stop revokes before its first fan command.
        let (gate, worker) = active_lease();
        gate.revoke_at(worker, 200);
        // Act
        let permitted = gate.permits_controller_fan(FanControllerOwner::Lease(worker), 100);
        // Assert
        assert!(!permitted);
    }

    #[test]
    fn a_pool_reconnect_keeps_the_lease_controlling_the_fan() {
        // Arrange: a reconnect or failover blocks work without ending the lease.
        let (gate, worker) = active_lease();
        gate.block_work();
        // Act
        let permitted = gate.permits_controller_fan(FanControllerOwner::Lease(worker), 25);
        // Assert
        assert!(permitted);
    }

    #[test]
    fn another_generation_cannot_drive_the_fan() {
        // Arrange
        let (gate, worker) = active_lease();
        let stale = WorkerGeneration(worker.0.wrapping_add(1 << GENERATION_SHIFT));
        // Act
        let permitted = gate.permits_controller_fan(FanControllerOwner::Lease(stale), 25);
        // Assert
        assert!(!permitted);
    }

    #[test]
    fn the_campaign_path_cannot_lower_the_fan_under_a_live_lease() {
        // Arrange
        let (gate, _worker) = active_lease();
        // Act
        let lowered = gate.permits_controller_fan(FanControllerOwner::Campaign, 25);
        let full = gate.permits_controller_fan(FanControllerOwner::Campaign, 100);
        // Assert
        assert_eq!((lowered, full), (false, true));
    }
}

#[cfg(test)]
mod campaign_tests {
    use super::*;

    #[test]
    fn the_campaign_path_may_lower_the_fan_without_a_lease() {
        // Arrange
        let gate = GenerationGate::new();
        // Act
        let permitted = gate.permits_controller_fan(FanControllerOwner::Campaign, 25);
        // Assert
        assert!(permitted);
    }
}
