//! Source guards for the local mining owner (ADR-0038) while it stays switched off.
const LOCAL_OWNER_SOURCE: &str = include_str!("production_mining_session/local_owner.rs");
const OWNER_LOOP_SOURCE: &str = include_str!("production_mining_session/owner_loop.rs");
const BWG_SOURCE: &str = include_str!("production_mining_session/bwg.rs");
const SESSION_SOURCE: &str = include_str!("production_mining_session.rs");
const FAN_RUNTIME_SOURCE: &str = include_str!("fan_controller_runtime.rs");

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
