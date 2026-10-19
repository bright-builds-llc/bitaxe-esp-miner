//! Source guards for the local mining owner (ADR-0038).
const LOCAL_OWNER_SOURCE: &str = include_str!("production_mining_session/local_owner.rs");
const OWNER_LOOP_SOURCE: &str = include_str!("production_mining_session/owner_loop.rs");
const BWG_SOURCE: &str = include_str!("production_mining_session/bwg.rs");
const SESSION_SOURCE: &str = include_str!("production_mining_session.rs");
const FAN_RUNTIME_SOURCE: &str = include_str!("fan_controller_runtime.rs");
const OVERHEAT_SOURCE: &str = include_str!("production_mining_session/local_owner/overheat.rs");
const LINK_SOURCE: &str = include_str!("bwg_worker_usb/link.rs");
const STATUS_SOURCE: &str = include_str!("production_mining_session/status_publication.rs");

#[test]
fn local_mining_is_on_with_the_crash_loop_guard() {
    // Arrange / Act / Assert
    assert!(LOCAL_OWNER_SOURCE.contains("pub(crate) const LOCAL_MINING_ENABLED: bool = true;"));
    assert!(include_str!("startup.rs").contains("crate::boot_guard::boot_preference("));
}

#[test]
fn the_local_tick_checks_the_switch_before_any_effect() {
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
    assert!(OVERHEAT_SOURCE.contains("self.pause_by_device(DevicePauseReason::Overheat);"));
    let (_, guard_hold) = LOCAL_OWNER_SOURCE
        .split_once("if reason == LocalHoldReason::RestartGuardHeld && !self.local.guard_paused {")
        .expect("guard hold");
    assert!(guard_hold.contains("self.pause_by_device(DevicePauseReason::RestartGuard);"));
    // The session wakes to stop, and the resume edge is measured from Paused.
    let (_, pause) = LOCAL_OWNER_SOURCE
        .split_once("fn pause_by_device(&mut self, reason: DevicePauseReason) {")
        .expect("device pause");
    assert!(pause.contains("ProductionSessionWakeup::OperatorIntentChanged"));
    assert!(pause.contains("self.local.resume_edge.note_device_pause();"));
    assert!(pause.contains("crate::device_pause::set(reason);"));
}

#[test]
fn every_published_snapshot_carries_the_local_hold() {
    // Arrange / Act / Assert
    assert!(SESSION_SOURCE.contains("self.local.label_hold(&mut snapshot);"));
}

#[test]
fn every_hello_and_heartbeat_reaches_the_local_owner() {
    // Arrange
    let (_, begin) = LINK_SOURCE
        .split_once("fn begin_generation(&self, now_ms: u64)")
        .expect("begin generation");
    let (_, heartbeat) = LINK_SOURCE
        .split_once("fn heartbeat_generation(")
        .expect("heartbeat");
    // Act: a Hello reports whether the gate admitted it.
    let decided = begin.find("let maybe_generation = revocation::begin_link(now_ms);");
    let reported = begin.find("local_owner::note_gate_hello(");
    // Assert
    assert!(decided.is_some() && reported.is_some() && decided < reported);
    assert!(begin.contains("maybe_generation.is_some(),"));
    assert!(heartbeat
        .contains("crate::production_mining_session::local_owner::note_gate_link_activity(now_ms);"));
}

#[test]
fn a_held_link_stops_a_run_through_the_core_and_holds_local_mining() {
    // Arrange
    let (_, handover) = LOCAL_OWNER_SOURCE
        .split_once("fn hand_over_to_an_active_link(&mut self, now_ms: u64) {")
        .expect("handover");
    // Act / Assert: the core decides once per run, and learns whether it revoked.
    assert!(handover.contains("self.local.core.handover_due(gate_link_holds(now_ms))"));
    assert!(handover.contains("let revoked = revocation::revoke_at(generation, now_ms);"));
    assert!(handover.contains("self.local.core.note_handover(revoked);"));
    assert!(LOCAL_OWNER_SOURCE.contains("|| gate_link_holds(now_ms),"));
}

#[test]
fn hashrate_reads_carry_the_owning_generation() {
    // Arrange: soak 001 found local runs reporting no hashrate, because the reads
    // carried only a lease's generation and so had no permit.
    let (_, read) = STATUS_SOURCE
        .split_once("AsicWorkerCommand::ReadHashrateRegisters {")
        .expect("hashrate read");
    // Act / Assert
    assert!(read.contains("self.maybe_owner_generation(),"));
    assert!(!read.contains("self.maybe_bwg_session"));
}

#[test]
fn local_run_lifecycle_lines_reach_the_retained_log() {
    // Arrange
    let (_, helper) = LOCAL_OWNER_SOURCE
        .split_once("fn retained(level: log::Level, line: &str) {")
        .expect("retained helper");
    // Act
    let retains = helper.contains("crate::log_buffer::append_runtime_log_line(&line);");
    // Assert: a soak reads why a run ended from the retained log, beside the Wi-Fi lines.
    assert!(retains);
    assert!(helper.contains("format!(\"{line} uptime_ms={}\""));
    for marker in [
        "local_mining=run_began",
        "local_mining=run_ended",
        "local_mining=hold",
        "local_mining=handover",
        "local_mining=begin_refused",
        "local_mining=hardware_fault",
    ] {
        for level in ["info", "warn", "error"] {
            assert!(!LOCAL_OWNER_SOURCE.contains(&format!("log::{level}!(\"{marker}")), "{marker}");
        }
        assert!(LOCAL_OWNER_SOURCE.contains(marker), "{marker}");
    }
}
