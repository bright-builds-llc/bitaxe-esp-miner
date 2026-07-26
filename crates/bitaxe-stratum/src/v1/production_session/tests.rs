use super::*;

fn ready() -> ProductionReadiness {
    ProductionReadiness {
        operator_intent: MiningOperatorIntent::Run,
        network_ready: true,
        stratum_v1_supported: true,
        safety_prerequisites_fresh: true,
        production_asic_ready: true,
        actuation_qualified: true,
    }
}

fn pools() -> ProductionPoolAvailability {
    ProductionPoolAvailability {
        primary_configured: true,
        fallback_configured: true,
        prefer_fallback: false,
    }
}

#[test]
fn readiness_gates_are_ordered_and_do_not_request_pool_configuration() {
    // Arrange
    let cases = [
        (
            ProductionReadiness {
                operator_intent: MiningOperatorIntent::Paused,
                ..ready()
            },
            ProductionSessionBlocker::OperatorPaused,
        ),
        (
            ProductionReadiness {
                network_ready: false,
                ..ready()
            },
            ProductionSessionBlocker::NetworkUnavailable,
        ),
        (
            ProductionReadiness {
                stratum_v1_supported: false,
                ..ready()
            },
            ProductionSessionBlocker::StratumV1Unsupported,
        ),
        (
            ProductionReadiness {
                safety_prerequisites_fresh: false,
                ..ready()
            },
            ProductionSessionBlocker::SafetyPrerequisitesStale,
        ),
        (
            ProductionReadiness {
                production_asic_ready: false,
                ..ready()
            },
            ProductionSessionBlocker::ProductionAsicUnavailable,
        ),
        (
            ProductionReadiness {
                actuation_qualified: false,
                ..ready()
            },
            ProductionSessionBlocker::ActuationUnqualified,
        ),
    ];

    // Act / Assert
    for (readiness, blocker) in cases {
        let mut session = ProductionMiningSession::new();
        let actions = session.on_wakeup(None, readiness, 0);
        assert_eq!(session.projection().maybe_blocker, Some(blocker));
        assert!(!actions.contains(&ProductionSessionAction::ReadPoolConfiguration));
        assert!(!actions
            .iter()
            .any(|action| matches!(action, ProductionSessionAction::ConnectPool(_))));
    }
}

#[test]
fn admitted_session_reads_pool_configuration_lazily_then_connects_primary() {
    // Arrange
    let mut session = ProductionMiningSession::new();

    // Act
    let wake_actions = session.on_wakeup(None, ready(), 0);
    let pool_actions = session.on_pool_configuration(pools());

    // Assert
    assert!(wake_actions.contains(&ProductionSessionAction::ReadPoolConfiguration));
    assert_eq!(
        pool_actions,
        vec![ProductionSessionAction::ConnectPool(
            ProductionPool::Primary
        )]
    );
}

#[test]
fn pool_preference_can_start_on_fallback() {
    // Arrange
    let mut session = ProductionMiningSession::new();
    let availability = ProductionPoolAvailability {
        prefer_fallback: true,
        ..pools()
    };

    // Act
    let actions = session.on_pool_configuration(availability);

    // Assert
    assert_eq!(
        actions,
        vec![ProductionSessionAction::ConnectPool(
            ProductionPool::Fallback
        )]
    );
}

#[test]
fn retry_budget_moves_to_fallback_then_recovery_pause() {
    // Arrange
    let mut session = ProductionMiningSession::new();
    session.on_pool_configuration(pools());

    // Act
    for attempt in 0..CONNECTION_ATTEMPTS_PER_POOL {
        session.on_connection_result(ProductionPool::Primary, false, u64::from(attempt) * 5_000);
        if attempt + 1 < CONNECTION_ATTEMPTS_PER_POOL {
            session.on_wakeup(None, ready(), u64::from(attempt + 1) * 5_000);
        }
    }
    assert_eq!(session.phase(), ProductionSessionPhase::ConnectingFallback);
    for attempt in 0..CONNECTION_ATTEMPTS_PER_POOL {
        session.on_connection_result(
            ProductionPool::Fallback,
            false,
            20_000 + u64::from(attempt) * 5_000,
        );
        if attempt + 1 < CONNECTION_ATTEMPTS_PER_POOL {
            session.on_wakeup(None, ready(), 25_000 + u64::from(attempt) * 5_000);
        }
    }

    // Assert
    assert_eq!(session.phase(), ProductionSessionPhase::RecoveryPaused);
    assert_eq!(
        session.projection().maybe_blocker,
        Some(ProductionSessionBlocker::PoolsExhausted)
    );
}

