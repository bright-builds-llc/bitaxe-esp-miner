use super::*;

#[test]
fn second_lease_after_terminal_stop_reloads_pool_configuration_before_connecting() {
    // Arrange: retain the same real production core across two owner leases.
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    let first_epoch = adapter.latest_transport_epoch(ProductionPool::Primary);
    let old_work = dispatched_observation(&adapter);
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation: old_work,
        now_ms: 4,
    });
    adapter.bytes(
        ProductionPool::Primary,
        b"{\"id\":4,\"result\":true,\"error\":null}\n",
        5,
    );
    assert_eq!(adapter.session.snapshot().mining.counters.accepted, 1);
    adapter.drive(ProductionSessionEvent::CampaignLeaseRevoked);
    assert_eq!(
        adapter.session.snapshot().campaign_state,
        MiningCampaignState::Consumed
    );
    assert_eq!(
        adapter.session.snapshot().hardware_state,
        MiningHardwareState::Stopped
    );
    let before = adapter.effects.len();
    let mut replacement = pools(false);
    replacement
        .primary
        .as_mut()
        .expect("synthetic primary")
        .runtime
        .credentials
        .username = "synthetic-second-owner".to_owned();
    adapter.pools = Some(replacement);
    // Act
    adapter.drive(wake(
        ProductionReadiness {
            maybe_campaign_lease: Some(monotonic_deadline_lease(2, 61_000)),
            ..ready()
        },
        1_000,
    ));
    // Assert
    assert_eq!(
        adapter.session.snapshot().hardware_state,
        MiningHardwareState::Ready
    );
    assert_eq!(
        adapter.pool_reads, 2,
        "second prepared lease must reload discarded endpoint/credentials"
    );
    assert!(adapter.effects[before..]
        .iter()
        .any(|effect| matches!(effect, ProductionSessionEffect::ReadPoolConfiguration)));
    assert_eq!(adapter.connections.len(), 2);
    assert_ne!(
        adapter.latest_transport_epoch(ProductionPool::Primary),
        first_epoch
    );
    let writes_before_stale_callback = adapter.writes.len();
    adapter.drive(ProductionSessionEvent::TransportConnected {
        pool: ProductionPool::Primary,
        transport_epoch: first_epoch,
        now_ms: 1_001,
    });
    assert_eq!(
        adapter.writes.len(),
        writes_before_stale_callback,
        "late old connection cannot authorize new lease"
    );
    adapter.connect(ProductionPool::Primary, 1_001);
    authorize_pool(&mut adapter, ProductionPool::Primary, 1_002);
    assert_eq!(
        adapter.session.snapshot().campaign_state,
        MiningCampaignState::Active
    );
    assert!(
        adapter.writes[writes_before_stale_callback..]
            .iter()
            .any(|(_, line)| line.contains("synthetic-second-owner")),
        "second handshake uses current grant context"
    );
    adapter.bytes(
        ProductionPool::Primary,
        concat!(
            "{\"id\":null,\"method\":\"mining.set_difficulty\",\"params\":[1e-30]}\n",
            "{\"id\":null,\"method\":\"mining.notify\",\"params\":[\"second-job\",",
            "\"0000000000000000000000000000000000000000000000000000000000000000\",",
            "\"ffffffff\",\"ffffffff\",[],\"20000004\",\"1705ae3a\",",
            "\"647025b5\",true]}\n"
        ),
        1_003,
    );
    assert_eq!(
        adapter.asic_commands.len(),
        2,
        "new lease produces new ASIC work"
    );
    let writes_before_old_result = adapter.writes.len();
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation: old_work,
        now_ms: 1_004,
    });
    assert_eq!(
        adapter.writes.len(),
        writes_before_old_result,
        "retired lease result cannot submit"
    );
    let current_work = dispatched_observation(&adapter);
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation: current_work,
        now_ms: 1_005,
    });
    assert!(adapter.writes[writes_before_old_result..]
        .iter()
        .any(|(_, line)| line.contains("mining.submit")));
    adapter.bytes(
        ProductionPool::Primary,
        b"{\"id\":4,\"result\":true,\"error\":null}\n",
        1_006,
    );
    assert_eq!(
        adapter.session.snapshot().mining.counters.accepted,
        1,
        "second runtime's first acceptance is one again"
    );
    assert_eq!(
        adapter.session.snapshot().lifetime_share_counters.accepted,
        2
    );
    assert_eq!(
        adapter
            .session
            .snapshot()
            .lifetime_share_counters
            .qualified_candidates,
        2
    );
    adapter.drive(ProductionSessionEvent::CampaignLeaseRevoked);
    assert_eq!(
        adapter.session.snapshot().campaign_state,
        MiningCampaignState::Consumed
    );
    assert_eq!(
        adapter.session.snapshot().hardware_state,
        MiningHardwareState::Stopped
    );
    assert_eq!(
        adapter
            .session
            .next_campaign_lease_id()
            .expect("next lease")
            .raw(),
        3
    );
}

#[test]
fn discarding_lease_pool_cache_does_not_reopen_global_shutdown() {
    // Arrange
    let mut policy = crate::v1::recovery_policy::RecoveryPolicy::new();
    policy.on_wakeup(Some(ProductionSessionWakeup::ShutdownRequested), ready(), 0);
    // Act
    policy.discard_pool_configuration();
    let effects = policy.on_wakeup(None, ready(), 1);
    // Assert
    assert_eq!(policy.projection().phase, ProductionSessionPhase::Shutdown);
    assert!(effects.is_empty());
}
