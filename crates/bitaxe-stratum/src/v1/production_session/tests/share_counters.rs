use super::*;
fn queue_candidate(adapter: &mut DeterministicProductionSessionAdapter, now: u64) {
    adapter.bytes(
        ProductionPool::Primary,
        concat!(
            "{\"id\":null,\"method\":\"mining.set_difficulty\",\"params\":[1e-30]}\n",
            "{\"id\":null,\"method\":\"mining.notify\",\"params\":[\"counter-job\",",
            "\"0000000000000000000000000000000000000000000000000000000000000000\",",
            "\"ffffffff\",\"ffffffff\",[],\"20000004\",\"1705ae3a\",\"647025b5\",true]}\n"
        ),
        now,
    );
    let observation = dispatched_observation(adapter);
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation,
        now_ms: now + 1,
    });
}
#[test]
fn lifetime_counts_survive_rebase_and_pool_replacement_without_double_counting_old_epoch() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    let old_epoch = adapter.latest_transport_epoch(ProductionPool::Primary);
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation: dispatched_observation(&adapter),
        now_ms: 4,
    });
    adapter.bytes(
        ProductionPool::Primary,
        b"{\"id\":4,\"result\":true,\"error\":null}\n",
        5,
    );
    let first = adapter.session.snapshot().lifetime_share_counters;
    assert_eq!(first.accepted, 1);
    assert_eq!(first.qualified_candidates, 1);
    // Act: rebasing work identity does not recreate a counter runtime.
    adapter
        .session
        .rebase_runtime_generation(ProductionPool::Primary);
    assert_eq!(adapter.session.snapshot().lifetime_share_counters, first);
    adapter.drive(ProductionSessionEvent::TransportFailed {
        pool: ProductionPool::Primary,
        transport_epoch: old_epoch,
        failure: ProductionTransportFailure::Read,
        now_ms: 6,
    });
    let retry = 6 + CONNECTION_RETRY_DELAY_MS;
    adapter.drive(wake(ready(), retry));
    adapter.connect(ProductionPool::Primary, retry + 1);
    authorize_pool(&mut adapter, ProductionPool::Primary, retry + 2);
    assert_eq!(adapter.session.snapshot().mining.counters.accepted, 0);
    queue_candidate(&mut adapter, retry + 3);
    adapter.drive(ProductionSessionEvent::TransportBytes {
        pool: ProductionPool::Primary,
        transport_epoch: old_epoch,
        bytes: b"{\"id\":4,\"result\":true,\"error\":null}\n".to_vec(),
        now_ms: retry + 5,
    });
    assert_eq!(
        adapter.session.snapshot().lifetime_share_counters.accepted,
        1
    );
    adapter.bytes(
        ProductionPool::Primary,
        b"{\"id\":4,\"result\":true,\"error\":null}\n",
        retry + 6,
    );
    // Assert
    let final_counts = adapter.session.snapshot().lifetime_share_counters;
    assert_eq!(final_counts.accepted, 2);
    assert_eq!(final_counts.qualified_candidates, 2);
    assert_eq!(adapter.session.snapshot().mining.counters.accepted, 1);
}
#[test]
fn lifetime_rejected_count_changes_only_for_a_matched_submit_response() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation: dispatched_observation(&adapter),
        now_ms: 4,
    });
    let rejection = b"{\"id\":4,\"result\":false,\"error\":[23,\"synthetic-rejected\",null]}\n";
    // Act
    adapter.bytes(ProductionPool::Primary, rejection, 5);
    adapter.bytes(ProductionPool::Primary, rejection, 6);
    // Assert
    let counters = adapter.session.snapshot().lifetime_share_counters;
    assert_eq!(counters.rejected, 1);
    assert_eq!(counters.accepted, 0);
    assert_eq!(counters.qualified_candidates, 1);
}
