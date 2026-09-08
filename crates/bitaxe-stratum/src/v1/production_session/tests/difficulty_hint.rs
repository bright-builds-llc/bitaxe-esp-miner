use super::*;
fn adapter(hint: Option<u16>) -> DeterministicProductionSessionAdapter {
    let mut pools = pools(false);
    pools
        .primary
        .as_mut()
        .expect("primary")
        .runtime
        .maybe_suggested_difficulty = hint;
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools));
    adapter.drive(wake(ready(), 0));
    adapter.connect(ProductionPool::Primary, 1);
    adapter
}
fn hints(adapter: &DeterministicProductionSessionAdapter) -> Vec<serde_json::Value> {
    adapter
        .writes
        .iter()
        .filter_map(|(_, line)| {
            let value: serde_json::Value = serde_json::from_str(line).expect("wire JSON");
            (value["method"] == "mining.suggest_difficulty").then_some(value)
        })
        .collect()
}
#[test]
fn nonzero_hint_is_queued_once_after_matching_authorization_and_does_not_set_target() {
    // Arrange
    let mut adapter = adapter(Some(1000));
    assert!(hints(&adapter).is_empty());
    // Act
    authorize_pool(&mut adapter, ProductionPool::Primary, 2);
    adapter.bytes(
        ProductionPool::Primary,
        "{\"id\":3,\"result\":true,\"error\":null}\n",
        3,
    );
    // Assert
    let hints = hints(&adapter);
    assert_eq!(hints.len(), 1);
    assert_eq!(hints[0]["params"], serde_json::json!([1000]));
    assert_eq!(hints[0]["id"], 4);
    assert!(adapter
        .session
        .snapshot()
        .mining
        .maybe_pool_difficulty
        .is_none());
}
#[test]
fn zero_absent_and_failed_authorization_never_suggest() {
    for hint in [None, Some(0)] {
        let mut adapter = adapter(hint);
        authorize_pool(&mut adapter, ProductionPool::Primary, 2);
        assert!(hints(&adapter).is_empty());
    }
    let mut adapter = adapter(Some(1000));
    adapter.bytes(
        ProductionPool::Primary,
        "{\"id\":2,\"result\":[[],\"4de05269\",8],\"error\":null}\n",
        2,
    );
    adapter.bytes(
        ProductionPool::Primary,
        "{\"id\":3,\"result\":false,\"error\":[24,\"fixture rejection\",null]}\n",
        3,
    );
    assert!(hints(&adapter).is_empty());
}
#[test]
fn advisory_success_and_rejection_do_not_count_shares_or_change_assigned_target() {
    // Arrange
    let mut adapter = adapter(Some(1000));
    authorize_pool(&mut adapter, ProductionPool::Primary, 2);
    let before = adapter.session.snapshot();
    // Act
    for result in ["true", "false"] {
        adapter.bytes(
            ProductionPool::Primary,
            format!("{{\"id\":4,\"result\":{result},\"error\":null}}\n"),
            3,
        );
    }
    // Assert
    let after = adapter.session.snapshot();
    assert_eq!(
        after.mining.counters.accepted,
        before.mining.counters.accepted
    );
    assert_eq!(
        after.mining.counters.rejected,
        before.mining.counters.rejected
    );
    assert_eq!(
        after.mining.maybe_pool_difficulty,
        before.mining.maybe_pool_difficulty
    );
    assert_eq!(after.campaign_state, before.campaign_state);
    assert_eq!(adapter.connections.len(), 1);
    adapter.bytes(
        ProductionPool::Primary,
        "{\"id\":null,\"method\":\"mining.set_difficulty\",\"params\":[1234]}\n",
        4,
    );
    assert_eq!(
        adapter
            .session
            .snapshot()
            .mining
            .maybe_pool_difficulty
            .expect("server target")
            .difficulty,
        1234.0
    );
}
#[test]
fn revoked_generation_does_not_queue_hint_from_a_late_authorization() {
    // Arrange
    let mut adapter = adapter(Some(1000));
    adapter.bytes(
        ProductionPool::Primary,
        "{\"id\":2,\"result\":[[],\"4de05269\",8],\"error\":null}\n",
        2,
    );
    // Act
    adapter.drive(ProductionSessionEvent::CampaignLeaseRevoked);
    let before = adapter.writes.len();
    adapter.bytes(
        ProductionPool::Primary,
        "{\"id\":3,\"result\":true,\"error\":null}\n",
        3,
    );
    // Assert
    assert!(hints(&adapter).is_empty());
    assert_eq!(adapter.writes.len(), before);
    assert_ne!(
        adapter.session.snapshot().campaign_state,
        MiningCampaignState::Active
    );
}
#[test]
fn each_new_pool_runtime_suggests_only_its_own_hint_after_authorization() {
    for (pool_kind, value) in [
        (ProductionPool::Primary, 1000),
        (ProductionPool::Fallback, 2000),
    ] {
        let mut configured = pools(pool_kind == ProductionPool::Fallback);
        configured
            .primary
            .as_mut()
            .expect("primary")
            .runtime
            .maybe_suggested_difficulty = Some(1000);
        configured
            .fallback
            .as_mut()
            .expect("fallback")
            .runtime
            .maybe_suggested_difficulty = Some(2000);
        let mut adapter = DeterministicProductionSessionAdapter::new(Some(configured));
        adapter.drive(wake(ready(), 0));
        adapter.connect(pool_kind, 1);
        assert!(hints(&adapter).is_empty());
        authorize_pool(&mut adapter, pool_kind, 2);
        assert_eq!(hints(&adapter)[0]["params"], serde_json::json!([value]));
        assert_eq!(hints(&adapter).len(), 1);
    }
}
#[test]
fn a_new_lease_runtime_gets_one_new_hint_after_its_own_authorization() {
    // Arrange
    let mut adapter = adapter(Some(1000));
    authorize_pool(&mut adapter, ProductionPool::Primary, 2);
    let old_epoch = adapter.latest_transport_epoch(ProductionPool::Primary);
    adapter.drive(ProductionSessionEvent::CampaignLeaseRevoked);
    // Act
    adapter.drive(wake(
        ProductionReadiness {
            maybe_campaign_lease: Some(monotonic_deadline_lease(2, 61000)),
            ..ready()
        },
        1000,
    ));
    adapter.connect(ProductionPool::Primary, 1001);
    let new_epoch = adapter.latest_transport_epoch(ProductionPool::Primary);
    // A late authorization from the retired transport must not emit the next hint.
    adapter.drive(ProductionSessionEvent::TransportBytes {
        pool: ProductionPool::Primary,
        transport_epoch: old_epoch,
        bytes: b"{\"id\":3,\"result\":true,\"error\":null}\n".to_vec(),
        now_ms: 1002,
    });
    assert_eq!(hints(&adapter).len(), 1);
    authorize_pool(&mut adapter, ProductionPool::Primary, 1003);
    // Assert
    assert_ne!(old_epoch, new_epoch);
    assert_eq!(hints(&adapter).len(), 2);
    assert!(hints(&adapter).iter().all(|hint| hint["id"] == 4));
}
#[test]
fn hint_reply_cannot_acknowledge_the_first_real_share() {
    // Arrange
    let mut configured = pools(false);
    configured
        .primary
        .as_mut()
        .expect("primary")
        .runtime
        .maybe_suggested_difficulty = Some(1000);
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(configured));
    establish_active(&mut adapter);
    let observation = dispatched_observation(&adapter);
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation,
        now_ms: 4,
    });
    let submitted: serde_json::Value = adapter
        .writes
        .iter()
        .filter_map(|(_, line)| {
            let value: serde_json::Value = serde_json::from_str(line).expect("wire");
            (value["method"] == "mining.submit").then_some(value)
        })
        .next()
        .expect("real submit");
    assert_eq!(submitted["id"], 5);
    // Act
    adapter.bytes(
        ProductionPool::Primary,
        "{\"id\":4,\"result\":true,\"error\":null}\n",
        5,
    );
    // Assert
    assert_eq!(adapter.session.snapshot().mining.counters.accepted, 0);
    adapter.bytes(
        ProductionPool::Primary,
        "{\"id\":5,\"result\":true,\"error\":null}\n",
        6,
    );
    assert_eq!(adapter.session.snapshot().mining.counters.accepted, 1);
}
