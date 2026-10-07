//! BWG-007 durable-replay attribution: N1-N4 plus read-only and no-write guarantees.
#[path = "restoration_diagnostics/support.rs"]
mod support;
#[path = "restoration_diagnostics/verifier.rs"]
mod verifier;

use bitaxe_worker_control::RestorationReason;
use serde_json::{json, Value};
use support::*;

fn start(
    worker: &mut Worker,
    binding: &str,
    sequence: u64,
    now: u64,
) -> Result<Value, &'static str> {
    let payload = start_payload(binding, sequence);
    send(worker, &command("start_lease", Some(&payload)), now).map_err(|error| error.category())
}

fn renew(
    worker: &mut Worker,
    binding: &str,
    sequence: u64,
    now: u64,
) -> Result<Value, &'static str> {
    let payload = renew_payload(binding, sequence);
    send(worker, &command("renew_lease", Some(&payload)), now).map_err(|error| error.category())
}

fn pause(worker: &mut Worker, now: u64) {
    send(worker, &command("pause", None), now).expect("pause");
}

fn last(review: &Value) -> Value {
    review["last"].clone()
}

fn record(
    ordinal: u32,
    operation: &str,
    signature: &str,
    context: &str,
    replay_guard: &str,
) -> Value {
    json!({"ordinal":ordinal,"operation":operation,"signature":signature,"context":context,
        "replayGuard":replay_guard})
}

/// Boot 1 accepts Start(1) under C0 and pauses; boot 2 reacquires with the persisted store.
struct Rebooted {
    store: PersistedStore,
    worker: Worker,
    pre_reboot_fingerprint: Value,
    pre_reboot_start: Value,
}

fn rebooted() -> Rebooted {
    let store = PersistedStore::default();
    let mut first = boot(&store, None);
    let c0 = possess(&mut first, 1, 1_000);
    let pre_reboot_start = start_payload(&c0, 1);
    send(
        &mut first,
        &command("start_lease", Some(&pre_reboot_start)),
        1_000,
    )
    .expect("Start(1)");
    pause(&mut first, 2_000);
    possess(&mut first, 2, 2_001);
    let pre_reboot_fingerprint =
        review(&mut first, 2_001)["highWater"]["fingerprintSha256"].clone();
    let mut worker = boot(&store, Some(RestorationReason::Reboot));
    possess(&mut worker, 3, 500);
    send(&mut worker, &command("status", None), 500).expect("reports the reboot");
    // The next frame acknowledges the report; keep that marker clear out of the legs.
    send(&mut worker, &command("status", None), 501).expect("acknowledges the report");
    Rebooted {
        store,
        worker,
        pre_reboot_fingerprint,
        pre_reboot_start,
    }
}

#[test]
fn n1_durable_replay_after_reboot_is_attributed_to_the_persisted_high_water() {
    // Arrange
    let mut rebooted = rebooted();
    let replay = command("start_lease", Some(&rebooted.pre_reboot_start));

    // Act
    let rejected = send(&mut rebooted.worker, &replay, 600).map_err(|error| error.category());
    possess(&mut rebooted.worker, 4, 700);
    let review = review(&mut rebooted.worker, 700);

    // Assert
    assert_eq!(
        rejected.expect_err("replay rejected"),
        "authentication_failed"
    );
    assert_eq!(
        last(&review),
        record(
            1,
            "start",
            "valid",
            "mismatch",
            "at_or_below_durable_high_water"
        )
    );
    assert_eq!(review["highWater"]["advancedThisBoot"], false);
    assert_eq!(
        review["highWater"]["fingerprintSha256"],
        rebooted.pre_reboot_fingerprint
    );
}

#[test]
fn n1_replay_never_writes_durable_state_or_reaches_session_start() {
    // Arrange
    let mut rebooted = rebooted();
    let before = rebooted.store.counts();
    let replay = command("start_lease", Some(&rebooted.pre_reboot_start));

    // Act
    let _ = send(&mut rebooted.worker, &replay, 600);

    // Assert
    let after = rebooted.store.counts();
    assert_eq!(after.compare_and_store, before.compare_and_store);
    assert_eq!(after.mark_effect_pending, before.mark_effect_pending);
    assert_eq!(after.clear_effect_pending, before.clear_effect_pending);
    assert!(rebooted.worker.session().events.is_empty());
}

#[test]
fn n2_a_start_after_context_expiry_is_attributed_as_expired() {
    // Arrange
    let store = PersistedStore::default();
    let mut worker = boot(&store, None);
    let c1 = possess(&mut worker, 1, 1_000);

    // Act
    let rejected = start(&mut worker, &c1, 1, 61_001);
    possess(&mut worker, 2, 61_002);
    let review = review(&mut worker, 61_002);

    // Assert
    assert_eq!(rejected.expect_err("context expired"), "admission_required");
    assert_eq!(
        last(&review),
        record(1, "start", "not_evaluated", "expired", "not_evaluated")
    );
    assert_eq!(store.counts(), StoreCounts::default());
    assert!(worker.session().events.is_empty());
}

#[test]
fn n3_a_start_for_a_previous_context_is_attributed_as_a_fresh_mismatch() {
    // Arrange
    let store = PersistedStore::default();
    let mut worker = boot(&store, None);
    let c1 = possess(&mut worker, 1, 1_000);
    let _ = start(&mut worker, &c1, 1, 61_001);
    possess(&mut worker, 2, 61_002);

    // Act
    let rejected = start(&mut worker, &c1, 1, 61_003);
    possess(&mut worker, 3, 61_004);
    let review = review(&mut worker, 61_004);

    // Assert
    assert_eq!(
        rejected.expect_err("cross context"),
        "authentication_failed"
    );
    assert_eq!(
        last(&review),
        record(2, "start", "valid", "mismatch", "fresh")
    );
    assert_eq!(review["bootRejections"], 2);
    assert_eq!(store.counts().compare_and_store, 0);
    assert!(worker.session().events.is_empty());
}

