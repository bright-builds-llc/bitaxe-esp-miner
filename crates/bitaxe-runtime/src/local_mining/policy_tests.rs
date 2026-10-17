use super::*;

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

fn hold(inputs: LocalInputs) -> LocalDecision {
    decide(inputs, &RestartGuard::default(), 0)
}

#[test]
fn a_ready_device_begins_a_run_with_the_saved_profile() {
    // Arrange / Act / Assert
    assert_eq!(hold(ready()), LocalDecision::Begin(profile()));
}

#[test]
fn another_boot_owner_holds_before_everything_else() {
    // Arrange
    let inputs = LocalInputs {
        other_boot_owner: true,
        gate_link: true,
        operator_paused: true,
        ..ready()
    };
    // Act / Assert
    assert_eq!(
        hold(inputs),
        LocalDecision::Hold(LocalHoldReason::OtherBootOwner)
    );
}

#[test]
fn a_gate_link_holds_local_mining() {
    // Arrange / Act / Assert
    assert_eq!(
        hold(LocalInputs {
            gate_link: true,
            ..ready()
        }),
        LocalDecision::Hold(LocalHoldReason::GateLink)
    );
}

#[test]
fn a_latched_hardware_fault_holds_even_when_resumed() {
    // Arrange / Act / Assert
    assert_eq!(
        hold(LocalInputs {
            hardware_fault: true,
            ..ready()
        }),
        LocalDecision::Hold(LocalHoldReason::HardwareFault)
    );
}

#[test]
fn an_overheat_holds_ahead_of_a_pause() {
    // Arrange
    let inputs = LocalInputs {
        overheat_holds: true,
        operator_paused: true,
        ..ready()
    };
    // Act / Assert
    assert_eq!(hold(inputs), LocalDecision::Hold(LocalHoldReason::Overheat));
}

#[test]
fn a_pause_holds_local_mining() {
    // Arrange / Act / Assert
    assert_eq!(
        hold(LocalInputs {
            operator_paused: true,
            ..ready()
        }),
        LocalDecision::Hold(LocalHoldReason::Paused)
    );
}

#[test]
fn an_unavailable_network_holds_ahead_of_stale_safety() {
    // Arrange
    let inputs = LocalInputs {
        network_ready: false,
        safety_fresh: false,
        ..ready()
    };
    // Act / Assert
    assert_eq!(
        hold(inputs),
        LocalDecision::Hold(LocalHoldReason::NetworkUnavailable)
    );
}

#[test]
fn stale_safety_holds_local_mining() {
    // Arrange / Act / Assert
    assert_eq!(
        hold(LocalInputs {
            safety_fresh: false,
            ..ready()
        }),
        LocalDecision::Hold(LocalHoldReason::SafetyStale)
    );
}

#[test]
fn unavailable_actuation_holds_instead_of_failing_a_run() {
    // Arrange / Act / Assert
    assert_eq!(
        hold(LocalInputs {
            actuation_available: false,
            ..ready()
        }),
        LocalDecision::Hold(LocalHoldReason::ActuationUnavailable)
    );
}

#[test]
fn readiness_holds_ahead_of_a_refused_profile() {
    // Arrange
    let refusal = MiningHardwareProfile::from_device_settings(Some(600.0), Some(1200))
        .expect_err("out of range");
    // Act
    let decision = hold(LocalInputs {
        safety_fresh: false,
        profile: Err(refusal),
        ..ready()
    });
    // Assert
    assert_eq!(decision, LocalDecision::Hold(LocalHoldReason::SafetyStale));
}

#[test]
fn a_hardware_fault_holds_ahead_of_an_overheat() {
    // Arrange / Act / Assert
    assert_eq!(
        hold(LocalInputs {
            hardware_fault: true,
            overheat_holds: true,
            ..ready()
        }),
        LocalDecision::Hold(LocalHoldReason::HardwareFault)
    );
}

#[test]
fn a_gate_link_holds_ahead_of_a_hardware_fault() {
    // Arrange / Act / Assert
    assert_eq!(
        hold(LocalInputs {
            gate_link: true,
            hardware_fault: true,
            ..ready()
        }),
        LocalDecision::Hold(LocalHoldReason::GateLink)
    );
}

#[test]
fn an_unsupported_protocol_holds_local_mining() {
    // Arrange / Act / Assert
    assert_eq!(
        hold(LocalInputs {
            stratum_v1_supported: false,
            ..ready()
        }),
        LocalDecision::Hold(LocalHoldReason::StratumV1Unsupported)
    );
}

#[test]
fn a_saved_profile_outside_the_catalog_holds_with_its_refusal() {
    // Arrange
    let refusal = MiningHardwareProfile::from_device_settings(Some(600.0), Some(1200))
        .expect_err("out of range");
    // Act
    let decision = hold(LocalInputs {
        profile: Err(refusal),
        ..ready()
    });
    // Assert
    assert_eq!(
        decision,
        LocalDecision::Hold(LocalHoldReason::ProfileRefused(refusal))
    );
}

