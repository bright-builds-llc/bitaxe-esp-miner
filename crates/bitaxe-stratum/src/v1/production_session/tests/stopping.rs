use super::*;

#[test]
fn preparation_failure_rolls_back_before_terminal_publication() {
    // Arrange
    let mut session = ProductionMiningSession::new();
    let lease_id = active_duration_lease(1, 600_000).id();
    let _preparing = session
        .handle(wake(ready(), 0))
        .expect("readiness should request preparation");

    // Act
    let failed = session
        .handle(ProductionSessionEvent::HardwarePreparationFailed {
            lease_id,
            failure: HardwarePreparationFailure::DeviceFault,
            now_ms: 1,
        })
        .expect("preparation failure should enter safe stop");
    let confirmed = session
        .handle(ProductionSessionEvent::HardwareSafeStopConfirmed {
            lease_id,
            now_ms: 2,
        })
        .expect("safe-stop confirmation should publish terminal state");

    // Assert: before confirmation only the non-terminal `stopping` snapshot is published.
    assert!(matches!(
        failed.as_slice(),
        [
            ProductionSessionEffect::BlockSubmissions,
            ProductionSessionEffect::InvalidateWorkAndSubmissions,
            ProductionSessionEffect::StopAsicInteraction,
            ProductionSessionEffect::Publish(stopping),
            ProductionSessionEffect::SafeStopHardware { .. }
        ] if stopping.hardware_state == MiningHardwareState::SafeStopping
            && stopping.mining.mining_activity == MiningActivityStatus::Stopping
    ));
    assert!(matches!(
        confirmed.as_slice(),
        [ProductionSessionEffect::Publish(snapshot)]
            if snapshot.hardware_state == MiningHardwareState::Stopped
                && snapshot.campaign_state == MiningCampaignState::Consumed
    ));
}

#[test]
fn a_safe_stop_reads_stopping_until_it_is_confirmed() {
    // Arrange
    let mut session = ProductionMiningSession::new();
    let lease_id = active_duration_lease(1, 600_000).id();
    let _preparing = session
        .handle(wake(ready(), 0))
        .expect("readiness should request preparation");
    let _failed = session
        .handle(ProductionSessionEvent::HardwarePreparationFailed {
            lease_id,
            failure: HardwarePreparationFailure::DeviceFault,
            now_ms: 1,
        })
        .expect("preparation failure should enter safe stop");

    // Act
    let stopping = session.snapshot();
    let _confirmed = session
        .handle(ProductionSessionEvent::HardwareSafeStopConfirmed {
            lease_id,
            now_ms: 2,
        })
        .expect("safe-stop confirmation should publish terminal state");
    let stopped = session.snapshot();

    // Assert
    assert_eq!(
        stopping.mining.mining_activity,
        MiningActivityStatus::Stopping
    );
    assert_eq!(stopping.mining.work_submission, WorkSubmissionGate::Blocked);
    assert_eq!(
        stopped.mining.mining_activity,
        MiningActivityStatus::SafeBlocked
    );
    assert_eq!(stopped.campaign_state, MiningCampaignState::Consumed);
    assert_eq!(stopped.hardware_state, MiningHardwareState::Stopped);
}

#[test]
fn a_failed_safe_stop_publishes_blocked_with_its_reason_not_stopping() {
    // Arrange
    let mut session = ProductionMiningSession::new();
    let lease_id = active_duration_lease(1, 600_000).id();
    let _preparing = session
        .handle(wake(ready(), 0))
        .expect("readiness should request preparation");
    let _failed = session
        .handle(ProductionSessionEvent::HardwarePreparationFailed {
            lease_id,
            failure: HardwarePreparationFailure::DeviceFault,
            now_ms: 1,
        })
        .expect("preparation failure should enter safe stop");

    // Act
    let stop_failed = session
        .handle(ProductionSessionEvent::EffectFailed {
            maybe_pool: None,
            reason: HARDWARE_SAFE_STOP_FAILED,
            now_ms: 2,
        })
        .expect("a failed safe stop should be handled");

    // Assert
    assert!(stop_failed.iter().any(|effect| matches!(
        effect,
        ProductionSessionEffect::Publish(snapshot)
            if snapshot.mining.mining_activity == MiningActivityStatus::SafeBlocked
                && snapshot.mining.maybe_blocked_reason == Some("hardware_safe_stop_failed")
    )));
}

#[test]
fn another_failed_effect_during_a_safe_stop_is_no_failed_stop() {
    // Arrange: an ASIC stop failure queued ahead of the safe-stop's confirmation.
    let mut session = ProductionMiningSession::new();
    let lease_id = active_duration_lease(1, 600_000).id();
    let _preparing = session
        .handle(wake(ready(), 0))
        .expect("readiness should request preparation");
    let _failed = session
        .handle(ProductionSessionEvent::HardwarePreparationFailed {
            lease_id,
            failure: HardwarePreparationFailure::DeviceFault,
            now_ms: 1,
        })
        .expect("preparation failure should enter safe stop");

    // Act
    let asic_stop_failed = session
        .handle(ProductionSessionEvent::EffectFailed {
            maybe_pool: None,
            reason: "asic_stop_failed",
            now_ms: 2,
        })
        .expect("an ASIC stop failure should be handled");

    // Assert
    assert!(!asic_stop_failed
        .iter()
        .any(|effect| matches!(effect, ProductionSessionEffect::Publish(_))));
    assert_eq!(
        session.snapshot().mining.mining_activity,
        MiningActivityStatus::Stopping
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
        ProductionSessionEffect::Publish(stopping)
            if stopping.mining.mining_activity == MiningActivityStatus::Stopping
    ));
    assert!(matches!(
        ordered[5],
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

#[test]
fn an_operator_pause_publishes_stopping_before_its_safe_stop() {
    // Arrange
    let lease = resumable_lease(7, 1_000, 1_000);
    let running = ProductionReadiness {
        maybe_campaign_lease: Some(lease),
        ..ready()
    };
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active_with_readiness(&mut adapter, running);
    adapter.effects.clear();
    let paused = ProductionReadiness {
        operator_intent: MiningOperatorIntent::Paused,
        ..running
    };

    // Act
    adapter.drive(ProductionSessionEvent::Wake {
        wakeup: Some(ProductionSessionWakeup::OperatorIntentChanged),
        readiness: paused,
        now_ms: 100,
    });

    // Assert
    let stopping_at = adapter.effects.iter().position(|effect| {
        matches!(
            effect,
            ProductionSessionEffect::Publish(snapshot)
                if snapshot.mining.mining_activity == MiningActivityStatus::Stopping
        )
    });
    let safe_stop_at = adapter
        .effects
        .iter()
        .position(|effect| matches!(effect, ProductionSessionEffect::SafeStopHardware { .. }));
    assert!(
        matches!((stopping_at, safe_stop_at), (Some(stopping), Some(safe_stop)) if stopping < safe_stop)
    );
}