#[test]
fn n4_an_in_context_renewal_replay_is_attributed_and_safe_stops() {
    // Arrange
    let store = PersistedStore::default();
    let mut worker = boot(&store, None);
    let c3 = possess(&mut worker, 1, 1_000);
    start(&mut worker, &c3, 5, 1_000).expect("fresh Start");
    renew(&mut worker, &c3, 6, 21_000).expect("renewal R1");

    // Act
    let rejected = renew(&mut worker, &c3, 6, 22_000);
    possess(&mut worker, 2, 22_001);
    let review = review(&mut worker, 22_001);

    // Assert
    assert_eq!(
        rejected.expect_err("replayed renewal"),
        "authentication_failed"
    );
    assert_eq!(
        last(&review),
        record(
            1,
            "renew",
            "valid",
            "current",
            "at_or_below_durable_high_water"
        )
    );
    assert_eq!(review["highWater"]["advancedThisBoot"], true);
    assert_eq!(
        worker.session().events,
        ["start", "renew", "control_failed"]
    );
}

#[test]
fn n4_replay_writes_only_the_safe_stop_cleanup_marker() {
    // Arrange
    let store = PersistedStore::default();
    let mut worker = boot(&store, None);
    let c3 = possess(&mut worker, 1, 1_000);
    start(&mut worker, &c3, 5, 1_000).expect("fresh Start");
    renew(&mut worker, &c3, 6, 21_000).expect("renewal R1");
    let before = store.counts();

    // Act
    let _ = renew(&mut worker, &c3, 6, 22_000);

    // Assert: the one clear is the active lease's safe stop, not a rejection write.
    let after = store.counts();
    assert_eq!(after.compare_and_store, before.compare_and_store);
    assert_eq!(after.mark_effect_pending, before.mark_effect_pending);
    assert_eq!(after.clear_effect_pending, before.clear_effect_pending + 1);
}

#[test]
fn the_rejection_review_is_refused_during_a_lease() {
    // Arrange
    let store = PersistedStore::default();
    let mut worker = boot(&store, None);
    let c1 = possess(&mut worker, 1, 1_000);
    start(&mut worker, &c1, 1, 1_000).expect("Start");
    let frame = command("authorization_rejection_review", Some(&json!({})));

    // Act
    let refused = worker.prepare_frame(&frame, 2_000);

    // Assert
    assert_eq!(
        refused.expect_err("lease active").category(),
        "invalid_transition"
    );
}

#[test]
fn the_rejection_review_requires_fresh_possession() {
    // Arrange
    let store = PersistedStore::default();
    let mut worker = boot(&store, None);
    possess(&mut worker, 1, 1_000);
    let frame = command("authorization_rejection_review", Some(&json!({})));

    // Act
    let refused = worker.prepare_frame(&frame, 61_000);

    // Assert
    assert_eq!(
        refused.expect_err("stale possession").category(),
        "admission_required"
    );
}

#[test]
fn the_rejection_review_writes_and_loads_nothing() {
    // Arrange
    let mut rebooted = rebooted();
    let replay = command("start_lease", Some(&rebooted.pre_reboot_start));
    let _ = send(&mut rebooted.worker, &replay, 600);
    possess(&mut rebooted.worker, 4, 700);
    let before = rebooted.store.counts();

    // Act
    let first = review(&mut rebooted.worker, 700);
    let second = review(&mut rebooted.worker, 701);

    // Assert
    assert_eq!(first, second);
    assert_eq!(rebooted.store.counts(), before);
    assert!(rebooted.worker.is_admitted());
}

#[test]
fn the_rejection_review_contains_no_authorization_material() {
    // Arrange
    let mut rebooted = rebooted();
    let replay = command("start_lease", Some(&rebooted.pre_reboot_start));
    let _ = send(&mut rebooted.worker, &replay, 600);
    let binding = possess(&mut rebooted.worker, 4, 700);
    let authorization = rebooted.pre_reboot_start["authorization"]
        .as_str()
        .expect("authorization")
        .to_owned();

    // Act
    let text = review(&mut rebooted.worker, 700).to_string();

    // Assert
    for secret in [
        authorization.as_str(),
        &authorization[..authorization.find('.').expect("segments")],
        LEASE_KEY_ID,
        LEASE_ID,
        CHALLENGE_ID,
        POOL_PASSWORD,
        binding.as_str(),
        "\"sequence\"",
    ] {
        assert!(!text.contains(secret), "review leaked {secret}");
    }
}

#[test]
fn reviews_survive_logical_session_replacement() {
    // Arrange
    let store = PersistedStore::default();
    let mut worker = boot(&store, None);
    let c1 = possess(&mut worker, 1, 1_000);
    let _ = start(&mut worker, &c1, 1, 61_001);
    worker.disconnect(61_002).expect("disconnect");

    // Act
    possess(&mut worker, 2, 61_003);
    let review = review(&mut worker, 61_003);

    // Assert
    assert_eq!(review["bootRejections"], 1);
}

#[test]
fn a_fresh_boot_starts_with_an_empty_rejection_record() {
    // Arrange
    let store = PersistedStore::default();
    let mut worker = boot(&store, None);
    let c1 = possess(&mut worker, 1, 1_000);
    let _ = start(&mut worker, &c1, 1, 61_001);
    let mut rebooted = boot(&store, None);
    possess(&mut rebooted, 2, 100);

    // Act
    let review = review(&mut rebooted, 100);

    // Assert
    assert_eq!(review["bootRejections"], 0);
    assert_eq!(review["last"], Value::Null);
}
