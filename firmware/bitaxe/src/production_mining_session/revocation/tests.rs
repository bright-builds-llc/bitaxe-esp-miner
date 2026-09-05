use super::*;

#[test]
fn queued_voltage_rechecks_generation_after_a_contended_owner_lock() {
    // Arrange
    let gate = std::sync::Arc::new(GenerationGate::new());
    let generation = gate.begin_link(0).expect("generation");
    assert!(gate.admit_budget(generation, u64::MAX));
    assert!(gate.activate(generation));
    let queued = gate.stamp(Some(generation));
    let owner = std::sync::Arc::new(std::sync::Mutex::new(()));
    let guard = owner.lock().expect("owner lock");
    let worker_gate = gate.clone();
    let worker_owner = owner.clone();
    let worker = std::thread::spawn(move || {
        let _guard = worker_owner.lock().expect("queued owner acquisition");
        worker_gate.permits_work(queued)
    });
    // Act
    assert!(gate.revoke_at(generation, 2_800));
    drop(guard);
    // Assert
    assert!(!worker.join().expect("worker exits"));
}

#[test]
fn old_unleased_voltage_cannot_run_after_a_worker_campaign_finishes() {
    // Arrange
    let gate = GenerationGate::new();
    let queued = gate.stamp(None);
    let worker = gate.begin_link(0).expect("generation");
    assert!(gate.admit_budget(worker, u64::MAX));
    assert!(gate.activate(worker));
    // Act
    gate.revoke_at(worker, 100);
    gate.finish_shutdown(worker);
    // Assert
    assert!(gate.permits(None));
    assert!(!gate.permits_work(queued));
}

#[test]
fn fresh_heartbeat_does_not_override_stalled_safety_observations() {
    // Arrange
    let gate = GenerationGate::new();
    let worker = gate.begin_link(0).expect("generation");
    assert!(gate.admit_budget(worker, u64::MAX));
    assert!(gate.activate(worker));
    gate.note_fan_proof(worker, 200);
    // Act
    assert!(gate.heartbeat(worker, 1_500));
    gate.check_deadline(1_500);
    // Assert
    assert!(!gate.permits(Some(worker)));
}

#[test]
fn zero_rpm_revokes_after_fan_proof_without_waiting_for_the_production_inbox() {
    // Arrange
    let gate = GenerationGate::new();
    let worker = gate.begin_link(0).expect("generation");
    assert!(gate.admit_budget(worker, u64::MAX));
    assert!(gate.activate(worker));
    gate.check_safety(true, false, 100);
    assert!(gate.permits(Some(worker)));
    gate.note_fan_proof(worker, 200);
    // Act
    gate.check_safety(true, false, 300);
    // Assert
    assert!(!gate.permits(Some(worker)));
}

#[test]
fn signed_lease_expiry_closes_with_fresh_heartbeats() {
    // Arrange
    let gate = GenerationGate::new();
    let generation = gate.begin_link(0).expect("generation");
    assert!(gate.admit_budget(generation, u64::MAX));
    assert!(gate.set_lease_deadline(generation, 500));
    assert!(gate.activate_at(generation, 0));
    // Act
    assert!(gate.heartbeat(generation, 499));
    gate.check_deadline(500);
    // Assert
    assert_eq!(gate.maybe_revoked(), Some(generation));
    assert_eq!(
        gate.timing(500).expect("timing").maybe_gate_closed_ms,
        Some(500)
    );
}

#[test]
fn mining_duration_excludes_preparation_and_stops_before_cooling() {
    // Arrange
    let gate = GenerationGate::new();
    let generation = gate.begin_link(0).expect("generation");
    assert!(gate.admit_budget(generation, 180_000));
    assert!(gate.activate_at(generation, 100));
    assert_eq!(gate.timing(400).expect("preparing").active_ms, 0);
    gate.note_first_dispatch(Some(generation), 500);
    // Act
    gate.check_deadline(2_800);
    gate.note_shutdown(generation, 1, 2_850);
    assert_eq!(gate.timing(3_500).expect("ramping down").active_ms, 3_000);
    gate.note_asic_halted(4_000);
    gate.finish_shutdown(generation);
    let timing = gate.timing(122_850).expect("retained timing");
    // Assert
    assert_eq!(timing.active_ms, 3_500);
    assert_eq!(timing.generation_elapsed_ms, 3_900);
    assert_eq!(timing.maybe_gate_closed_ms, Some(2_800));
    assert_eq!(timing.maybe_shutdown_started_ms, Some(2_850));
    assert!(timing.shutdown_complete);
}

