use super::*;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde_json::json;

fn allowance(ordinal: u32) -> SoakAllowance {
    serde_json::from_value(json!({"id": URL_SAFE_NO_PAD.encode([7; 16]), "maximumActiveMilliseconds": SOAK_MAXIMUM_ACTIVE_MS,
        "ordinal": ordinal, "schema": "worker-soak-allowance-v1"}))
    .expect("allowance")
}

#[test]
fn a_soak_allowance_must_carry_exactly_the_work_gate_plus_shutdown_tail() {
    // Arrange
    let mut value = serde_json::to_value(allowance(1)).expect("value");
    value["maximumActiveMilliseconds"] = json!(SOAK_WORK_GATE_MS);
    // Act
    let short: SoakAllowance = serde_json::from_value(value).expect("parsed");
    // Assert
    assert!(allowance(1).validate());
    assert!(!short.validate());
    assert_eq!(SOAK_MAXIMUM_ACTIVE_MS - SOAK_WORK_GATE_MS, 15_550);
}

#[test]
fn reserving_charges_the_whole_allowance_before_preparation() {
    // Arrange
    let ledger = SoakLedger::default();
    // Act
    let reserved = ledger.reserve(&allowance(1)).expect("reserved");
    // Assert
    assert!(reserved.pending());
    assert_eq!(
        (
            reserved.total_charged_ms(),
            reserved.next_ordinal(),
            reserved.last_completed_ordinal()
        ),
        (615_550, 2, 0)
    );
}

#[test]
fn a_pending_soak_blocks_the_next_reservation() {
    // Arrange
    let reserved = SoakLedger::default()
        .reserve(&allowance(1))
        .expect("reserved");
    // Act / Assert
    assert!(reserved.reserve(&allowance(2)).is_err());
}

#[test]
fn replays_and_gaps_are_rejected() {
    // Arrange
    let finished = SoakLedger::default()
        .reserve(&allowance(1))
        .and_then(|ledger| ledger.finish())
        .expect("finished");
    // Act / Assert
    assert!(finished.reserve(&allowance(1)).is_err());
    assert!(finished.reserve(&allowance(3)).is_err());
    assert!(finished.reserve(&allowance(2)).is_ok());
}

#[test]
fn finishing_is_idempotent_and_never_refunds() {
    // Arrange
    let reserved = SoakLedger::default()
        .reserve(&allowance(1))
        .expect("reserved");
    // Act
    let once = reserved.finish().expect("once");
    let twice = once.finish().expect("twice");
    // Assert
    assert_eq!(once, twice);
    assert_eq!(
        (
            twice.total_charged_ms(),
            twice.pending(),
            twice.last_completed_ordinal()
        ),
        (615_550, false, 1)
    );
}

#[test]
fn a_stored_ledger_whose_total_disagrees_with_its_ordinals_is_rejected() {
    // Arrange
    let mut value = serde_json::to_value(
        SoakLedger::default()
            .reserve(&allowance(1))
            .expect("reserved"),
    )
    .expect("value");
    value["total_charged_ms"] = json!(600_000);
    // Act
    let tampered: SoakLedger = serde_json::from_value(value).expect("parsed");
    // Assert
    assert!(tampered.validate().is_err());
}