#[test]
fn recovery_pause_retries_after_thirty_seconds() {
    // Arrange
    let mut session = ProductionMiningSession::new();
    session.on_pool_configuration(ProductionPoolAvailability {
        primary_configured: true,
        fallback_configured: false,
        prefer_fallback: false,
    });
    session.on_connection_result(ProductionPool::Primary, false, 0);
    session.on_wakeup(None, ready(), 5_000);
    session.on_connection_result(ProductionPool::Primary, false, 5_000);
    session.on_wakeup(None, ready(), 10_000);
    session.on_connection_result(ProductionPool::Primary, false, 10_000);

    // Act
    let early = session.on_wakeup(None, ready(), 39_999);
    let due = session.on_wakeup(None, ready(), 40_000);

    // Assert
    assert!(early.is_empty());
    assert_eq!(
        due,
        vec![ProductionSessionAction::ConnectPool(
            ProductionPool::Primary
        )]
    );
}

#[test]
fn fallback_probes_primary_after_ten_seconds_and_then_sixty_seconds() {
    // Arrange
    let mut session = ProductionMiningSession::new();
    session.on_pool_configuration(pools());
    session.on_connection_result(ProductionPool::Primary, false, 0);
    session.on_wakeup(None, ready(), 5_000);
    session.on_connection_result(ProductionPool::Primary, false, 5_000);
    session.on_wakeup(None, ready(), 10_000);
    session.on_connection_result(ProductionPool::Primary, false, 10_000);
    session.on_connection_result(ProductionPool::Fallback, true, 10_000);

    // Act
    let first_probe = session.on_wakeup(None, ready(), 20_000);
    session.on_connection_result(ProductionPool::Primary, false, 20_000);
    let recurring_probe = session.on_wakeup(None, ready(), 80_000);

    // Assert
    assert_eq!(
        first_probe,
        vec![ProductionSessionAction::ConnectPool(
            ProductionPool::Primary
        )]
    );
    assert_eq!(session.phase(), ProductionSessionPhase::ConnectingPrimary);
    assert_eq!(
        recurring_probe,
        vec![ProductionSessionAction::ConnectPool(
            ProductionPool::Primary
        )]
    );
}

#[test]
fn settings_change_resets_attempts_and_rereads_configuration() {
    // Arrange
    let mut session = ProductionMiningSession::new();
    session.on_pool_configuration(pools());
    session.on_connection_result(ProductionPool::Primary, false, 0);

    // Act
    let actions = session.on_wakeup(Some(ProductionSessionWakeup::SettingsChanged), ready(), 1);

    // Assert
    assert!(actions.contains(&ProductionSessionAction::ReadPoolConfiguration));
    assert_eq!(session.phase(), ProductionSessionPhase::WaitingForReadiness);
}

#[test]
fn safe_stop_orders_effects_before_final_projection_and_is_idempotent() {
    // Arrange
    let mut session = ProductionMiningSession::new();
    session.on_pool_configuration(pools());
    session.on_connection_result(ProductionPool::Primary, true, 0);
    let blocked = ProductionReadiness {
        network_ready: false,
        ..ready()
    };

    // Act
    let first = session.on_wakeup(Some(ProductionSessionWakeup::NetworkChanged), blocked, 1);
    let repeated = session.on_wakeup(Some(ProductionSessionWakeup::NetworkChanged), blocked, 2);

    // Assert
    assert_eq!(
        &first[..4],
        &[
            ProductionSessionAction::BlockSubmissions,
            ProductionSessionAction::InvalidateWorkAndSubmissions,
            ProductionSessionAction::StopAsicInteraction,
            ProductionSessionAction::ClosePoolConnection,
        ]
    );
    assert!(matches!(
        first.last(),
        Some(ProductionSessionAction::Publish(_))
    ));
    assert!(repeated.is_empty());
}

#[test]
fn pause_and_readiness_recovery_reconnect_from_authoritative_state() {
    // Arrange
    let mut session = ProductionMiningSession::new();
    session.on_pool_configuration(pools());
    session.on_connection_result(ProductionPool::Primary, true, 0);
    let paused = ProductionReadiness {
        operator_intent: MiningOperatorIntent::Paused,
        ..ready()
    };
    session.on_wakeup(
        Some(ProductionSessionWakeup::OperatorIntentChanged),
        paused,
        1,
    );

    // Act
    let resumed = session.on_wakeup(
        Some(ProductionSessionWakeup::OperatorIntentChanged),
        ready(),
        2,
    );

    // Assert
    assert_eq!(
        resumed,
        vec![ProductionSessionAction::ConnectPool(
            ProductionPool::Primary
        )]
    );
}

#[test]
fn shutdown_safe_stops_and_cannot_resume() {
    // Arrange
    let mut session = ProductionMiningSession::new();
    session.on_pool_configuration(pools());
    session.on_connection_result(ProductionPool::Primary, true, 0);

    // Act
    let actions = session.on_wakeup(Some(ProductionSessionWakeup::ShutdownRequested), ready(), 1);
    let after_shutdown = session.on_wakeup(
        Some(ProductionSessionWakeup::OperatorIntentChanged),
        ready(),
        2,
    );

    // Assert
    assert_eq!(session.phase(), ProductionSessionPhase::Shutdown);
    assert!(matches!(
        actions.last(),
        Some(ProductionSessionAction::Publish(projection))
            if projection.phase == ProductionSessionPhase::Shutdown
    ));
    assert!(after_shutdown.is_empty());
}