#[test]
fn idle_reconnect_cannot_replace_the_previous_started_generations_proof() {
    // Arrange
    let gate = GenerationGate::new();
    let generation = gate.begin_link(0).expect("generation");
    assert!(gate.admit_budget(generation, 180_000));
    assert!(gate.activate_at(generation, 100));
    gate.note_first_dispatch(Some(generation), 500);
    gate.publish_counts(generation, 1, 2, 3);
    gate.revoke_at(generation, 2_000);
    gate.note_shutdown(generation, 1, 2_050);
    gate.note_asic_halted(3_000);
    gate.finish_shutdown(generation);
    // Act
    let idle = gate.begin_link(130_000).expect("idle reconnect");
    gate.revoke_at(idle, 130_500);
    gate.finish_shutdown(idle);
    let retained = gate.timing(140_000).expect("last started proof");
    // Assert
    assert_eq!(retained.generation, generation.raw());
    assert_eq!(retained.active_ms, 2_500);
    assert_eq!(retained.last_valid_heartbeat_ms, 0);
    assert_eq!(retained.maybe_gate_closed_ms, Some(2_000));
    assert_eq!(retained.accepted, 1);
    assert_eq!(retained.rejected, 2);
    assert_eq!(retained.nonce_work_correlations, 3);
    assert!(retained.shutdown_complete);
}

#[test]
fn heartbeat_cutoff_revokes_without_consuming_a_queue_slot() {
    // Arrange
    let gate = GenerationGate::new();
    let generation = gate.begin_link(100).expect("fresh generation");
    assert!(gate.admit_budget(generation, 180_100));
    assert!(gate.activate(generation));
    let permit = gate.stamp(Some(generation));
    // Act
    gate.check_deadline(2_900);
    // Assert
    assert!(!gate.permits_work(permit));
    assert_eq!(gate.maybe_revoked(), Some(generation));
}

#[test]
fn late_start_and_heartbeat_cannot_reopen_a_revoked_generation() {
    // Arrange
    let gate = GenerationGate::new();
    let generation = gate.begin_link(0).expect("generation");
    assert!(gate.admit_budget(generation, u64::MAX));
    assert!(gate.activate(generation));
    // Act
    gate.revoke(generation);
    // Assert
    assert!(!gate.activate(generation));
    assert!(!gate.heartbeat(generation, 4_000));
    assert!(gate.begin_link(4_000).is_none());
}

#[test]
fn old_queued_work_stays_invalid_after_a_new_link() {
    // Arrange
    let gate = GenerationGate::new();
    let old = gate.begin_link(0).expect("generation");
    assert!(gate.admit_budget(old, 180_000));
    assert!(gate.activate(old));
    let queued = gate.stamp(Some(old));
    gate.revoke(old);
    gate.finish_shutdown(old);
    // Act
    let new = gate.begin_link(4_000).expect("new generation");
    assert!(gate.admit_budget(new, 184_000));
    assert!(gate.activate(new));
    // Assert
    assert!(!gate.permits_work(queued));
    assert!(!gate.revoke(old));
    assert!(gate.permits(Some(new)));
}

#[test]
fn failed_open_cannot_refresh_an_existing_links_deadline() {
    // Arrange
    let gate = GenerationGate::new();
    let generation = gate.begin_link(0).expect("generation");
    // Act
    assert!(gate.begin_link(2_000).is_none());
    gate.check_deadline(2_800);
    // Assert
    assert!(!gate.is_live(generation));
    assert!(gate.permits(None));
}

#[test]
fn idle_link_connect_and_loss_leave_ordinary_mining_admitted() {
    // Arrange
    let gate = GenerationGate::new();
    let ordinary = gate.stamp(None);
    let idle = gate.begin_link(0).expect("idle link");
    assert!(gate.permits_work(ordinary));
    // Act
    gate.check_deadline(2_800);
    // Assert
    assert!(!gate.is_live(idle));
    assert!(gate.permits_work(ordinary));
    assert_eq!(gate.maybe_revoked(), None);
}

#[test]
fn active_worker_ownership_excludes_unleased_effects() {
    let gate = GenerationGate::new();
    let worker = gate.begin_link(0).expect("worker link");
    assert!(gate.admit_budget(worker, u64::MAX));
    assert!(gate.activate(worker));
    assert!(!gate.permits(None));
    gate.revoke_at(worker, 100);
    assert!(!gate.permits(None));
}

