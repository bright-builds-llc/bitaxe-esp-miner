use super::*;

#[test]
fn explicit_fallback_preference_does_not_schedule_primary_probe() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(true)));
    adapter.drive(wake(ready(), 0));
    adapter.connect(ProductionPool::Fallback, 1);
    authorize_pool(&mut adapter, ProductionPool::Fallback, 2);
    adapter.connections.clear();

    // Act
    adapter.drive(wake(ready(), PRIMARY_INITIAL_PROBE_DELAY_MS + 2));

    // Assert
    assert!(adapter.connections.is_empty());
    assert_eq!(
        adapter.session.snapshot().maybe_active_pool,
        Some(ProductionPool::Fallback)
    );
}

#[test]
fn retry_budgets_exhaust_primary_then_fallback_and_recovery_probe() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    adapter.drive(wake(ready(), 0));

    // Act
    for attempt in 0..CONNECTION_ATTEMPTS_PER_POOL {
        let now_ms = u64::from(attempt) * CONNECTION_RETRY_DELAY_MS;
        adapter.drive(ProductionSessionEvent::TransportConnectFailed {
            pool: ProductionPool::Primary,
            now_ms,
        });
        if attempt + 1 < CONNECTION_ATTEMPTS_PER_POOL {
            adapter.drive(wake(ready(), now_ms + CONNECTION_RETRY_DELAY_MS));
        }
    }
    assert_eq!(adapter.connections.last(), Some(&ProductionPool::Fallback));
    for attempt in 0..CONNECTION_ATTEMPTS_PER_POOL {
        let now_ms = 20_000 + u64::from(attempt) * CONNECTION_RETRY_DELAY_MS;
        adapter.drive(ProductionSessionEvent::TransportConnectFailed {
            pool: ProductionPool::Fallback,
            now_ms,
        });
        if attempt + 1 < CONNECTION_ATTEMPTS_PER_POOL {
            adapter.drive(wake(ready(), now_ms + CONNECTION_RETRY_DELAY_MS));
        }
    }
    let paused_at = 30_000;
    let before = adapter.connections.len();
    adapter.drive(wake(ready(), paused_at + RECOVERY_PROBE_DELAY_MS - 1));
    let before_due = adapter.connections.len();
    adapter.drive(wake(ready(), paused_at + RECOVERY_PROBE_DELAY_MS));

    // Assert
    assert_eq!(
        adapter.session.snapshot().phase,
        ProductionSessionPhase::ConnectingPrimary
    );
    assert_eq!(before, before_due);
    assert_eq!(adapter.connections.last(), Some(&ProductionPool::Primary));
}

#[test]
fn automatic_fallback_probe_keeps_fallback_until_primary_authorizes() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_automatic_fallback(&mut adapter);
    let fallback_generation = adapter.session.snapshot().generation;
    adapter.connections.clear();

    // Act
    adapter.drive(wake(ready(), 11_001 + PRIMARY_INITIAL_PROBE_DELAY_MS));
    assert_eq!(
        adapter.session.snapshot().maybe_active_pool,
        Some(ProductionPool::Fallback)
    );
    assert_eq!(
        adapter.session.snapshot().generation,
        fallback_generation,
        "background probes must not replace the active fallback generation"
    );
    adapter.connect(ProductionPool::Primary, 21_002);
    assert_eq!(adapter.session.snapshot().generation, fallback_generation);
    authorize_pool(&mut adapter, ProductionPool::Primary, 21_003);

    // Assert
    assert_eq!(adapter.connections, [ProductionPool::Primary]);
    assert_eq!(
        adapter.session.snapshot().maybe_active_pool,
        Some(ProductionPool::Primary)
    );
    assert_ne!(adapter.session.snapshot().generation, fallback_generation);
    let close_fallback = adapter.effects.iter().position(|effect| {
        matches!(
            effect,
            ProductionSessionEffect::ClosePoolConnection(ProductionPool::Fallback)
        )
    });
    let primary_publish = adapter.effects.iter().rposition(|effect| {
        matches!(
            effect,
            ProductionSessionEffect::Publish(snapshot)
                if snapshot.maybe_active_pool == Some(ProductionPool::Primary)
        )
    });
    assert!(matches!(
        (close_fallback, primary_publish),
        (Some(close), Some(publish)) if close < publish
    ));
}

#[test]
fn failed_primary_probe_does_not_disrupt_automatic_fallback() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_automatic_fallback(&mut adapter);
    let fallback_generation = adapter.session.snapshot().generation;
    adapter.effects.clear();
    adapter.drive(wake(ready(), 11_001 + PRIMARY_INITIAL_PROBE_DELAY_MS));
    adapter.connect(ProductionPool::Primary, 21_002);

    // Act
    adapter.bytes(ProductionPool::Primary, b"{malformed}\n", 21_003);

    // Assert
    assert_eq!(
        adapter.session.snapshot().phase,
        ProductionSessionPhase::RunningFallback
    );
    assert_eq!(
        adapter.session.snapshot().maybe_active_pool,
        Some(ProductionPool::Fallback)
    );
    assert_eq!(adapter.session.snapshot().generation, fallback_generation);
    assert!(adapter.effects.iter().any(|effect| {
        matches!(
            effect,
            ProductionSessionEffect::ClosePoolConnection(ProductionPool::Primary)
        )
    }));
    assert!(!adapter.effects.iter().any(|effect| {
        matches!(
            effect,
            ProductionSessionEffect::ClosePoolConnection(ProductionPool::Fallback)
                | ProductionSessionEffect::StopAsicInteraction
        )
    }));
}

