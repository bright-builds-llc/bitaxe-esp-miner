//! Who may drive the fan controller's duty (ADR-0037 decision 6, ADR-0038). While a
//! Gate lease mines, the controller follows the device's fan settings. Every write
//! names that lease's generation and is applied only while the generation is
//! active. Every lease safe-stop revokes the generation before its first fan
//! command, so a late write is refused and safe-stop and cooling own the fan.
//! The work epoch is deliberately not checked: pool reconnects and failovers
//! block work without ending the lease, and the fan must stay controlled.
//!
//! While idle, the controller runs upstream's idle duties under an epoch the
//! production owner grants and withdraws. It withdraws before the cooling review
//! and before any preparation, so a write queued earlier never overrides their
//! 100% fan, and `permits(None)` refuses idle writes during any lease or shutdown.
use super::*;

impl GenerationGate {
    /// Whether a controller write of `duty_percent` may reach the fan now. The
    /// campaign path keeps the rule that a duty below 100% needs no live lease.
    pub fn permits_controller_fan(&self, owner: FanControllerOwner, duty_percent: u8) -> bool {
        match owner {
            FanControllerOwner::Lease(generation) | FanControllerOwner::Local(generation) => {
                self.permits(Some(generation))
            }
            FanControllerOwner::Campaign => duty_percent >= 100 || self.permits(None),
            FanControllerOwner::Idle(epoch) => {
                epoch.0 & 1 == 1
                    && self.idle_fan_epoch.load(Ordering::Acquire) == epoch.0
                    && self.permits(None)
            }
        }
    }

    /// Grants idle fan control, or returns the grant already in force. An odd
    /// counter value is a granted epoch; an even one means withdrawn.
    pub fn grant_idle_fan(&self) -> IdleFanEpoch {
        loop {
            let current = self.idle_fan_epoch.load(Ordering::Acquire);
            if current & 1 == 1 {
                return IdleFanEpoch(current);
            }
            let next = current.wrapping_add(1);
            if self
                .idle_fan_epoch
                .compare_exchange(current, next, Ordering::AcqRel, Ordering::Acquire)
                .is_ok()
            {
                return IdleFanEpoch(next);
            }
        }
    }

    /// Ends any idle grant; its writes are refused from now on.
    pub fn withdraw_idle_fan(&self) {
        loop {
            let current = self.idle_fan_epoch.load(Ordering::Acquire);
            if current & 1 == 0 {
                return;
            }
            if self
                .idle_fan_epoch
                .compare_exchange(
                    current,
                    current.wrapping_add(1),
                    Ordering::AcqRel,
                    Ordering::Acquire,
                )
                .is_ok()
            {
                return;
            }
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

    #[test]
    fn a_granted_idle_epoch_may_set_the_fan_while_no_lease_exists() {
        // Arrange
        let gate = GenerationGate::new();
        let epoch = gate.grant_idle_fan();
        // Act
        let permitted = gate.permits_controller_fan(FanControllerOwner::Idle(epoch), 30);
        // Assert
        assert!(permitted);
    }

    #[test]
    fn an_idle_write_queued_before_a_withdrawal_is_refused() {
        // Arrange: the cooling review withdraws before its 100% command.
        let gate = GenerationGate::new();
        let epoch = gate.grant_idle_fan();
        gate.withdraw_idle_fan();
        // Act
        let permitted = gate.permits_controller_fan(FanControllerOwner::Idle(epoch), 30);
        // Assert
        assert!(!permitted);
    }

    #[test]
    fn a_new_grant_never_revives_an_old_epoch() {
        // Arrange
        let gate = GenerationGate::new();
        let stale = gate.grant_idle_fan();
        gate.withdraw_idle_fan();
        let current = gate.grant_idle_fan();
        // Act
        let stale_permitted = gate.permits_controller_fan(FanControllerOwner::Idle(stale), 30);
        let current_permitted = gate.permits_controller_fan(FanControllerOwner::Idle(current), 30);
        // Assert
        assert_eq!((stale_permitted, current_permitted), (false, true));
    }

    #[test]
    fn idle_control_is_refused_while_a_lease_mines_or_shuts_down() {
        // Arrange
        let gate = GenerationGate::new();
        let worker = gate.begin_link(0).expect("link");
        assert!(gate.admit_budget(worker, 180_000));
        assert!(gate.activate_at(worker, 0));
        let epoch = gate.grant_idle_fan();
        // Act
        let while_active = gate.permits_controller_fan(FanControllerOwner::Idle(epoch), 30);
        gate.revoke_at(worker, 200);
        let while_shutting_down = gate.permits_controller_fan(FanControllerOwner::Idle(epoch), 30);
        // Assert
        assert_eq!((while_active, while_shutting_down), (false, false));
    }

    #[test]
    fn idle_control_works_beside_an_idle_gate_link() {
        // Arrange: a connected Gate with no lease leaves the device idle.
        let gate = GenerationGate::new();
        gate.begin_link(0).expect("link");
        let epoch = gate.grant_idle_fan();
        // Act
        let permitted = gate.permits_controller_fan(FanControllerOwner::Idle(epoch), 30);
        // Assert
        assert!(permitted);
    }

    #[test]
    fn idle_control_is_refused_from_reservation_through_activation() {
        // Arrange: the cooling review runs while the reservation holds.
        let gate = GenerationGate::new();
        let worker = gate.begin_link(0).expect("link");
        let epoch = gate.grant_idle_fan();
        assert!(gate.begin_reservation(worker));
        // Act
        let while_reserved = gate.permits_controller_fan(FanControllerOwner::Idle(epoch), 30);
        gate.release_unbudgeted_reservation(worker);
        assert!(gate.admit_budget(worker, 180_000));
        let while_budgeted = gate.permits_controller_fan(FanControllerOwner::Idle(epoch), 30);
        // Assert
        assert_eq!((while_reserved, while_budgeted), (false, false));
    }

    #[test]
    fn idle_control_is_refused_while_a_diagnostic_holds_the_gate() {
        // Arrange
        let gate = GenerationGate::new();
        let worker = gate.begin_link(0).expect("link");
        assert!(gate.claim_diagnostic(worker, 10_000));
        let epoch = gate.grant_idle_fan();
        // Act
        let permitted = gate.permits_controller_fan(FanControllerOwner::Idle(epoch), 30);
        // Assert
        assert!(!permitted);
    }

    #[test]
    fn an_active_local_generation_may_set_any_duty() {
        // Arrange
        let gate = GenerationGate::new();
        let local = gate.begin_local().expect("local generation");
        assert!(gate.admit_budget(local, u64::MAX));
        assert!(gate.activate_at(local, 0));
        // Act
        let permitted = gate.permits_controller_fan(FanControllerOwner::Local(local), 25);
        // Assert
        assert!(permitted);
    }

    #[test]
    fn a_revoked_local_generation_can_no_longer_set_the_fan() {
        // Arrange: its safe-stop revokes before the first fan command.
        let gate = GenerationGate::new();
        let local = gate.begin_local().expect("local generation");
        assert!(gate.admit_budget(local, u64::MAX));
        assert!(gate.activate_at(local, 0));
        gate.revoke_at(local, 100);
        // Act
        let permitted = gate.permits_controller_fan(FanControllerOwner::Local(local), 100);
        // Assert
        assert!(!permitted);
    }
}
