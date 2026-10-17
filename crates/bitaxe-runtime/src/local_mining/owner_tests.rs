use super::*;
use crate::local_mining::policy::{RESTART_BACKOFF_BASE_MS, STABLE_RUN_MS};
use bitaxe_stratum::v1::production_session::MiningHardwareProfile;

fn profile() -> MiningHardwareProfile {
    MiningHardwareProfile::from_device_settings(Some(485.0), Some(1200)).expect("catalog pair")
}

fn ready() -> LocalInputs {
    LocalInputs {
        other_boot_owner: false,
        gate_link: false,
        operator_paused: false,
        hardware_fault: false,
        overheat_holds: false,
        network_ready: true,
        stratum_v1_supported: true,
        safety_fresh: true,
        actuation_available: true,
        profile: Ok(profile()),
    }
}

/// Ends one run begun at `at_ms` with no hashing, as a failure.
fn fail_run(core: &mut LocalOwnerCore, at_ms: u64) {
    core.run_began();
    core.run_ended(
        None,
        Some(ProductionSessionBlocker::CampaignLeaseConsumed),
        at_ms,
    );
}

#[test]
fn an_idle_hold_is_kept_for_status() {
    // Arrange
    let mut core = LocalOwnerCore::default();
    // Act
    core.decide_idle(
        LocalInputs {
            operator_paused: true,
            ..ready()
        },
        0,
    );
    // Assert
    assert_eq!(core.maybe_hold(), Some(LocalHoldReason::Paused));
}

#[test]
fn a_running_owner_reports_no_hold() {
    // Arrange
    let mut core = LocalOwnerCore::default();
    core.decide_idle(
        LocalInputs {
            operator_paused: true,
            ..ready()
        },
        0,
    );
    // Act
    core.run_began();
    // Assert
    assert_eq!(core.maybe_hold(), None);
}

#[test]
fn a_run_stopped_by_a_network_outage_does_not_spend_the_guard() {
    // Arrange
    let mut core = LocalOwnerCore::default();
    core.run_began();
    // Act
    let ending = core.run_ended(None, Some(ProductionSessionBlocker::NetworkUnavailable), 20);
    // Assert
    assert_eq!(ending, Some(RunEnding::Requested));
    assert_eq!(
        core.decide_idle(ready(), 21),
        LocalDecision::Begin(profile())
    );
}

#[test]
fn a_revoked_run_backs_off() {
    // Arrange
    let mut core = LocalOwnerCore::default();
    // Act
    fail_run(&mut core, 20);
    // Assert
    assert_eq!(
        core.decide_idle(ready(), 20 + RESTART_BACKOFF_BASE_MS - 1),
        LocalDecision::Hold(LocalHoldReason::RestartBackoff)
    );
}

#[test]
fn an_unsafe_stop_fails_whatever_the_session_blocked_on() {
    // Arrange: the unsafe sample also left the session with stale safety.
    let mut core = LocalOwnerCore::default();
    core.run_began();
    // Act
    let ending = core.run_ended(
        Some(UnsafeStop::Other),
        Some(ProductionSessionBlocker::SafetyPrerequisitesStale),
        1,
    );
    // Assert
    assert_eq!(ending, Some(RunEnding::Failed));
}

#[test]
fn a_settings_change_is_a_requested_ending() {
    // Arrange
    let mut core = LocalOwnerCore::default();
    core.run_began();
    core.note_settings_changed();
    // Act
    let ending = core.run_ended(
        None,
        Some(ProductionSessionBlocker::CampaignLeaseConsumed),
        5,
    );
    // Assert
    assert_eq!(ending, Some(RunEnding::Requested));
}

#[test]
fn an_overheat_outranks_a_settings_change_and_a_pause() {
    // Arrange
    let mut core = LocalOwnerCore::default();
    core.run_began();
    core.note_settings_changed();
    // Act
    let ending = core.run_ended(
        Some(UnsafeStop::Overheat),
        Some(ProductionSessionBlocker::OperatorPaused),
        2,
    );
    // Assert
    assert_eq!(ending, Some(RunEnding::Overheat));
}

#[test]
fn ending_without_a_run_records_nothing() {
    // Arrange
    let mut core = LocalOwnerCore::default();
    // Act
    let maybe_ending = core.run_ended(None, None, 0);
    // Assert
    assert_eq!(maybe_ending, None);
    assert_eq!(
        core.decide_idle(ready(), 1),
        LocalDecision::Begin(profile())
    );
}

#[test]
fn a_stable_run_resets_earlier_failures() {
    // Arrange: one failure, then a run that hashed ten minutes and then failed.
    let mut core = LocalOwnerCore::default();
    fail_run(&mut core, 0);
    core.run_began();
    core.note_run_progress(true, 1);
    // Act
    core.run_ended(None, None, 1 + STABLE_RUN_MS);
    // Assert: only the latest failure counts, so the first backoff applies.
    assert_eq!(
        core.decide_idle(ready(), 1 + STABLE_RUN_MS + RESTART_BACKOFF_BASE_MS),
        LocalDecision::Begin(profile())
    );
}

#[test]
fn hashing_time_accrues_across_interruptions() {
    // Arrange: four failures, then a run whose two hashing stretches together reach
    // a stable run before it fails. Without accrual this fifth failure would hold.
    let mut core = LocalOwnerCore::default();
    for at in 0..4 {
        fail_run(&mut core, at);
    }
    core.run_began();
    core.note_run_progress(true, 10);
    core.note_run_progress(false, 10 + STABLE_RUN_MS / 2);
    core.note_run_progress(true, 20 + STABLE_RUN_MS / 2);
    let end_ms = 20 + STABLE_RUN_MS;
    // Act
    core.run_ended(None, None, end_ms);
    // Assert
    assert_eq!(
        core.decide_idle(ready(), end_ms + RESTART_BACKOFF_BASE_MS),
        LocalDecision::Begin(profile())
    );
}

#[test]
fn five_failed_runs_hold_until_resumed() {
    // Arrange
    let mut core = LocalOwnerCore::default();
    for at in 0..5 {
        fail_run(&mut core, at);
    }
    let held = core.decide_idle(ready(), u64::MAX);
    // Act
    core.resume();
    // Assert
    assert_eq!(held, LocalDecision::Hold(LocalHoldReason::RestartGuardHeld));
    assert_eq!(
        core.decide_idle(ready(), 5),
        LocalDecision::Begin(profile())
    );
}
