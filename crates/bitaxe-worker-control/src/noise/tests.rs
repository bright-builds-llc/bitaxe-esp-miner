use super::*;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
fn input() -> NoiseStart {
    NoiseStart {
        schema: StartSchema::V2,
        attempt_id: URL_SAFE_NO_PAD.encode([1; 16]),
        expected_boot_ordinal: 2,
        network_observed_at_us: 1_000_000,
        fixture_ipv4: "192.168.1.3".into(),
        fixture_port: 12345,
        authority_public_key: URL_SAFE_NO_PAD.encode([2; 32]),
    }
}
fn observation() -> NoiseObservation {
    NoiseObservation {
        boot_ordinal: 2,
        worker_generation: 7,
        transport_epoch: 4,
        observed_at_us: 1_000_000,
        station_ipv4: Some("192.168.1.2".into()),
        wifi_connected: true,
    }
}
fn record() -> NoiseRecord {
    let input = input();
    NoiseRecord::admit(
        &input,
        &observation(),
        input.input_sha256().expect("digest"),
    )
    .expect("admit")
}
fn successful_protocol(record: &mut NoiseRecord) {
    assert!(record.dispatch(1_000_001));
    record.stage(NoiseStage::NoisePrepared, 2_000_000, 999_999, None);
    record.socket_opened(32123);
    record.stage(NoiseStage::TcpConnected, 2_001_000, 1000, None);
    record.stage(NoiseStage::ActOneWritten, 2_002_000, 1000, Some(64));
    record.stage(NoiseStage::ActTwoReceived, 2_003_000, 1000, Some(234));
    record.stage(NoiseStage::AuthorityVerified, 2_004_000, 1000, None);
    record.stage(NoiseStage::ProofWritten, 2_005_000, 1000, Some(22));
    record.cleanup(Some(2_005_000));
    record.socket_closed(2_006_000, 1000);
}
#[test]
fn positive_cleanup_may_exceed_five_seconds_but_must_fit_authority() {
    // Arrange
    let mut record = record();
    successful_protocol(&mut record);
    // Act
    record.joined(9_000_000, false);
    // Assert
    assert_eq!(
        record.job().terminal.expect("terminal").outcome,
        NoiseOutcome::Accepted
    );
    assert_eq!(record.job().resources.deadline_at_us, Some(126_000_000));
    assert_eq!(record.job().resources.deadline_met, Some(true));
    assert!(!record.active());
}
#[test]
fn cancelled_opaque_work_keeps_fence_until_observed_join() {
    // Arrange
    let mut record = record();
    record.dispatch(1_000_001);
    // Act
    record.fail(NoiseFailure::new(
        FailureStage::NoisePrepared,
        NoiseCategory::AuthorityLost,
        NoiseDetail::CancelRequested,
        Some(2_000_000),
    ));
    record.tick(8_000_000);
    // Assert: the v2 observation horizon is not cancellation plus five seconds.
    assert!(record.active());
    assert!(record.job().terminal.is_none());
    record.joined(9_000_000, false);
    assert_eq!(
        record.job().terminal.expect("terminal").outcome,
        NoiseOutcome::Cancelled
    );
    assert!(!record.active());
}
#[test]
fn late_join_preserves_incomplete_and_missed_horizon() {
    // Arrange
    let mut record = record();
    record.dispatch(1_000_001);
    // Act
    record.tick(126_000_000);
    let first = record.job().first_failure;
    record.joined(127_000_000, false);
    // Assert
    assert_eq!(
        record.job().terminal.expect("terminal").outcome,
        NoiseOutcome::Incomplete
    );
    assert_eq!(record.job().first_failure, first);
    assert_eq!(record.job().resources.deadline_met, Some(false));
    assert_eq!(record.job().resources.released_at_us, Some(127_000_000));
    assert!(!record.active());
}
#[test]
fn protocol_success_joined_after_authority_is_expired() {
    // Arrange
    let mut record = record();
    successful_protocol(&mut record);
    // Act
    record.joined(122_000_000, false);
    // Assert
    assert_eq!(
        record.job().terminal.expect("terminal").outcome,
        NoiseOutcome::Expired
    );
}
#[test]
fn clock_rollback_cannot_fabricate_timely_release() {
    // Arrange
    let mut record = record();
    record.dispatch(1_000_001);
    // Act
    record.tick(1);
    record.joined(2, false);
    // Assert
    assert_eq!(
        record.job().terminal.expect("terminal").outcome,
        NoiseOutcome::Incomplete
    );
    assert_eq!(record.job().resources.released_at_us, None);
    assert_eq!(record.job().resources.deadline_met, Some(false));
    assert!(record.job().resources.failure.is_some());
}
#[test]
fn malformed_and_old_start_versions_fail_closed() {
    // Arrange
    let value = serde_json::to_value(input()).expect("json");
    let mut wrong = value.clone();
    wrong["schema"] = serde_json::json!("worker-noise-diagnostic-start-v1");
    // Act / Assert
    assert!(serde_json::from_value::<NoiseStart>(wrong).is_err());
    let mut extra = value;
    extra["password"] = serde_json::json!("synthetic-forbidden");
    assert!(serde_json::from_value::<NoiseStart>(extra).is_err());
}

