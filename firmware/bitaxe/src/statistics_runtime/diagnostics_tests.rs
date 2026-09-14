use super::*;
fn recording() -> StartupDiagnostic {
    let record = StartupDiagnostic::new();
    assert!(record.begin(8192));
    record.observe_before(
        2052,
        HeapObservation {
            free_bytes: 16000,
            largest_block_bytes: 12000,
        },
    );
    record
}
fn after() -> HeapObservation {
    HeapObservation {
        free_bytes: 7000,
        largest_block_bytes: 2000,
    }
}

#[test]
fn successful_spawn_retains_matching_capability_facts_through_activation() {
    // Arrange
    let record = recording();
    // Act
    record.finish(after(), Ok(()));
    let prepared = record.maybe_marker().expect("prepared marker");
    record.active();
    let active = record.maybe_marker().expect("active marker");
    // Assert
    assert!(prepared.contains("state=prepared errno=unavailable stack_bytes=8192 stack_caps=2052"));
    assert_eq!(active, prepared.replace("state=prepared", "state=active"));
    assert!(active.contains("before_free_bytes=16000 before_largest_block_bytes=12000 after_free_bytes=7000 after_largest_block_bytes=2000"));
    assert!(!record.begin(8192));
    assert_eq!(record.maybe_marker(), Some(active));
}

#[test]
fn raw_spawn_errno_is_distinct_from_unavailable_errno() {
    // Arrange
    let known = recording();
    let unavailable = recording();
    // Act
    known.finish(after(), Err(Some(12)));
    unavailable.finish(after(), Err(None));
    // Assert
    assert!(known
        .maybe_marker()
        .expect("known error")
        .contains("state=spawn_failed errno=12"));
    assert!(unavailable
        .maybe_marker()
        .expect("unknown error")
        .contains("state=spawn_failed errno=unavailable"));
}

#[test]
fn configuration_failure_never_mislabels_an_esp_error_as_pthread_errno() {
    // Arrange
    let record = StartupDiagnostic::new();
    assert!(record.begin(8192));
    // Act
    record.config_failed(8192);
    // Assert
    assert_eq!(record.maybe_marker().expect("configuration error"), "statistics_startup schema=v1 state=config_failed errno=unavailable stack_bytes=8192 stack_caps=unavailable before_free_bytes=unavailable before_largest_block_bytes=unavailable after_free_bytes=unavailable after_largest_block_bytes=unavailable redacted=true");
}

#[test]
fn cancellation_retains_the_successful_spawn_observation() {
    // Arrange
    let record = recording();
    record.finish(after(), Ok(()));
    // Act
    record.cancelled();
    // Assert
    assert!(record
        .maybe_marker()
        .expect("cancelled marker")
        .contains("state=cancelled errno=unavailable stack_bytes=8192 stack_caps=2052"));
}

#[test]
fn preparing_and_empty_records_do_not_publish_partial_measurements() {
    // Arrange
    let record = StartupDiagnostic::new();
    // Act / Assert
    assert!(record.maybe_marker().is_none());
    assert!(record.begin(8192));
    assert!(record.maybe_marker().is_none());
    record.observe_before(2052, after());
    assert!(record.maybe_marker().is_none());
}

#[test]
fn diagnostic_storage_and_worst_case_marker_remain_bounded() {
    // Arrange
    let record = StartupDiagnostic::new();
    let heap = HeapObservation {
        free_bytes: u32::MAX,
        largest_block_bytes: u32::MAX,
    };
    assert!(record.begin(u32::MAX));
    record.observe_before(u32::MAX, heap);
    // Act
    record.finish(heap, Err(Some(i32::MIN)));
    // Assert
    assert_eq!(std::mem::size_of::<StartupDiagnostic>(), 36);
    assert!(record.maybe_marker().expect("bounded marker").len() < 512);
}
