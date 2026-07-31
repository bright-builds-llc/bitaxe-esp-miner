use super::*;

fn notify(job: &str, previous_block_byte: &str, clean_jobs: bool) -> String {
    format!(
        concat!(
            "{{\"id\":null,\"method\":\"mining.notify\",\"params\":[\"{}\",",
            "\"{}\",\"ffffffff\",\"ffffffff\",[],\"20000004\",",
            "\"1705ae3a\",\"647025b5\",{}]}}\n"
        ),
        job,
        previous_block_byte.repeat(32),
        clean_jobs
    )
}

#[test]
fn clean_new_block_dispatches_and_correlates_replacement_generation() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    let stale_observation = dispatched_observation(&adapter);

    // Act
    adapter.bytes(
        ProductionPool::Primary,
        notify("replacement", "11", true),
        4,
    );
    let replacement_observation = dispatched_observation(&adapter);
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation: stale_observation,
        now_ms: 5,
    });
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation: replacement_observation,
        now_ms: 6,
    });

    // Assert
    let evidence = adapter.session.snapshot().job_transition;
    assert_eq!(evidence.previous_block_change_count, 1);
    assert_eq!(evidence.new_block_generation_count, 1);
    assert_eq!(evidence.replacement_dispatch_count, 1);
    assert_eq!(evidence.replacement_result_count, 1);
    assert_eq!(evidence.completed_transition_count, 1);
    assert_eq!(evidence.stale_generation_result_discard_count, 1);
    assert_eq!(evidence.stale_generation_submit_count, 0);
}

#[test]
fn same_block_clean_refresh_keeps_replacement_result_in_transition_lineage() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    adapter.bytes(ProductionPool::Primary, notify("new-block", "11", true), 4);

    // Act
    adapter.bytes(
        ProductionPool::Primary,
        notify("same-block-refresh", "11", true),
        5,
    );
    let refreshed_observation = dispatched_observation(&adapter);
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation: refreshed_observation,
        now_ms: 6,
    });

    // Assert
    let evidence = adapter.session.snapshot().job_transition;
    assert_eq!(evidence.previous_block_change_count, 1);
    assert_eq!(evidence.new_block_generation_count, 1);
    assert_eq!(evidence.replacement_dispatch_count, 2);
    assert_eq!(evidence.replacement_result_count, 1);
    assert_eq!(evidence.completed_transition_count, 1);
}

#[test]
fn changed_block_without_clean_jobs_begins_terminal_safe_stop() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);

    // Act
    adapter.bytes(
        ProductionPool::Primary,
        notify("inconsistent", "22", false),
        4,
    );

    // Assert
    let snapshot = adapter.session.snapshot();
    assert_eq!(
        snapshot.maybe_blocker,
        Some(ProductionSessionBlocker::JobTransitionProtocolInconsistent)
    );
    assert_eq!(snapshot.campaign_state, MiningCampaignState::Consumed);
    assert_eq!(snapshot.job_transition.completed_transition_count, 0);
}

#[test]
fn same_block_refresh_and_reconnect_are_distinct_evidence() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    let transport_epoch = adapter.latest_transport_epoch(ProductionPool::Primary);

    // Act
    adapter.bytes(ProductionPool::Primary, notify("refresh", "00", false), 4);
    adapter.drive(ProductionSessionEvent::TransportClosed {
        pool: ProductionPool::Primary,
        transport_epoch,
        now_ms: 5,
    });

    // Assert
    let evidence = adapter.session.snapshot().job_transition;
    assert_eq!(evidence.pool_notify_count, 2);
    assert_eq!(evidence.previous_block_change_count, 0);
    assert_eq!(evidence.completed_transition_count, 0);
    assert_eq!(evidence.reconnect_count, 1);
}
