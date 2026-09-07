use super::*;
#[test]
fn below_target_nonce_is_visible_without_qualified_candidate() {
    // Arrange
    let mut snapshot =
        bitaxe_stratum::v1::production_session::ProductionMiningSession::new().snapshot();
    snapshot.asic_bridge.nonce_completion_count = 7;
    snapshot.mining.counters.below_pool_target = 7;
    // Act
    let value = project(3, 123, counts(&snapshot));
    // Assert
    assert_eq!(value["poll_nonce"], "7");
    assert_eq!(value["below_pool_target"], "7");
    assert_eq!(value["qualified_candidates"], "0");
}
#[test]
fn cache_preserves_u64_and_rejects_wrong_generation_without_time_gate() {
    // Arrange
    let cache = Cache::new();
    cache.publish(3, u64::MAX, [u64::MAX; COUNT]);
    // Act / Assert
    assert_eq!(cache.read(3), Some((u64::MAX, [u64::MAX; COUNT])));
    assert!(cache.read(2).is_none());
    assert_eq!(
        project(3, u64::MAX, [u64::MAX; COUNT])["poll_nonce"],
        "18446744073709551615"
    );
}
#[test]
fn projection_never_serializes_arbitrary_runtime_rejection_text() {
    // Arrange
    let mut snapshot =
        bitaxe_stratum::v1::production_session::ProductionMiningSession::new().snapshot();
    snapshot
        .mining
        .counters
        .rejected_reasons
        .push("synthetic-private-text".to_owned());
    snapshot.asic_bridge.discards.invalid_crc = 2;
    snapshot.asic_bridge.blocked_correlations.work_stale = 5;
    // Act
    let value = project(7, 100, counts(&snapshot));
    // Assert
    assert_eq!(value["discards"]["invalid_crc"], "2");
    assert_eq!(value["blocked"]["work_stale"], "5");
    assert!(!value.to_string().contains("synthetic-private-text"));
}
