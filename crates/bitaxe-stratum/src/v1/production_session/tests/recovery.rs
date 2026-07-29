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
fn retry_budgets_exhaust_primary_then_fallback_and_consume_lease() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    adapter.drive(wake(ready(), 0));

    // Act
    for attempt in 0..CONNECTION_ATTEMPTS_PER_POOL {
        let now_ms = u64::from(attempt) * CONNECTION_RETRY_DELAY_MS;
        adapter.fail_connect(ProductionPool::Primary, now_ms);
        if attempt + 1 < CONNECTION_ATTEMPTS_PER_POOL {
            adapter.drive(wake(ready(), now_ms + CONNECTION_RETRY_DELAY_MS));
        }
    }
    assert_eq!(adapter.connections.last(), Some(&ProductionPool::Fallback));
    for attempt in 0..CONNECTION_ATTEMPTS_PER_POOL {
        let now_ms = 20_000 + u64::from(attempt) * CONNECTION_RETRY_DELAY_MS;
        adapter.fail_connect(ProductionPool::Fallback, now_ms);
        if attempt + 1 < CONNECTION_ATTEMPTS_PER_POOL {
            adapter.drive(wake(ready(), now_ms + CONNECTION_RETRY_DELAY_MS));
        }
    }
    let before = adapter.connections.len();
    adapter.drive(wake(ready(), 30_000 + RECOVERY_PROBE_DELAY_MS));

    // Assert
    assert_eq!(
        adapter.session.snapshot().phase,
        ProductionSessionPhase::WaitingForReadiness
    );
    assert_eq!(
        adapter.session.snapshot().maybe_blocker,
        Some(ProductionSessionBlocker::CampaignLeaseConsumed)
    );
    assert_eq!(adapter.connections.len(), before);
    assert_eq!(
        adapter.session.snapshot().campaign_state,
        MiningCampaignState::Consumed
    );
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
            ProductionSessionEffect::ClosePoolConnection {
                pool: ProductionPool::Fallback,
                ..
            }
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
            ProductionSessionEffect::ClosePoolConnection {
                pool: ProductionPool::Primary,
                ..
            }
        )
    }));
    assert!(!adapter.effects.iter().any(|effect| {
        matches!(
            effect,
            ProductionSessionEffect::ClosePoolConnection {
                pool: ProductionPool::Fallback,
                ..
            } | ProductionSessionEffect::StopAsicInteraction
        )
    }));
}

#[test]
fn stale_primary_worker_feedback_cannot_mutate_its_replacement() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_automatic_fallback(&mut adapter);
    adapter.drive(wake(ready(), 11_001 + PRIMARY_INITIAL_PROBE_DELAY_MS));
    let stale_epoch = adapter.latest_transport_epoch(ProductionPool::Primary);
    adapter.connect(ProductionPool::Primary, 21_002);
    adapter.drive(ProductionSessionEvent::TransportFailed {
        pool: ProductionPool::Primary,
        transport_epoch: stale_epoch,
        failure: ProductionTransportFailure::Read,
        now_ms: 21_003,
    });
    adapter.drive(wake(ready(), 21_003 + PRIMARY_RECURRING_PROBE_DELAY_MS));
    let current_epoch = adapter.latest_transport_epoch(ProductionPool::Primary);
    adapter.connect(
        ProductionPool::Primary,
        21_004 + PRIMARY_RECURRING_PROBE_DELAY_MS,
    );
    let writes_before_stale_feedback = adapter.writes.len();

    // Act
    adapter.drive(ProductionSessionEvent::TransportBytes {
        pool: ProductionPool::Primary,
        transport_epoch: stale_epoch,
        bytes: b"{\"id\":1,\"result\":{\"version-rolling\":true,\"version-rolling.mask\":\"1fffe000\"},\"error\":null}\n".to_vec(),
        now_ms: 21_005 + PRIMARY_RECURRING_PROBE_DELAY_MS,
    });
    adapter.drive(ProductionSessionEvent::TransportFailed {
        pool: ProductionPool::Primary,
        transport_epoch: stale_epoch,
        failure: ProductionTransportFailure::Write,
        now_ms: 21_006 + PRIMARY_RECURRING_PROBE_DELAY_MS,
    });

    // Assert
    assert_ne!(current_epoch, stale_epoch);
    assert_eq!(adapter.writes.len(), writes_before_stale_feedback);
    assert!(adapter
        .session
        .transport_epoch_is_active(ProductionPool::Primary, current_epoch));
    assert_eq!(
        adapter.session.snapshot().maybe_active_pool,
        Some(ProductionPool::Fallback)
    );
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
    assert_eq!(
        adapter.session.snapshot().mining.work_submission,
        WorkSubmissionGate::Ready
    );
}

#[test]
fn stale_asic_timeout_and_failure_do_not_mutate_the_current_generation() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    let stale_generation = adapter.session.snapshot().generation;
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
    let current_generation = adapter.session.snapshot().generation;
    adapter.effects.clear();
    let dispatches_before = adapter.asic_commands.len();

    // Act
    adapter.drive(ProductionSessionEvent::AsicPollTimedOut {
        generation: stale_generation,
        now_ms: u64::MAX,
    });
    adapter.drive(ProductionSessionEvent::AsicInteractionFailed {
        generation: stale_generation,
        failure: ProductionAsicFailure::Poll,
        now_ms: u64::MAX,
    });

    // Assert
    assert_ne!(current_generation, stale_generation);
    assert!(adapter.effects.is_empty());
    assert_eq!(adapter.asic_commands.len(), dispatches_before);
    assert_eq!(
        adapter.session.snapshot().phase,
        ProductionSessionPhase::RunningPrimary
    );
}

#[test]
fn current_generation_asic_failure_enters_terminal_safe_stop() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    let generation = adapter.session.snapshot().generation;
    adapter.effects.clear();

    // Act
    adapter.drive(ProductionSessionEvent::AsicInteractionFailed {
        generation,
        failure: ProductionAsicFailure::Dispatch,
        now_ms: 4,
    });

    // Assert
    assert_eq!(
        adapter.session.snapshot().hardware_state,
        MiningHardwareState::Stopped
    );
    assert_eq!(
        adapter.session.snapshot().campaign_state,
        MiningCampaignState::Consumed
    );
    assert!(adapter
        .effects
        .iter()
        .any(|effect| matches!(effect, ProductionSessionEffect::SafeStopHardware { .. })));
}

#[test]
fn cadence_regenerates_work_and_poll_timeout_is_non_terminal() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    let initial_dispatches = adapter.asic_commands.len();
    let generation = adapter.session.snapshot().generation;

    // Act
    adapter.drive(ProductionSessionEvent::AsicPollTimedOut {
        generation,
        now_ms: 100,
    });
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
        .any(|effect| matches!(effect, ProductionSessionEffect::ClosePoolConnection { .. })));
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
    let generation = adapter.session.snapshot().generation;

    // Act
    adapter.drive(ProductionSessionEvent::AsicPollTimedOut {
        generation,
        now_ms: u64::MAX,
    });

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
    assert_eq!(adapter.pool_reads, 1);
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
                    | ProductionSessionEffect::ClosePoolConnection { .. }
                    | ProductionSessionEffect::SafeStopHardware { .. }
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
        ProductionSessionEffect::ClosePoolConnection { .. }
    ));
    assert!(matches!(
        ordered[4],
        ProductionSessionEffect::SafeStopHardware { .. }
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
            | ProductionSessionEffect::ClosePoolConnection { .. }
    )));
}
