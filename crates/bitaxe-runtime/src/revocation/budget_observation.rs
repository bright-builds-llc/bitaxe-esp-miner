use super::*;
impl GenerationGate {
    #[cfg(test)]
    pub fn note_first_dispatch(&self, maybe_generation: Option<WorkerGeneration>, now_ms: u64) {
        let _ = self.begin_dispatch(self.stamp(maybe_generation), now_ms);
    }

    /// Captures a newly armed budget using the caller's original full uptime.
    /// No wrapped timestamp is expanded into an invented later epoch.
    pub fn begin_dispatch_observed(
        &self,
        permit: WorkPermit,
        now_ms: u64,
    ) -> (bool, Option<(WorkerGeneration, u64, u32)>) {
        let before = permit
            .maybe_generation
            .and_then(|g| self.maybe_dispatch_budget(g));
        let admitted = self.begin_dispatch(permit, now_ms);
        let observed = if before.is_none() {
            permit.maybe_generation.and_then(|g| {
                self.maybe_dispatch_budget(g)
                    .filter(|(origin, _)| *origin == now_ms as u32)
                    .map(|(_, limit)| (g, now_ms, limit))
            })
        } else {
            None
        };
        (admitted, observed)
    }
    fn maybe_dispatch_budget(&self, generation: WorkerGeneration) -> Option<(u32, u32)> {
        if self.first_dispatch_generation.load(Ordering::Acquire) != generation.0
            || self.budget_armed_generation.load(Ordering::Acquire) != generation.0
        {
            return None;
        }
        let value = (
            self.first_dispatch_ms.load(Ordering::Acquire),
            self.budget_limit_ms.load(Ordering::Acquire),
        );
        (self.first_dispatch_generation.load(Ordering::Acquire) == generation.0
            && self.budget_generation.load(Ordering::Acquire) == generation.0)
            .then_some(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn arming_observation_preserves_full_epoch_and_is_once_even_without_uart_success() {
        // Arrange
        let gate = GenerationGate::new();
        let epoch = u64::from(u32::MAX) + 2000;
        let generation = gate.begin_link(epoch).expect("link");
        assert!(gate.admit_budget(generation, 180_000));
        assert!(gate.activate_at(generation, epoch));
        let permit = gate.stamp(Some(generation));
        // Act: no successful-dispatch callback is made; UART could fail next.
        let first = gate.begin_dispatch_observed(permit, epoch + 1);
        let second = gate.begin_dispatch_observed(permit, epoch + 2);
        // Assert
        assert!(first.0);
        assert_eq!(first.1, Some((generation, epoch + 1, 180_000)));
        assert!(second.0);
        assert_eq!(second.1, None);
        assert_eq!(gate.timing(epoch + 2).expect("timing").work_dispatched, 0);
    }
    #[test]
    fn revoked_or_stale_dispatch_never_creates_arming_metadata() {
        // Arrange
        let gate = GenerationGate::new();
        let generation = gate.begin_link(1000).expect("link");
        assert!(gate.admit_budget(generation, 180_000));
        assert!(gate.activate_at(generation, 1000));
        let permit = gate.stamp(Some(generation));
        gate.revoke_at(generation, 1100);
        // Act / Assert
        assert_eq!(gate.begin_dispatch_observed(permit, 1101), (false, None));
    }
}

#[cfg(test)]
mod reconnect_tests {
    use super::*;
    #[test]
    fn fresh_idle_link_preserves_old_heartbeat_fault_and_completed_shutdown_atoms() {
        // Arrange
        let gate = GenerationGate::new();
        let work = gate.begin_link(1000).expect("work link");
        assert!(gate.admit_budget(work, 180_000));
        assert!(gate.activate_at(work, 1000));
        assert!(gate.begin_dispatch_observed(gate.stamp(Some(work)), 1100).0);
        assert!(gate.heartbeat(work, 1200));
        gate.publish_counts(work, 1, 0, 1);
        gate.check_deadline(4000);
        gate.note_shutdown(work, 1, 4080);
        gate.note_asic_halted(5000);
        gate.note_shutdown(work, 8, 7000);
        gate.finish_shutdown(work);
        // Act
        let fresh = gate
            .begin_link(149_000)
            .expect("fresh admission after wait");
        assert_ne!(fresh, work);
        assert!(gate.heartbeat(fresh, 150_000));
        let retained = gate
            .timing(150_000)
            .expect("retained actual work generation");
        // Assert
        assert_eq!(retained.generation, work.raw());
        assert_eq!(
            retained.revocation_reason,
            RevocationReason::HeartbeatTimeout
        );
        assert_eq!(retained.last_valid_heartbeat_ms, 1200);
        assert_eq!(retained.maybe_gate_closed_ms, Some(4000));
        assert_eq!(retained.maybe_shutdown_started_ms, Some(4080));
        assert_eq!(retained.active_limit_ms, Some(180_000));
        assert_eq!(retained.shutdown_stage, 8);
        assert!(retained.shutdown_complete);
        assert_eq!(retained.accepted, 1);
    }
}
