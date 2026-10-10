//! `boot_review`: this boot's closed reset category, read-only after fresh idle possession.
use bitaxe_worker_control::BootResetCause;
use serde_json::{json, Value};

use super::support::*;

fn boot_review_frame(payload: Option<&Value>) -> Vec<u8> {
    command("boot_review", payload)
}

fn reset_cause_of(cause: BootResetCause) -> Value {
    let store = PersistedStore::default();
    let mut worker = boot_with_cause(&store, cause);
    possess(&mut worker, 1, 1_000);
    reviewed(&mut worker, "boot_review", 1_000)["result"]["resetCause"].clone()
}

#[test]
fn the_boot_review_has_the_exact_v1_shape() {
    // Arrange
    let store = PersistedStore::default();
    let mut worker = boot_with_cause(&store, BootResetCause::PowerOn);
    possess(&mut worker, 1, 1_000);

    // Act
    let response = reviewed(&mut worker, "boot_review", 1_000);

    // Assert
    assert_eq!(
        response,
        json!({"protocolVersion":"bwg-worker-controller/0.4","requestId":"serial_restoration",
            "ok":true,"result":{"schema":"worker-boot-review-v1","resetCause":"power_on"}})
    );
}

#[test]
fn every_reset_category_maps_to_its_stable_label() {
    // Arrange
    let cases = [
        (BootResetCause::PowerOn, "power_on"),
        (BootResetCause::SoftwareCpu, "software_cpu"),
        (BootResetCause::Watchdog, "watchdog"),
        (BootResetCause::Panic, "panic"),
        (BootResetCause::Brownout, "brownout"),
        (BootResetCause::Other, "other"),
    ];

    // Act
    let labels = cases.map(|(cause, _)| reset_cause_of(cause));

    // Assert
    assert_eq!(labels, cases.map(|(_, label)| json!(label)));
}

#[test]
fn the_boot_review_requires_an_exactly_empty_payload() {
    // Arrange
    let store = PersistedStore::default();
    let mut worker = boot_with_cause(&store, BootResetCause::PowerOn);
    possess(&mut worker, 1, 1_000);

    // Act
    let missing = worker.prepare_frame(&boot_review_frame(None), 1_000);
    let extra = worker.prepare_frame(&boot_review_frame(Some(&json!({"x": 1}))), 1_000);

    // Assert
    assert_eq!(
        missing.expect_err("payload required").category(),
        "invalid_request"
    );
    assert_eq!(
        extra.expect_err("payload must be empty").category(),
        "invalid_request"
    );
}

#[test]
fn the_boot_review_requires_fresh_possession() {
    // Arrange
    let store = PersistedStore::default();
    let mut worker = boot_with_cause(&store, BootResetCause::PowerOn);
    possess(&mut worker, 1, 1_000);

    // Act
    let refused = worker.prepare_frame(&boot_review_frame(Some(&json!({}))), 61_000);

    // Assert
    assert_eq!(
        refused.expect_err("stale possession").category(),
        "admission_required"
    );
}

#[test]
fn the_boot_review_is_refused_during_a_lease() {
    // Arrange
    let store = PersistedStore::default();
    let mut worker = boot_with_cause(&store, BootResetCause::PowerOn);
    let binding = possess(&mut worker, 1, 1_000);
    send(
        &mut worker,
        &command("start_lease", Some(&start_payload(&binding, 1))),
        1_000,
    )
    .expect("Start");

    // Act
    let refused = worker.prepare_frame(&boot_review_frame(Some(&json!({}))), 2_000);

    // Assert
    assert_eq!(
        refused.expect_err("lease active").category(),
        "invalid_transition"
    );
}

#[test]
fn the_boot_review_follows_the_reboot_status_report() {
    // Arrange
    let store = PersistedStore::default();
    let mut worker = boot(
        &store,
        Some(bitaxe_worker_control::RestorationReason::Reboot),
    );
    possess(&mut worker, 1, 1_000);
    let status = send(&mut worker, &command("status", None), 1_000).expect("status");

    // Act
    let response = reviewed(&mut worker, "boot_review", 1_001);

    // Assert
    assert_eq!(status["result"]["restoration"]["reason"], "reboot");
    assert_eq!(response["result"]["resetCause"], "other");
}

#[test]
fn the_boot_review_writes_signs_and_starts_nothing() {
    // Arrange
    let store = PersistedStore::default();
    let mut worker = boot_with_cause(&store, BootResetCause::Brownout);
    possess(&mut worker, 1, 1_000);
    let before = store.counts();

    // Act
    let first = reviewed(&mut worker, "boot_review", 1_000);
    let second = reviewed(&mut worker, "boot_review", 1_001);

    // Assert
    assert_eq!(first["result"], second["result"]);
    assert_eq!(store.counts(), before);
    assert!(worker.session().events.is_empty());
    assert_eq!(worker.session().reset_cause_reads.get(), 2);
    assert!(worker.is_admitted());
    assert_eq!(review(&mut worker, 1_002)["bootRejections"], 0);
}

#[test]
fn the_boot_review_carries_no_status_evidence() {
    // Arrange
    let store = PersistedStore::default();
    let mut worker = boot_with_cause(&store, BootResetCause::SoftwareCpu);
    possess(&mut worker, 1, 1_000);

    // Act
    let response = reviewed(&mut worker, "boot_review", 1_000);

    // Assert
    let mut keys: Vec<&String> = response["result"]
        .as_object()
        .expect("result object")
        .keys()
        .collect();
    keys.sort();
    assert_eq!(keys, ["resetCause", "schema"]);
}
