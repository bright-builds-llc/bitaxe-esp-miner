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
