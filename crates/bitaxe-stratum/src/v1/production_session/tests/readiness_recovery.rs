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