#[test]
fn renewed_heartbeat_cannot_extend_the_reserved_campaign_window() {
    // Arrange
    let gate = GenerationGate::new();
    let generation = gate.begin_link(0).expect("generation");
    assert!(gate.admit_budget(generation, 30_000));
    assert!(gate.activate(generation));
    gate.note_first_dispatch(Some(generation), 0);
    let stop_at = 30_000 - super::super::shutdown_budget::PRE_RESET_BOUND_MS;
    // Act
    for now in (1000..stop_at).step_by(1000) {
        assert!(gate.heartbeat(generation, u64::from(now)));
    }
    assert!(gate.heartbeat(generation, u64::from(stop_at - 1)));
    assert!(!gate.admit_budget(generation, 240_000));
    gate.check_deadline(u64::from(stop_at));
    // Assert
    assert!(!gate.permits(Some(generation)));
}

#[test]
fn active_budget_is_not_spent_on_preparation_and_cannot_omit_shutdown_reserve() {
    // Arrange
    let gate = GenerationGate::new();
    let worker = gate.begin_link(0).expect("link");
    assert!(!gate.admit_budget(worker, 1_000));
    assert!(!gate.activate(worker));
    assert!(gate.admit_budget(worker, 30_000));
    assert!(gate.activate_at(worker, 0));
    // Act
    for now in (1000..=10_000).step_by(1000) {
        gate.heartbeat(worker, now);
        gate.check_deadline(now);
    }
    assert!(gate.permits(Some(worker)));
    assert_eq!(
        gate.timing(10_000)
            .expect("preparing")
            .work_gate_remaining_ms,
        None
    );
    assert!(gate.begin_dispatch(gate.stamp(Some(worker)), 10_000));
    // Assert
    let timing = gate.timing(10_000).expect("first work");
    assert_eq!(timing.active_ms, 0);
    assert_eq!(timing.active_limit_ms, Some(30_000));
    assert_eq!(timing.work_gate_remaining_ms, Some(14_450));
}

#[test]
fn full_effect_queue_does_not_delay_revocation() {
    // Arrange
    let gate = GenerationGate::new();
    let generation = gate.begin_link(0).expect("generation");
    assert!(gate.admit_budget(generation, u64::MAX));
    assert!(gate.activate(generation));
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    sender
        .try_send(gate.stamp(Some(generation)))
        .expect("queued work");
    assert!(sender.try_send(gate.stamp(Some(generation))).is_err());
    // Act
    gate.check_deadline(2_800);
    // Assert
    assert!(!gate.permits_work(receiver.try_recv().expect("queued work remains")));
}

#[test]
fn budget_admission_is_required_before_activation() {
    let gate = GenerationGate::new();
    let generation = gate.begin_link(0).expect("generation");
    assert!(!gate.activate(generation));
}

#[test]
fn heartbeat_reason_survives_later_cleanup_and_idle_reconnection() {
    // Arrange
    let gate = GenerationGate::new();
    let generation = gate.begin_link(0).expect("link");
    assert!(gate.admit_budget(generation, u64::MAX));
    assert!(gate.activate_at(generation, 0));
    // Act
    gate.check_deadline(2_800);
    assert!(!gate.revoke_reason_at(generation, 2_900, RevocationReason::RestorationRequested));
    gate.finish_shutdown(generation);
    let idle = gate.begin_link(3_000).expect("fresh idle link");
    gate.revoke_reason_at(idle, 3_100, RevocationReason::LinkClosed);
    // Assert
    let timing = gate.timing(3_200).expect("retained started generation");
    assert_eq!(timing.revocation_reason, RevocationReason::HeartbeatTimeout);
    assert_eq!(timing.maybe_gate_closed_ms, Some(2_800));
}

#[test]
fn unsafe_observation_is_not_misclassified_as_later_heartbeat_loss() {
    // Arrange
    let gate = GenerationGate::new();
    let generation = gate.begin_link(0).expect("link");
    assert!(gate.admit_budget(generation, u64::MAX));
    assert!(gate.activate_at(generation, 0));
    // Act
    gate.check_safety(false, true, 500);
    gate.check_deadline(2_800);
    // Assert
    let timing = gate.timing(3_000).expect("started generation");
    assert_eq!(
        timing.revocation_reason,
        RevocationReason::UnsafeObservation
    );
    assert_eq!(timing.maybe_gate_closed_ms, Some(500));
}

#[test]
fn a_signed_lease_cutoff_records_its_own_reason_with_fresh_heartbeats() {
    // Arrange
    let gate = GenerationGate::new();
    let generation = gate.begin_link(0).expect("link");
    assert!(gate.admit_budget(generation, u64::MAX));
    assert!(gate.activate_at(generation, 0));
    assert!(gate.set_lease_deadline(generation, 1_000));
    assert!(gate.heartbeat(generation, 900));
    // Act
    gate.check_deadline(1_000);
    // Assert
    assert_eq!(
        gate.timing(1_001).expect("timing").revocation_reason,
        RevocationReason::LeaseOrBudgetExpired
    );
}
