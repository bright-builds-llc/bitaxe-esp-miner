use super::*;

#[test]
fn terminal_expiry_consumes_a_lease_already_stopped_by_resumable_pause() {
    // Arrange
    let lease = resumable_lease(8, 2_000, 1_000);
    let running = ProductionReadiness {
        maybe_campaign_lease: Some(lease),
        ..ready()
    };
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active_with_readiness(&mut adapter, running);
    let paused = ProductionReadiness {
        operator_intent: MiningOperatorIntent::Paused,
        ..running
    };
    let pause_effects = adapter
        .session
        .handle(ProductionSessionEvent::Wake {
            wakeup: Some(ProductionSessionWakeup::OperatorIntentChanged),
            readiness: paused,
            now_ms: 1_001,
        })
        .expect("pause should begin a resumable safe stop");
    let confirmed_effects = adapter
        .session
        .handle(ProductionSessionEvent::HardwareSafeStopConfirmed {
            lease_id: lease.id(),
            now_ms: 1_003,
        })
        .expect("resumable safe stop should be confirmed");

    // Act
    let expired_effects = adapter
        .session
        .handle(wake(running, 1_004))
        .expect("expired stopped lease should become terminal");
    let snapshot = adapter.session.snapshot();

    // Assert
    assert!(pause_effects.iter().any(|effect| matches!(
        effect,
        ProductionSessionEffect::SafeStopHardware {
            purpose: HardwareSafeStopPurpose::ResumablePause,
            ..
        }
    )));
    assert!(confirmed_effects.iter().any(|effect| matches!(
        effect,
        ProductionSessionEffect::Publish(snapshot)
            if snapshot.campaign_state == MiningCampaignState::Armed
    )));
    assert!(!expired_effects
        .iter()
        .any(|effect| matches!(effect, ProductionSessionEffect::SafeStopHardware { .. })));
    assert_eq!(snapshot.campaign_state, MiningCampaignState::Consumed);
    assert_eq!(
        snapshot.maybe_blocker,
        Some(ProductionSessionBlocker::CampaignLeaseConsumed)
    );
}

#[test]
fn terminal_expiry_overtakes_a_pending_resumable_safe_stop() {
    // Arrange
    let lease = resumable_lease(9, 2_000, 1_000);
    let running = ProductionReadiness {
        maybe_campaign_lease: Some(lease),
        ..ready()
    };
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active_with_readiness(&mut adapter, running);
    let paused = ProductionReadiness {
        operator_intent: MiningOperatorIntent::Paused,
        ..running
    };
    adapter
        .session
        .handle(ProductionSessionEvent::Wake {
            wakeup: Some(ProductionSessionWakeup::OperatorIntentChanged),
            readiness: paused,
            now_ms: 1_001,
        })
        .expect("pause should begin a resumable safe stop");

    // Act
    adapter
        .session
        .handle(wake(running, 1_003))
        .expect("terminal expiry should supersede resumability");
    let confirmed_effects = adapter
        .session
        .handle(ProductionSessionEvent::HardwareSafeStopConfirmed {
            lease_id: lease.id(),
            now_ms: 1_004,
        })
        .expect("terminal safe stop should be confirmed");
    let snapshot = adapter.session.snapshot();

    // Assert
    assert!(confirmed_effects.iter().any(|effect| matches!(
        effect,
        ProductionSessionEffect::Publish(snapshot)
            if snapshot.campaign_state == MiningCampaignState::Consumed
    )));
    assert_eq!(snapshot.campaign_state, MiningCampaignState::Consumed);
    assert_eq!(
        snapshot.maybe_blocker,
        Some(ProductionSessionBlocker::CampaignLeaseConsumed)
    );
}