#[test]
fn clean_jobs_and_reconnect_invalidate_stale_nonce_results() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    let stale = dispatched_observation(&adapter);
    let submit_count = adapter
        .writes
        .iter()
        .filter(|(_, line)| line.contains("mining.submit"))
        .count();

    // Act
    adapter.bytes(
        ProductionPool::Primary,
        concat!(
            "{\"id\":null,\"method\":\"mining.notify\",\"params\":[\"job-2\",",
            "\"0000000000000000000000000000000000000000000000000000000000000000\",",
            "\"ffffffff\",\"ffffffff\",[],\"20000004\",\"1705ae3a\",",
            "\"647025b6\",true]}\n"
        ),
        4,
    );
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation: stale,
        now_ms: 5,
    });

    // Assert
    assert_eq!(
        adapter
            .writes
            .iter()
            .filter(|(_, line)| line.contains("mining.submit"))
            .count(),
        submit_count
    );
    assert_eq!(adapter.session.snapshot().mining.counters.accepted, 0);
}

#[test]
fn cadence_regenerates_work_and_poll_timeout_is_non_terminal() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    let initial_dispatches = adapter.asic_commands.len();

    // Act
    adapter.drive(ProductionSessionEvent::AsicPollTimedOut { now_ms: 100 });
    adapter.drive(wake(ready(), 2_003));

    // Assert
    assert_eq!(
        adapter.session.snapshot().phase,
        ProductionSessionPhase::RunningPrimary
    );
    assert!(adapter.asic_commands.len() > initial_dispatches);
    assert!(!adapter
        .effects
        .iter()
        .any(|effect| matches!(effect, ProductionSessionEffect::ClosePoolConnection(_))));
}

#[test]
fn regressed_event_clock_keeps_polling_without_early_regeneration() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    let initial_dispatches = adapter.asic_commands.len();
    adapter.effects.clear();

    // Act
    adapter.drive(wake(ready(), 2));

    // Assert
    assert_eq!(adapter.asic_commands.len(), initial_dispatches);
    assert!(adapter
        .effects
        .iter()
        .any(|effect| matches!(effect, ProductionSessionEffect::PollAsic { .. })));
}

#[test]
fn maximum_event_timestamp_regenerates_without_overflow() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    let initial_dispatches = adapter.asic_commands.len();

    // Act
    adapter.drive(ProductionSessionEvent::AsicPollTimedOut { now_ms: u64::MAX });

    // Assert
    assert!(adapter.asic_commands.len() > initial_dispatches);
    assert_eq!(
        adapter.session.snapshot().phase,
        ProductionSessionPhase::RunningPrimary
    );
}

#[test]
fn pause_settings_change_and_shutdown_reread_authoritative_state() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    let paused = ProductionReadiness {
        operator_intent: MiningOperatorIntent::Paused,
        ..ready()
    };

    // Act
    adapter.drive(ProductionSessionEvent::Wake {
        wakeup: Some(ProductionSessionWakeup::OperatorIntentChanged),
        readiness: paused,
        now_ms: 10,
    });
    let paused_snapshot = adapter.session.snapshot();
    adapter.drive(ProductionSessionEvent::Wake {
        wakeup: Some(ProductionSessionWakeup::SettingsChanged),
        readiness: ready(),
        now_ms: 11,
    });
    adapter.drive(ProductionSessionEvent::Wake {
        wakeup: Some(ProductionSessionWakeup::ShutdownRequested),
        readiness: ready(),
        now_ms: 12,
    });

    // Assert
    assert_eq!(
        paused_snapshot.mining.mining_activity,
        MiningActivityStatus::Paused
    );
    assert!(adapter.pool_reads >= 2);
    assert_eq!(
        adapter.session.snapshot().phase,
        ProductionSessionPhase::Shutdown
    );
}

#[test]
fn safe_stop_effect_order_and_final_snapshot_are_idempotent() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    adapter.effects.clear();
    let blocked = ProductionReadiness {
        network_ready: false,
        ..ready()
    };

    // Act
    adapter.drive(ProductionSessionEvent::Wake {
        wakeup: Some(ProductionSessionWakeup::NetworkChanged),
        readiness: blocked,
        now_ms: 10,
    });
    let first = adapter.effects.clone();
    adapter.effects.clear();
    adapter.drive(ProductionSessionEvent::Wake {
        wakeup: Some(ProductionSessionWakeup::NetworkChanged),
        readiness: blocked,
        now_ms: 11,
    });

    // Assert
    let ordered: Vec<&ProductionSessionEffect> = first
        .iter()
        .filter(|effect| {
            matches!(
                effect,
                ProductionSessionEffect::BlockSubmissions
                    | ProductionSessionEffect::InvalidateWorkAndSubmissions
                    | ProductionSessionEffect::StopAsicInteraction
                    | ProductionSessionEffect::ClosePoolConnection(_)
                    | ProductionSessionEffect::Publish(_)
            )
        })
        .collect();
    assert!(matches!(
        ordered[0],
        ProductionSessionEffect::BlockSubmissions
    ));
    assert!(matches!(
        ordered[1],
        ProductionSessionEffect::InvalidateWorkAndSubmissions
    ));
    assert!(matches!(
        ordered[2],
        ProductionSessionEffect::StopAsicInteraction
    ));
    assert!(matches!(
        ordered[3],
        ProductionSessionEffect::ClosePoolConnection(_)
    ));
    assert!(matches!(
        ordered.last(),
        Some(ProductionSessionEffect::Publish(_))
    ));
    assert_eq!(
        adapter.session.snapshot().mining.work_submission,
        WorkSubmissionGate::Blocked
    );
    assert!(!adapter.effects.iter().any(|effect| matches!(
        effect,
        ProductionSessionEffect::StopAsicInteraction
            | ProductionSessionEffect::ClosePoolConnection(_)
    )));
}
