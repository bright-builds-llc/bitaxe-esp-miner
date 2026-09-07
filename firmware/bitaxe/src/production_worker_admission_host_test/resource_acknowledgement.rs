use super::*;

#[test]
fn start_acknowledgement_samples_active_owner_even_before_work_dispatch() {
    // Arrange
    let mut scope = TestScope::new();
    let mut adapter = blocked_adapter();
    adapter.readiness = readiness();
    let mut core = ProductionMiningSession::new();
    let (generation, receiver) = start(&mut scope, &mut adapter, &mut core);
    let mut snapshot = core.snapshot();
    snapshot.campaign_state = MiningCampaignState::Active;
    // Act
    adapter.complete_reply(&snapshot);
    // Assert
    assert_eq!(receiver.try_recv(), Ok(Ok(())));
    assert!(owner_resources::ACTIVE_CAPTURED.load(Ordering::SeqCst));
    assert_eq!(
        revocation::timing(1_000)
            .expect("active generation")
            .work_dispatched,
        0
    );
    assert!(revocation::permits(Some(generation)));
}
