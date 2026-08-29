use super::*;

#[test]
fn lease_renewal_restarts_the_active_duration_for_the_same_campaign() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    let renewed = active_duration_lease(1, 60);

    // Act
    adapter.drive(ProductionSessionEvent::CampaignLeaseRenewed {
        lease: renewed,
        now_ms: 50,
    });
    adapter.drive(wake(
        ProductionReadiness {
            maybe_campaign_lease: Some(renewed),
            ..ready()
        },
        100,
    ));

    // Assert
    assert_eq!(
        adapter.session.snapshot().campaign_state,
        MiningCampaignState::Active
    );
}

#[test]
fn monotonic_deadline_expires_at_the_absolute_instant() {
    // Arrange
    let lease = monotonic_deadline_lease(1, 50);
    let readiness = ProductionReadiness {
        maybe_campaign_lease: Some(lease),
        ..ready()
    };
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active_with_readiness(&mut adapter, readiness);

    // Act
    adapter.drive(wake(readiness, 49));
    let before_deadline = adapter.session.snapshot().campaign_state;
    adapter.drive(wake(readiness, 50));

    // Assert
    assert_eq!(before_deadline, MiningCampaignState::Active);
    assert_eq!(
        adapter.session.snapshot().campaign_state,
        MiningCampaignState::Consumed
    );
}

#[test]
fn renewal_replaces_the_absolute_deadline_without_changing_identity() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    let renewed = monotonic_deadline_lease(1, 100);

    // Act
    adapter.drive(ProductionSessionEvent::CampaignLeaseRenewed {
        lease: renewed,
        now_ms: 50,
    });
    adapter.drive(wake(
        ProductionReadiness {
            maybe_campaign_lease: Some(renewed),
            ..ready()
        },
        99,
    ));
    let before_deadline = adapter.session.snapshot().campaign_state;
    adapter.drive(wake(
        ProductionReadiness {
            maybe_campaign_lease: Some(renewed),
            ..ready()
        },
        100,
    ));

    // Assert
    assert_eq!(before_deadline, MiningCampaignState::Active);
    assert_eq!(
        adapter.session.snapshot().campaign_state,
        MiningCampaignState::Consumed
    );
}

#[test]
fn owner_local_campaign_ids_advance_after_consumption() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    let initial = adapter.session.next_campaign_lease_id();
    establish_active(&mut adapter);

    // Act
    adapter.drive(ProductionSessionEvent::CampaignLeaseRevoked);
    let after_consumption = adapter.session.next_campaign_lease_id();

    // Assert
    assert_eq!(initial.map(MiningCampaignLeaseId::raw), Some(1));
    assert_eq!(after_consumption.map(MiningCampaignLeaseId::raw), Some(2));
}

#[test]
fn external_lease_revocation_completes_the_existing_safe_stop_ordering() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);

    // Act
    adapter.drive(ProductionSessionEvent::CampaignLeaseRevoked);

    // Assert
    let snapshot = adapter.session.snapshot();
    assert_eq!(snapshot.campaign_state, MiningCampaignState::Consumed);
    assert_eq!(snapshot.hardware_state, MiningHardwareState::Stopped);
    assert!(adapter.session.maybe_pool_set.is_none());
    assert!(adapter.session.primary.is_none());
    assert!(adapter.session.fallback.is_none());
    assert!(adapter
        .effects
        .iter()
        .any(|effect| { matches!(effect, ProductionSessionEffect::SafeStopHardware { .. }) }));
}
