//! Source guards for the local mining owner (ADR-0038) while it stays switched off.
const LOCAL_OWNER_SOURCE: &str = include_str!("production_mining_session/local_owner.rs");
const OWNER_LOOP_SOURCE: &str = include_str!("production_mining_session/owner_loop.rs");
const BWG_SOURCE: &str = include_str!("production_mining_session/bwg.rs");
const SESSION_SOURCE: &str = include_str!("production_mining_session.rs");
const FAN_RUNTIME_SOURCE: &str = include_str!("fan_controller_runtime.rs");
const OVERHEAT_SOURCE: &str = include_str!("production_mining_session/local_owner/overheat.rs");

#[test]
fn local_mining_stays_off_until_the_crash_loop_guard_lands() {
    // Arrange / Act / Assert
    assert!(LOCAL_OWNER_SOURCE.contains("pub(crate) const LOCAL_MINING_ENABLED: bool = false;"));
}

#[test]
fn the_local_tick_returns_before_any_effect_while_switched_off() {
    // Arrange
    let (_, tick) = LOCAL_OWNER_SOURCE
        .split_once("pub(super) fn service_local_mining(")
        .expect("local tick");
    // Act
    let guard = tick.find("if !LOCAL_MINING_ENABLED {");
    let snapshot = tick.find("session.snapshot()");
    // Assert: the switch is checked before the tick reads or changes anything.
    assert!(guard.is_some() && snapshot.is_some() && guard < snapshot);
}

#[test]
fn the_local_tick_runs_after_revocation_and_before_the_inbox() {
    // Arrange
    let revoked = OWNER_LOOP_SOURCE.find("revocation::maybe_revoked()");
    let local = OWNER_LOOP_SOURCE.find("adapter.service_local_mining(&session, revocation_now)");
    let inbox = OWNER_LOOP_SOURCE.find("receiver.recv_timeout(wait)");
    // Act / Assert
    assert!(revoked < local && local < inbox && revoked.is_some());
}

#[test]
fn a_lease_start_is_refused_beside_a_local_run() {
    // Arrange / Act / Assert
    assert!(BWG_SOURCE.contains("|| self.local.maybe_run.is_some()"));
}

#[test]
fn a_prepared_local_run_owns_the_fan_controller() {
    // Arrange / Act / Assert
    assert!(SESSION_SOURCE.contains("revocation::FanControllerOwner::Local("));
}

#[test]
fn only_a_real_fan_write_failure_latches_the_hardware_fault() {
    // Arrange
    let (_, failure_arm) = FAN_RUNTIME_SOURCE
        .split_once("state.controller.record_apply_failure(now_ms);")
        .expect("failure arm");
    // Act
    let latch = failure_arm.find("local_owner::note_fan_control_failure(");
    let gate = failure_arm.find("if outcome == SafetyActuationRequestOutcome::HardwareWriteFailed {");
    // Assert: a refusal, timeout or full queue is not a hardware fault.
    assert!(gate.is_some() && latch.is_some() && gate < latch);
}

#[test]
fn the_latch_covers_local_runs_only_and_holds_until_reboot() {
    // Arrange / Act / Assert
    assert!(LOCAL_OWNER_SOURCE
        .contains("let Some(generation) = faulted_local_generation(owner) else {"));
    assert!(LOCAL_OWNER_SOURCE.contains("hardware_fault: maybe_hardware_fault_context().is_some(),"));
}

#[test]
fn an_overheat_ending_enters_upstreams_recovery() {
    // Arrange
    let (_, after_ending) = LOCAL_OWNER_SOURCE
        .split_once("if maybe_ending == Some(RunEnding::Overheat) {")
        .expect("overheat ending");
    // Act / Assert
    assert!(after_ending.trim_start().starts_with("self.enter_overheat(run.lease.profile(), now_ms);"));
}

#[test]
fn an_unsaved_lowered_pair_keeps_the_asic_down() {
    // Arrange / Act / Assert: the next run would otherwise read the pair that overheated.
    assert!(OVERHEAT_SOURCE
        .contains("self.recovery.holds_power_down() || self.maybe_unsaved_pair.is_some()"));
    assert!(LOCAL_OWNER_SOURCE.contains("overheat_holds: self.local.overheat.holds(),"));
}

#[test]
fn a_stop_and_the_guards_hold_pause_the_device_for_the_operator() {
    // Arrange / Act / Assert
    assert!(OVERHEAT_SOURCE.contains("self.pause_by_device();"));
    let (_, guard_hold) = LOCAL_OWNER_SOURCE
        .split_once("if reason == LocalHoldReason::RestartGuardHeld && !self.local.guard_paused {")
        .expect("guard hold");
    assert!(guard_hold.contains("self.pause_by_device();"));
    // The session wakes to stop, and the resume edge is measured from Paused.
    let (_, pause) = LOCAL_OWNER_SOURCE
        .split_once("fn pause_by_device(&mut self) {")
        .expect("device pause");
    assert!(pause.contains("ProductionSessionWakeup::OperatorIntentChanged"));
    assert!(pause.contains("self.local.last_requested_intent = Some(MiningOperatorIntent::Paused);"));
}

#[test]
fn every_published_snapshot_carries_the_local_hold() {
    // Arrange / Act / Assert
    assert!(SESSION_SOURCE.contains("self.local.label_hold(&mut snapshot);"));
}