#[test]
fn a_failed_run_backs_off_for_thirty_seconds() {
    // Arrange
    let mut guard = RestartGuard::default();
    guard.note_run_ended(RunEnding::Failed, 0, 1_000);
    // Act
    let during = decide(ready(), &guard, 30_999);
    let after = decide(ready(), &guard, 31_000);
    // Assert
    assert_eq!(
        (during, after),
        (
            LocalDecision::Hold(LocalHoldReason::RestartBackoff),
            LocalDecision::Begin(profile())
        )
    );
}

#[test]
fn each_further_failure_doubles_the_backoff() {
    // Arrange
    let mut guard = RestartGuard::default();
    guard.note_run_ended(RunEnding::Failed, 0, 0);
    // Act
    guard.note_run_ended(RunEnding::Failed, 0, 100_000);
    // Assert: the second backoff is 60 s.
    assert_eq!(
        (
            decide(ready(), &guard, 159_999),
            decide(ready(), &guard, 160_000)
        ),
        (
            LocalDecision::Hold(LocalHoldReason::RestartBackoff),
            LocalDecision::Begin(profile())
        )
    );
}

fn guard_after(failures: u32) -> RestartGuard {
    let mut guard = RestartGuard::default();
    for at in 0..failures {
        guard.note_run_ended(RunEnding::Failed, 0, u64::from(at));
    }
    guard
}

#[test]
fn four_failed_runs_back_off_without_holding() {
    // Arrange
    let guard = guard_after(4);
    // Act: the fourth backoff is 240 s from its failure at 3 ms.
    let during = decide(ready(), &guard, 240_002);
    let after = decide(ready(), &guard, 240_003);
    // Assert
    assert_eq!(
        (during, after),
        (
            LocalDecision::Hold(LocalHoldReason::RestartBackoff),
            LocalDecision::Begin(profile())
        )
    );
}

#[test]
fn five_failed_runs_in_a_row_hold() {
    // Arrange
    let guard = guard_after(RESTART_FAILURES_BEFORE_HOLD);
    // Act / Assert
    assert_eq!(
        decide(ready(), &guard, u64::MAX),
        LocalDecision::Hold(LocalHoldReason::RestartGuardHeld)
    );
}

#[test]
fn resuming_ends_the_guards_hold() {
    // Arrange
    let mut guard = guard_after(RESTART_FAILURES_BEFORE_HOLD);
    // Act
    guard.resume();
    // Assert
    assert_eq!(
        decide(ready(), &guard, u64::MAX),
        LocalDecision::Begin(profile())
    );
}

#[test]
fn a_pause_is_reported_ahead_of_the_guards_hold() {
    // Arrange
    let guard = guard_after(RESTART_FAILURES_BEFORE_HOLD);
    // Act / Assert
    assert_eq!(
        decide(
            LocalInputs {
                operator_paused: true,
                ..ready()
            },
            &guard,
            0
        ),
        LocalDecision::Hold(LocalHoldReason::Paused)
    );
}

#[test]
fn the_guard_holds_ahead_of_readiness() {
    // Arrange
    let guard = guard_after(1);
    // Act / Assert
    assert_eq!(
        decide(
            LocalInputs {
                network_ready: false,
                ..ready()
            },
            &guard,
            1_000
        ),
        LocalDecision::Hold(LocalHoldReason::RestartBackoff)
    );
}

#[test]
fn a_stable_run_resets_the_guard() {
    // Arrange
    let mut guard = RestartGuard::default();
    for at in 0..4 {
        guard.note_run_ended(RunEnding::Failed, 0, at);
    }
    // Act: a run that hashed ten minutes, then failed.
    guard.note_run_ended(RunEnding::Failed, STABLE_RUN_MS, 1_000_000);
    // Assert: it counts as the first failure again, a 30 s backoff.
    assert_eq!(
        (
            decide(ready(), &guard, 1_029_999),
            decide(ready(), &guard, 1_030_000)
        ),
        (
            LocalDecision::Hold(LocalHoldReason::RestartBackoff),
            LocalDecision::Begin(profile())
        )
    );
}

#[test]
fn requested_and_overheat_endings_never_count_as_failures() {
    // Arrange
    let mut guard = RestartGuard::default();
    // Act
    guard.note_run_ended(RunEnding::Requested, 0, 0);
    guard.note_run_ended(RunEnding::Overheat, 0, 0);
    // Assert
    assert_eq!(decide(ready(), &guard, 0), LocalDecision::Begin(profile()));
}

#[test]
fn a_stable_run_that_ends_on_request_resets_the_guard() {
    // Arrange
    let mut guard = guard_after(4);
    // Act
    guard.note_run_ended(RunEnding::Requested, STABLE_RUN_MS, 10);
    // Assert
    assert_eq!(decide(ready(), &guard, 10), LocalDecision::Begin(profile()));
}