#[test]
fn delayed_stage_publication_does_not_turn_a_later_supervisor_sample_into_clock_rollback() {
    // Arrange
    let mut record = record();
    record.dispatch(1_000_001);
    record.tick(2_000_100);
    // Act
    record.stage(NoiseStage::NoisePrepared, 2_000_000, 999_999, None);
    // Assert
    assert!(record.job().first_failure.is_none());
    assert!(record.permitted());
}
#[test]
fn cleanup_clock_failure_preserves_first_io_cause_and_explains_null_times() {
    // Arrange
    let mut record = record();
    record.dispatch(1_000_001);
    let first = NoiseFailure::new(
        FailureStage::ActTwoReceived,
        NoiseCategory::Read,
        NoiseDetail::Io,
        Some(2_000_000),
    );
    record.fail(first);
    // Act
    record.fail(NoiseFailure::new(
        FailureStage::Cleanup,
        NoiseCategory::ClockInvalid,
        NoiseDetail::ClockDiscontinuity,
        None,
    ));
    record.joined(2_000_001, false);
    // Assert
    assert_eq!(record.job().first_failure, Some(first));
    assert_eq!(
        record
            .job()
            .resources
            .failure
            .expect("secondary cause")
            .category,
        NoiseCategory::ClockInvalid
    );
    assert_eq!(record.job().resources.released_at_us, None);
    assert!(record.reportable());
}
#[test]
fn clock_failure_after_frozen_horizon_does_not_rewrite_causes_or_invent_late_times() {
    // Arrange
    let mut record = record();
    record.dispatch(1_000_001);
    record.tick(126_000_000);
    let first = record.job().first_failure;
    let cleanup = record.job().resources.failure;
    // Act
    record.tick(1);
    record.joined(2, false);
    // Assert
    assert_eq!(record.job().first_failure, first);
    assert_eq!(record.job().resources.failure, cleanup);
    assert_eq!(record.job().resources.released_at_us, None);
    assert_eq!(record.job().resources.deadline_met, Some(false));
    assert!(record.job().stages.is_empty());
    assert!(!record.reportable());
}

#[test]
fn serialized_producer_corpus_contains_closed_statuses() {
    // Arrange: deterministic synthetic inputs; no device or private runtime data.
    let mut cases = serde_json::Map::new();
    let mut add = |name: &str, record: &NoiseRecord, now: u64| {
        let mut observation = observation();
        observation.observed_at_us = now;
        cases.insert(
            name.into(),
            serde_json::to_value(record.status(observation)).expect("status"),
        );
    };
    let mut admitted = record();
    add("admitted", &admitted, 1_000_000);
    admitted.dispatch(1_000_001);
    add("running", &admitted, 1_000_002);
    let mut accepted = record();
    successful_protocol(&mut accepted);
    accepted.joined(9_000_000, false);
    add("accepted", &accepted, 9_000_001);
    let mut rejected = record();
    rejected.fail(NoiseFailure::new(
        FailureStage::NoisePrepared,
        NoiseCategory::Preparation,
        NoiseDetail::Io,
        Some(2_000_000),
    ));
    rejected.joined(2_000_001, false);
    add("rejected", &rejected, 2_000_002);
    let mut cancelled = record();
    cancelled.fail(NoiseFailure::new(
        FailureStage::Cleanup,
        NoiseCategory::AuthorityLost,
        NoiseDetail::CancelRequested,
        Some(2_000_000),
    ));
    cancelled.joined(9_000_000, false);
    add("cancelled", &cancelled, 9_000_001);
    let mut expired = record();
    successful_protocol(&mut expired);
    expired.joined(122_000_000, false);
    add("expired", &expired, 122_000_001);
    let mut incomplete = record();
    incomplete.tick(126_000_000);
    add("incomplete", &incomplete, 126_000_001);
    incomplete.joined(127_000_000, false);
    add("late_released", &incomplete, 127_000_001);
    rejected.fail(NoiseFailure::new(
        FailureStage::Cleanup,
        NoiseCategory::ClockInvalid,
        NoiseDetail::ClockDiscontinuity,
        None,
    ));
    let mut clock = record();
    clock.fail(NoiseFailure::new(
        FailureStage::NoisePrepared,
        NoiseCategory::Preparation,
        NoiseDetail::Io,
        Some(2_000_000),
    ));
    clock.tick(1);
    clock.joined(2, false);
    add("io_then_clock_failure", &clock, 3);
    // Act
    let encoded = serde_json::to_string_pretty(&cases).expect("corpus");
    // Assert
    assert_eq!(cases.len(), 9);
    assert!(cases
        .values()
        .all(|case| case["schema"] == "worker-noise-diagnostic-status-v2"));
    // Explicit developer export of this actual serializer, never hardware evidence.
    if let Ok(path) = std::env::var("NOISE_STATUS_CORPUS_PATH") {
        use std::io::Write;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)
            .expect("exclusive synthetic export");
        file.write_all(encoded.as_bytes()).expect("write corpus");
    }
}

#[test]
fn missing_clock_revokes_permission_while_opaque_worker_is_still_running() {
    // Arrange
    let mut record = record();
    record.dispatch(1_000_001);
    // Act
    let permitted = record.observe_clock(None);
    // Assert
    assert!(!permitted);
    assert!(record.active());
    assert!(record.job().terminal.is_none());
    assert_eq!(
        record.job().first_failure.expect("clock cause").category,
        NoiseCategory::ClockInvalid
    );
    assert_eq!(record.job().first_failure.expect("clock cause").at_us, None);
}
