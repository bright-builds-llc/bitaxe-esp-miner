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
fn stale_old_generation_poll_completion_rearms_replacement_generation_poll() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    let old_generation = adapter.session.snapshot().generation;
    adapter.drive(wake(ready(), 4));
    assert!(adapter.effects.iter().any(|effect| matches!(
        effect,
        ProductionSessionEffect::PollAsic { generation, .. }
            if *generation == old_generation
    )));

    // Act
    adapter.bytes(
        ProductionPool::Primary,
        notify("replacement", "11", true),
        5,
    );
    let replacement_generation = adapter.session.snapshot().generation;
    let effects_before_stale_completion = adapter.effects.len();
    adapter.drive(ProductionSessionEvent::AsicPollTimedOut {
        generation: old_generation,
        now_ms: 6,
    });
    adapter.drive(wake(ready(), 7));

    // Assert
    assert_ne!(replacement_generation, old_generation);
    assert!(adapter.effects[effects_before_stale_completion..]
        .iter()
        .any(|effect| matches!(
            effect,
            ProductionSessionEffect::PollAsic { generation, .. }
                if *generation == replacement_generation
        )));
    let diagnostics = adapter.session.snapshot().asic_bridge;
    assert_eq!(diagnostics.stale_completion_count, 1);
    assert_eq!(diagnostics.post_transition_poll_request_count, 1);
    assert_eq!(
        diagnostics.changed_block_to_replacement_dispatch_ms,
        Some(0)
    );
    assert_eq!(diagnostics.changed_block_to_first_poll_ms, Some(2));
    assert_eq!(diagnostics.final_poll_state, AsicPollState::InFlight);
}

#[test]
fn stale_nonce_completion_does_not_clear_newer_poll_in_flight() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    let stale_observation = dispatched_observation(&adapter);
    adapter.drive(wake(ready(), 4));
    adapter.bytes(
        ProductionPool::Primary,
        notify("replacement", "11", true),
        5,
    );
    let replacement_generation = adapter.session.snapshot().generation;
    adapter.drive(wake(ready(), 6));
    let replacement_poll_count = adapter
        .effects
        .iter()
        .filter(|effect| {
            matches!(
                effect,
                ProductionSessionEffect::PollAsic { generation, .. }
                    if *generation == replacement_generation
            )
        })
        .count();

    // Act
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation: stale_observation,
        now_ms: 7,
    });
    adapter.drive(wake(ready(), 8));

    // Assert
    let poll_count_after_stale_nonce = adapter
        .effects
        .iter()
        .filter(|effect| {
            matches!(
                effect,
                ProductionSessionEffect::PollAsic { generation, .. }
                    if *generation == replacement_generation
            )
        })
        .count();
    assert_eq!(replacement_poll_count, 1);
    assert_eq!(poll_count_after_stale_nonce, replacement_poll_count);
    assert_eq!(
        adapter
            .session
            .snapshot()
            .asic_bridge
            .stale_completion_count,
        1
    );
}

#[test]
fn same_block_clean_refresh_invalidates_old_poll_and_rearms_current_generation() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    let old_generation = adapter.session.snapshot().generation;
    adapter.drive(wake(ready(), 4));

    // Act
    adapter.bytes(
        ProductionPool::Primary,
        notify("same-block-clean", "00", true),
        5,
    );
    let refreshed_generation = adapter.session.snapshot().generation;
    adapter.drive(ProductionSessionEvent::AsicPollTimedOut {
        generation: old_generation,
        now_ms: 6,
    });
    adapter.drive(wake(ready(), 7));

    // Assert
    let snapshot = adapter.session.snapshot();
    assert_eq!(snapshot.job_transition.previous_block_change_count, 0);
    assert_eq!(snapshot.asic_bridge.generation_invalidation_count, 2);
    assert!(adapter.effects.iter().any(|effect| matches!(
        effect,
        ProductionSessionEffect::PollAsic { generation, .. }
            if *generation == refreshed_generation
    )));
}

#[test]
fn repeated_clean_notifications_rearm_each_successive_generation() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    adapter.drive(wake(ready(), 4));
    adapter.bytes(ProductionPool::Primary, notify("new-block", "11", true), 5);
    let first_replacement_generation = adapter.session.snapshot().generation;
    adapter.drive(wake(ready(), 6));

    // Act
    adapter.bytes(
        ProductionPool::Primary,
        notify("same-block-refresh", "11", true),
        7,
    );
    let second_replacement_generation = adapter.session.snapshot().generation;
    adapter.drive(ProductionSessionEvent::AsicPollTimedOut {
        generation: first_replacement_generation,
        now_ms: 8,
    });
    adapter.drive(wake(ready(), 9));

    // Assert
    let evidence = adapter.session.snapshot().asic_bridge;
    assert_ne!(first_replacement_generation, second_replacement_generation);
    assert_eq!(evidence.stale_completion_count, 1);
    assert!(adapter.effects.iter().any(|effect| matches!(
        effect,
        ProductionSessionEffect::PollAsic { generation, .. }
            if *generation == second_replacement_generation
    )));
}

#[test]
fn post_transition_parser_discard_is_counted_by_closed_subtype() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    adapter.bytes(ProductionPool::Primary, notify("new-block", "11", true), 4);
    let generation = adapter.session.snapshot().generation;
    adapter.drive(wake(ready(), 5));

    // Act
    adapter.drive(ProductionSessionEvent::AsicPollCompleted {
        generation,
        completion: AsicPollCompletion::Discarded(
            bitaxe_asic::bm1366::result::Bm1366ResultDiscardReason::JobLookup,
        ),
        now_ms: 6,
    });

    // Assert
    let evidence = adapter.session.snapshot().asic_bridge;
    assert_eq!(evidence.discards.job_lookup, 1);
    assert_eq!(evidence.post_transition_completion_count, 1);
    assert_eq!(evidence.post_transition_nonce_emission_count, 0);
}

#[test]
fn post_transition_blocked_correlation_is_counted_by_closed_reason() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    adapter.bytes(ProductionPool::Primary, notify("new-block", "11", true), 4);
    let mut observation = dispatched_observation(&adapter);
    observation.result.job_id = bitaxe_asic::bm1366::work::Bm1366JobId::new(0x30);

    // Act
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation,
        now_ms: 5,
    });

    // Assert
    let evidence = adapter.session.snapshot().asic_bridge;
    assert_eq!(evidence.blocked_correlation_count, 1);
    assert_eq!(evidence.blocked_correlations.job_lookup, 1);
    assert_eq!(evidence.post_transition_correlation_count, 0);
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
