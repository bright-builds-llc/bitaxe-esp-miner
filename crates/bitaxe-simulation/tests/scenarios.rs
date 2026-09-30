use bitaxe_simulation::{run_scenario, CheckStatus};

#[test]
fn scenario_repeatability_includes_the_entire_serialized_result() {
    // Arrange
    let first = run_scenario("healthy-lifecycle", 42).expect("scenario");
    // Act
    let second = run_scenario("healthy-lifecycle", 42).expect("scenario");
    // Assert
    assert_eq!(first, second);
    assert_eq!(
        serde_json::to_vec(&first).expect("encode"),
        serde_json::to_vec(&second).expect("encode")
    );
}

#[test]
fn real_signed_controller_start_prepares_then_restores_the_board() {
    // Arrange / Act
    let result = run_scenario("healthy-lifecycle", 1).expect("scenario");
    // Assert
    assert_eq!(result.actual_outcome, "started", "{result:?}");
    assert!(
        result
            .checks
            .iter()
            .filter(|c| c.status != CheckStatus::Unsupported)
            .all(|c| c.status == CheckStatus::Passed),
        "{result:?}"
    );
    assert!(!result.hardware_qualified);
    assert!(result
        .checks
        .iter()
        .any(|c| c.id == "strict_live_profile_share" && c.status == CheckStatus::Unsupported));
}

#[test]
fn injected_safety_failures_stop_at_the_actual_stabilization_boundary() {
    // Arrange
    for name in [
        "stale-safety-step5",
        "zero-fan-step5",
        "unsafe-reading-step5",
    ] {
        // Act
        let result = run_scenario(name, 7).expect("scenario");
        // Assert
        assert_eq!(result.actual_outcome, "session_failed", "{result:?}");
        assert_eq!(
            result
                .maybe_earliest_failure
                .as_ref()
                .expect("failure")
                .phase,
            "wait_for_core_voltage_stabilization_500_ms"
        );
        assert!(
            result
                .checks
                .iter()
                .all(|c| c.status == CheckStatus::Passed),
            "{result:?}"
        );
    }
}

#[test]
fn production_verifier_rejects_reused_possession_challenge() {
    // Arrange / Act
    let result = run_scenario("replay-challenge", 1).expect("scenario");
    // Assert
    assert_eq!(result.actual_outcome, "invalid_proof");
    assert!(result.journal.is_empty());
}

#[test]
fn persistence_rejection_precedes_any_board_preparation() {
    // Arrange / Act
    let result = run_scenario("persistence-failure", 1).expect("scenario");
    // Assert
    assert_eq!(result.actual_outcome, "authentication_failed", "{result:?}");
    assert!(!result
        .journal
        .iter()
        .any(|event| event.phase == "enable_asic"));
    assert!(
        result
            .checks
            .iter()
            .all(|c| c.status == CheckStatus::Passed),
        "{result:?}"
    );
}

#[test]
fn unsupported_boundary_never_passes_by_default() {
    // Arrange / Act
    let result = run_scenario("live-writer", 1).expect("scenario");
    // Assert
    assert_eq!(result.actual_outcome, "unsupported");
    assert_eq!(result.checks[0].status, CheckStatus::Unsupported);
}

#[test]
fn implemented_fault_scenarios_do_not_hide_failed_checks() {
    // Arrange
    for name in [
        "heartbeat-loss",
        "cancel-preparation",
        "stop-rejection",
        "reboot-before-status",
        "max-status",
        "heap-fragmented",
        "internal-allocation-failure",
    ] {
        // Act
        let result = run_scenario(name, 123).expect("scenario");
        // Assert
        assert!(
            result
                .checks
                .iter()
                .all(|check| check.status != CheckStatus::Failed),
            "{name}: {result:?}"
        );
    }
}

#[test]
fn fixed_and_expanded_seed_corpus_preserves_repeatability_and_nonclaims() {
    // Arrange
    let seeds = [1, 19, 205].into_iter().chain(1000..1032);
    // Act / Assert
    for seed in seeds {
        for name in bitaxe_simulation::scenario_names() {
            let first = run_scenario(name, seed).expect("known scenario");
            let second = run_scenario(name, seed).expect("known scenario");
            assert_eq!(first, second, "{name} seed={seed}");
            assert!(!first.hardware_qualified, "{name} seed={seed}");
            assert!(first.modeled_memory, "{name} seed={seed}");
            assert!(
                first
                    .checks
                    .iter()
                    .all(|check| check.status != CheckStatus::Failed),
                "{name} seed={seed}: {first:?}"
            );
        }
    }
}

#[test]
fn split_initialization_preserves_diagnostic_checkpoint_order_and_result() {
    // Arrange
    let mut phases = Vec::with_capacity(4);
    // Act
    let observed =
        bitaxe_simulation::run_scenario_with_observer("healthy-lifecycle", 1, &mut |phase| {
            phases.push(phase)
        })
        .expect("observed scenario");
    let normal = run_scenario("healthy-lifecycle", 1).expect("normal scenario");
    // Assert
    assert_eq!(
        phases,
        [
            bitaxe_simulation::ScenarioPhase::Entry,
            bitaxe_simulation::ScenarioPhase::BeforeController,
            bitaxe_simulation::ScenarioPhase::BeforePossession,
            bitaxe_simulation::ScenarioPhase::AfterPossession
        ]
    );
    assert_eq!(observed, normal);
}
