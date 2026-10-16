//! ADR-0033 one-shot soak budget admission and work-gate timing.
use super::*;

#[test]
fn only_the_exact_soak_budget_exceeds_the_limited_ceiling() {
    // Arrange
    let budgets = [
        SOAK_ACTIVE_LIMIT_MS - 1,
        SOAK_ACTIVE_LIMIT_MS + 1,
        300_000,
        SOAK_ACTIVE_LIMIT_MS,
    ];
    // Act
    let admitted: Vec<bool> = budgets
        .iter()
        .map(|budget| {
            let gate = GenerationGate::new();
            let worker = gate.begin_link(0).expect("link");
            gate.admit_budget(worker, *budget)
        })
        .collect();
    // Assert
    assert_eq!(admitted, [false, false, false, true]);
    assert_eq!(SOAK_ACTIVE_LIMIT_MS, 619_050);
}

#[test]
fn a_soak_keeps_its_work_gate_open_for_exactly_600_seconds_after_first_work() {
    // Arrange
    let gate = GenerationGate::new();
    let worker = gate.begin_link(0).expect("link");
    assert!(gate.admit_budget(worker, SOAK_ACTIVE_LIMIT_MS));
    assert!(gate.activate_at(worker, 0));
    gate.heartbeat(worker, 1_000);
    // Act
    assert!(gate.begin_dispatch(gate.stamp(Some(worker)), 1_000));
    // Assert
    let timing = gate.timing(1_000).expect("first work");
    assert_eq!(timing.active_limit_ms, Some(619_050));
    assert_eq!(timing.work_gate_remaining_ms, Some(600_000));
}

#[test]
fn a_soak_budget_reserves_the_upstream_default_tail() {
    // Arrange
    let gate = GenerationGate::new();
    let worker = gate.begin_link(0).expect("link");
    assert!(gate.admit_budget(worker, SOAK_ACTIVE_LIMIT_MS));
    // Act
    let tail = gate.reserved_shutdown_tail_ms(worker);
    // Assert
    assert_eq!(tail, 19_050);
}

#[test]
fn a_normal_budget_reserves_the_conservative_tail() {
    // Arrange
    let gate = GenerationGate::new();
    let worker = gate.begin_link(0).expect("link");
    assert!(gate.admit_budget(worker, 180_000));
    // Act
    let tail = gate.reserved_shutdown_tail_ms(worker);
    // Assert
    assert_eq!(tail, 15_550);
}

#[test]
fn another_generation_never_inherits_a_soak_tail() {
    // Arrange: a soak was admitted for one generation; a later one has no budget.
    let gate = GenerationGate::new();
    let soak = gate.begin_link(0).expect("link");
    assert!(gate.admit_budget(soak, SOAK_ACTIVE_LIMIT_MS));
    let other = WorkerGeneration(soak.0.wrapping_add(1));
    // Act
    let tail = gate.reserved_shutdown_tail_ms(other);
    // Assert
    assert_eq!(tail, 15_550);
}
