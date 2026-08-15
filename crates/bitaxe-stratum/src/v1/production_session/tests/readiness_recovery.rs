use super::*;

#[test]
fn fresh_observation_wakeup_resumes_after_a_stale_resume_readiness_sample() {
    // Arrange
    let lease = resumable_lease(8, 1_000, 1_000);
    let running = ProductionReadiness {
        maybe_campaign_lease: Some(lease),
        ..ready()
    };
    let paused = ProductionReadiness {
        operator_intent: MiningOperatorIntent::Paused,
        ..running
    };
    let stale_resume = ProductionReadiness {
        safety_prerequisites_fresh: false,
        ..running
    };
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active_with_readiness(&mut adapter, running);
    adapter.drive(ProductionSessionEvent::Wake {
        wakeup: Some(ProductionSessionWakeup::OperatorIntentChanged),
        readiness: paused,
        now_ms: 100,
    });
    adapter.effects.clear();

    // Act
    adapter.drive(ProductionSessionEvent::Wake {
        wakeup: Some(ProductionSessionWakeup::OperatorIntentChanged),
        readiness: stale_resume,
        now_ms: 200,
    });
    let stale_effects = adapter.effects.clone();
    let stale_snapshot = adapter.session.snapshot();
    adapter.effects.clear();
    adapter.drive(ProductionSessionEvent::Wake {
        wakeup: Some(ProductionSessionWakeup::ObservationsChanged),
        readiness: running,
        now_ms: 201,
    });

    // Assert
    assert_eq!(
        stale_snapshot.maybe_blocker,
        Some(ProductionSessionBlocker::SafetyPrerequisitesStale)
    );
    assert!(!stale_effects
        .iter()
        .any(|effect| matches!(effect, ProductionSessionEffect::PrepareHardware { .. })));
    assert_eq!(
        adapter
            .effects
            .iter()
            .filter(|effect| matches!(effect, ProductionSessionEffect::PrepareHardware { .. }))
            .count(),
        1
    );
    assert_ne!(
        adapter.session.snapshot().maybe_blocker,
        Some(ProductionSessionBlocker::SafetyPrerequisitesStale)
    );
}

#[test]
fn stale_safety_after_reactivation_preparation_preserves_resumable_campaign() {
    // Arrange
    let lease = resumable_lease(8, 1_000, 1_000);
    let running = ProductionReadiness {
        maybe_campaign_lease: Some(lease),
        ..ready()
    };
    let paused = ProductionReadiness {
        operator_intent: MiningOperatorIntent::Paused,
        ..running
    };
    let stale = ProductionReadiness {
        safety_prerequisites_fresh: false,
        ..running
    };
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active_with_readiness(&mut adapter, running);
    adapter.drive(ProductionSessionEvent::Wake {
        wakeup: Some(ProductionSessionWakeup::OperatorIntentChanged),
        readiness: paused,
        now_ms: 100,
    });
    adapter.effects.clear();
    adapter.drive(ProductionSessionEvent::Wake {
        wakeup: Some(ProductionSessionWakeup::OperatorIntentChanged),
        readiness: running,
        now_ms: 200,
    });
    let prepared_snapshot = adapter.session.snapshot();
    adapter.effects.clear();

    // Act
    adapter.drive(ProductionSessionEvent::Wake {
        wakeup: Some(ProductionSessionWakeup::ObservationsChanged),
        readiness: stale,
        now_ms: 201,
    });
    let stale_effects = adapter.effects.clone();
    let stale_snapshot = adapter.session.snapshot();
    adapter.effects.clear();
    adapter.drive(ProductionSessionEvent::Wake {
        wakeup: Some(ProductionSessionWakeup::ObservationsChanged),
        readiness: running,
        now_ms: 202,
    });
    adapter.connect(ProductionPool::Primary, 203);
    authorize_pool(&mut adapter, ProductionPool::Primary, 204);

    // Assert
    assert_eq!(
        prepared_snapshot.phase,
        ProductionSessionPhase::ConnectingPrimary
    );
    assert_eq!(prepared_snapshot.hardware_state, MiningHardwareState::Ready);
    assert!(stale_effects.iter().any(|effect| matches!(
        effect,
        ProductionSessionEffect::SafeStopHardware {
            lease_id,
            purpose: HardwareSafeStopPurpose::ResumablePause,
        } if *lease_id == lease.id()
    )));
    assert_eq!(stale_snapshot.campaign_state, MiningCampaignState::Armed);
    assert_eq!(stale_snapshot.hardware_state, MiningHardwareState::Stopped);
    assert_eq!(
        adapter.session.snapshot().campaign_state,
        MiningCampaignState::Active
    );
}

#[test]
fn stale_safety_before_first_active_state_consumes_resumable_campaign() {
    // Arrange
    let lease = resumable_lease(8, 1_000, 1_000);
    let running = ProductionReadiness {
        maybe_campaign_lease: Some(lease),
        ..ready()
    };
    let stale = ProductionReadiness {
        safety_prerequisites_fresh: false,
        ..running
    };
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    adapter.drive(wake(running, 0));
    adapter.effects.clear();

    // Act
    adapter.drive(ProductionSessionEvent::Wake {
        wakeup: Some(ProductionSessionWakeup::ObservationsChanged),
        readiness: stale,
        now_ms: 1,
    });

    // Assert
    assert!(adapter.effects.iter().any(|effect| matches!(
        effect,
        ProductionSessionEffect::SafeStopHardware {
            lease_id,
            purpose: HardwareSafeStopPurpose::Terminal,
        } if *lease_id == lease.id()
    )));
    assert_eq!(
        adapter.session.snapshot().campaign_state,
        MiningCampaignState::Consumed
    );
}

#[test]
fn stale_safety_while_active_consumes_resumable_campaign() {
    // Arrange
    let lease = resumable_lease(8, 1_000, 1_000);
    let running = ProductionReadiness {
        maybe_campaign_lease: Some(lease),
        ..ready()
    };
    let stale = ProductionReadiness {
        safety_prerequisites_fresh: false,
        ..running
    };
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active_with_readiness(&mut adapter, running);
    adapter.effects.clear();

    // Act
    adapter.drive(ProductionSessionEvent::Wake {
        wakeup: Some(ProductionSessionWakeup::ObservationsChanged),
        readiness: stale,
        now_ms: 100,
    });

    // Assert
    assert!(adapter.effects.iter().any(|effect| matches!(
        effect,
        ProductionSessionEffect::SafeStopHardware {
            lease_id,
            purpose: HardwareSafeStopPurpose::Terminal,
        } if *lease_id == lease.id()
    )));
    assert_eq!(
        adapter.session.snapshot().campaign_state,
        MiningCampaignState::Consumed
    );
}
