use super::*;

#[test]
fn resumable_duration_does_not_expire_before_first_active_state() {
    // Arrange
    let mut readiness = ready();
    readiness.maybe_campaign_lease = Some(resumable_lease(9, 200, 100));
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    adapter.drive(wake(readiness, 0));

    // Act
    adapter.drive(wake(readiness, 100));

    // Assert
    assert_eq!(
        adapter.session.snapshot().campaign_state,
        MiningCampaignState::Armed
    );
}

#[test]
fn resumable_activation_timeout_fails_closed_at_exact_boundary() {
    // Arrange
    let mut readiness = ready();
    readiness.maybe_campaign_lease = Some(resumable_lease(9, 100, 1_000));
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    adapter.drive(wake(readiness, 0));

    // Act
    adapter.drive(wake(readiness, 99));
    let before_timeout = adapter.session.snapshot();
    adapter.drive(wake(readiness, 100));
    let at_timeout = adapter.session.snapshot();

    // Assert
    assert_eq!(before_timeout.campaign_state, MiningCampaignState::Armed);
    assert_eq!(at_timeout.campaign_state, MiningCampaignState::Consumed);
    assert_eq!(
        at_timeout.maybe_blocker,
        Some(ProductionSessionBlocker::CampaignActivationTimedOut)
    );
}

#[test]
fn resumable_duration_starts_at_first_active_state() {
    // Arrange
    let mut readiness = ready();
    readiness.maybe_campaign_lease = Some(resumable_lease(9, 100, 10));
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active_with_readiness(&mut adapter, readiness);

    // Act
    adapter.drive(wake(readiness, 11));
    let before_expiry = adapter.session.snapshot();
    adapter.drive(wake(readiness, 12));
    let at_expiry = adapter.session.snapshot();

    // Assert
    assert_eq!(before_expiry.campaign_state, MiningCampaignState::Active);
    assert_eq!(at_expiry.campaign_state, MiningCampaignState::Consumed);
    assert_eq!(
        at_expiry.maybe_blocker,
        Some(ProductionSessionBlocker::CampaignLeaseConsumed)
    );
}
