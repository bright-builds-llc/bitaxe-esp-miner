use super::*;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use serde_json::json;
fn observation() -> CurrentObservation {
    CurrentObservation {
        boot_ordinal: 2,
        worker_generation: 7,
        serial_transport_epoch: 4,
        maybe_observed_at_us: Some(1000),
        clock_valid: true,
        maybe_station_ipv4: Some("192.168.1.2".into()),
        wifi_connected: true,
        maybe_socket: None,
    }
}
fn record(scope: Scope) -> V2Record {
    V2Record::admit(
        scope,
        URL_SAFE_NO_PAD.encode([1; 16]),
        &observation(),
        if scope == Scope::Channel {
            Some(1000 + AUTHORITY_US)
        } else {
            None
        },
    )
    .expect("bounded record")
}
#[test]
fn share_deadline_is_observed_once_at_guarded_dispatch_not_admission() {
    // Arrange
    let mut r = record(Scope::Share);
    assert_eq!(r.snapshot().maybe_authority_deadline_device_us, None);
    // Act
    assert!(r.budget_armed(2, 180_000, Some(2000)));
    assert!(r.budget_armed(2, 180_000, Some(2000)));
    assert!(!r.budget_armed(3, 180_000, Some(3000)));
    // Assert
    assert_eq!(
        r.snapshot().maybe_authority_deadline_device_us,
        Some(180_002_000)
    );
    assert_eq!(
        r.snapshot()
            .maybe_first_failure
            .expect("changed epoch")
            .category,
        FailureCategory::Evidence
    );
}
#[test]
fn unavailable_or_overflowed_budget_origin_is_never_a_zero_deadline() {
    // Arrange
    let mut r = record(Scope::Share);
    // Act / Assert
    assert!(!r.budget_armed(u64::MAX, 180_000, Some(2000)));
    assert_eq!(r.snapshot().maybe_authority_deadline_device_us, None);
    assert_eq!(
        r.snapshot()
            .maybe_first_failure
            .expect("clock failure")
            .category,
        FailureCategory::Clock
    );
}
#[test]
fn in_flight_timing_has_no_completed_aggregate_and_preserves_actual_origin() {
    // Arrange
    let mut r = record(Scope::Channel);
    // Act
    r.begin(Operation::ActTwoRead, Some(1200));
    let started = r.snapshot();
    r.end(Operation::ActTwoRead, Some(1300), false);
    let ended = r.snapshot();
    // Assert
    let t = &started.timings[Operation::ActTwoRead as usize];
    assert_eq!(t.count, 0);
    assert_eq!(t.maybe_first_started_at_device_us, None);
    assert_eq!(t.maybe_in_flight_started_at_device_us, Some(1200));
    let t = &ended.timings[Operation::ActTwoRead as usize];
    assert_eq!(t.count, 1);
    assert_eq!(t.maybe_first_started_at_device_us, Some(1200));
    assert_eq!(t.maybe_total_duration_us, Some(100));
}
#[test]
fn clock_failure_during_cleanup_retains_the_first_protocol_failure() {
    // Arrange
    let mut r = record(Scope::Channel);
    r.fail(Stage::Setup, FailureCategory::Protocol, Some(1100));
    // Act
    r.begin(Operation::SocketClose, Some(1200));
    r.end(Operation::SocketClose, None, true);
    r.socket_closed(None);
    r.joined(None, true);
    // Assert
    let s = r.snapshot();
    assert_eq!(
        s.maybe_first_failure.expect("first").category,
        FailureCategory::Protocol
    );
    assert!(s
        .secondary_failures
        .iter()
        .any(|f| f.category == FailureCategory::Clock));
    assert!(s.resources.socket_closed);
    assert_eq!(s.resources.maybe_socket_closed_at_us, None);
    assert_eq!(s.maybe_outcome, Some(Outcome::Incomplete));
}
#[test]
fn late_release_cannot_upgrade_an_incomplete_horizon() {
    // Arrange
    let mut r = record(Scope::Channel);
    r.tick(Some(1000 + OBSERVATION_US));
    // Act
    r.socket_closed(Some(1001 + OBSERVATION_US));
    r.joined(Some(1002 + OBSERVATION_US), true);
    // Assert
    let s = r.snapshot();
    assert_eq!(s.maybe_outcome, Some(Outcome::Incomplete));
    assert_eq!(s.maybe_terminal_at_device_us, Some(1000 + OBSERVATION_US));
    assert!(s.resources.worker_quiescent);
    assert!(!s.resources.fence_retained);
}
#[test]
fn event_overflow_is_sticky_instead_of_eviction() {
    // Arrange
    let mut r = record(Scope::Channel);
    // Act
    for i in 1..=MAX_EVENTS {
        r.event(
            Stage::Preparing,
            Some(1000 + i as u64),
            None,
            None,
            None,
            None,
        );
    }
    // Assert
    let s = r.snapshot();
    assert_eq!(s.events.len(), MAX_EVENTS);
    assert_eq!(s.events[0].kind, Stage::Admitted);
    assert_eq!(
        s.maybe_first_failure.expect("overflow").category,
        FailureCategory::Evidence
    );
}
#[test]
fn secret_bearing_setup_payload_is_never_fingerprinted() {
    // Arrange
    let mut r = record(Scope::Channel);
    // Act
    r.event(
        Stage::Setup,
        Some(1100),
        None,
        None,
        None,
        Some("a".repeat(64)),
    );
    // Assert
    assert_eq!(r.snapshot().events[1].maybe_payload_sha256, None);
}
#[test]
fn v2_grant_input_rejects_noncanonical_addresses_and_mixed_variants() {
    // Arrange
    let valid = json!({"profile":"bwg-worker-stratum-v2-standard/0.1","endpoint":"stratum+tcp://192.168.1.3:12345/","authorityPublicKey":URL_SAFE_NO_PAD.encode([2;32]),"userIdentity":"synthetic"});
    assert!(serde_json::from_value::<V2Stratum>(valid.clone())
        .expect("shape")
        .valid());
    // Act / Assert
    for endpoint in [
        "stratum+tcp://127.0.0.1:12345/",
        "stratum+tcp://192.168.01.3:12345/",
        "stratum+tcp://192.168.1.3:012345/",
        "stratum+tcp://host:12345/",
    ] {
        let mut bad = valid.clone();
        bad["endpoint"] = json!(endpoint);
        assert!(!serde_json::from_value::<V2Stratum>(bad)
            .expect("typed")
            .valid());
    }
    let mut mixed = valid;
    mixed["password"] = json!("not a V2 field");
    assert!(serde_json::from_value::<V2Stratum>(mixed).is_err());
}
#[test]
fn required_nullable_query_does_not_accept_a_missing_attempt_field() {
    // Arrange / Act / Assert
    assert!(serde_json::from_value::<V2Query>(
        json!({"schema":"worker-stratum-v2-query-v1","scope":"channel"})
    )
    .is_err());
}
#[test]
fn real_serde_producer_corpus_has_closed_shapes() {
    // Arrange
    let mut channel = record(Scope::Channel);
    channel.bind_pool(1, 1);
    channel.event(Stage::Preparing, Some(1100), None, None, None, None);
    channel.begin(Operation::InitiatorConstruction, Some(1101));
    let mut cancelled = record(Scope::Share);
    cancelled.fail(Stage::Revoked, FailureCategory::Authority, Some(1100));
    cancelled.joined(Some(1200), true);
    let mut incomplete = record(Scope::Channel);
    incomplete.tick(Some(1000 + OBSERVATION_US));
    let frozen = incomplete.snapshot();
    incomplete.socket_closed(Some(1001 + OBSERVATION_US));
    incomplete.joined(Some(1002 + OBSERVATION_US), true);
    let mut armed = record(Scope::Share);
    armed.bind_pool(3, 5);
    armed.budget_armed(2, 180_000, Some(2000));
    let records = vec![
        ("preparing", channel.snapshot()),
        ("pre_dispatch_cancel", cancelled.snapshot()),
        ("incomplete", frozen),
        ("late_release", incomplete.snapshot()),
        ("budget_armed", armed.snapshot()),
    ];
    // Act
    let cases:Vec<_>=records.into_iter().map(|(name,r)|{let mut observed=observation();observed.maybe_observed_at_us=r.maybe_observed_at_us;observed.clock_valid=r.maybe_observed_at_us.is_some();json!({"name":name,"status":V2Status {schema:"worker-stratum-v2-status-v1",scope:r.scope,state:r.state,observation:observed,maybe_connection:None,maybe_record:Some(r)}})}).collect();
    let output = serde_json::to_vec_pretty(&cases).expect("actual producer serde");
    // Assert: optional output is a synthetic interoperability corpus, never hardware proof.
    assert!(output.len() < 65536);
    if let Ok(path) = std::env::var("BWG_V2_CORPUS_PATH") {
        std::fs::write(path, output).expect("explicit synthetic corpus path");
    }
}

#[test]
fn protocol_failure_then_revocation_retains_only_independent_cleanup_causes() {
    // Arrange: native capture revokes on the first protocol failure; the network
    // owner may then report cancellation while unwinding its scoped resources.
    let mut r = record(Scope::Channel);
    r.fail(Stage::Job, FailureCategory::Protocol, Some(1100));
    let original = r.snapshot().maybe_first_failure;
    // Act
    r.fail(
        Stage::WorkerQuiescent,
        FailureCategory::Authority,
        Some(1101),
    );
    r.fail(Stage::WorkerQuiescent, FailureCategory::Timeout, Some(1102));
    r.fail(
        Stage::WorkerQuiescent,
        FailureCategory::Protocol,
        Some(1103),
    );
    r.fail(Stage::SocketClosed, FailureCategory::Cleanup, Some(1104));
    r.fail(Stage::WorkerQuiescent, FailureCategory::Clock, None);
    // Assert
    let result = r.snapshot();
    assert_eq!(result.maybe_first_failure, original);
    assert_eq!(
        result
            .secondary_failures
            .iter()
            .map(|f| f.category)
            .collect::<Vec<_>>(),
        vec![FailureCategory::Cleanup, FailureCategory::Clock]
    );
}
