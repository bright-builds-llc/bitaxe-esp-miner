use super::*;

#[test]
fn filter_counts_are_captured_before_terminal_release_and_reset_for_new_worker() {
    // Arrange
    let mut scope = TestScope::new();
    let mut adapter = blocked_adapter();
    adapter.readiness = readiness();
    let mut core = ProductionMiningSession::new();
    let (generation, _reply) = start(&mut scope, &mut adapter, &mut core);
    adapter
        .maybe_bwg_session
        .as_mut()
        .expect("owner")
        .expected_filter_counts = [5, 7];
    let mut terminal = core.snapshot();
    terminal.campaign_state = MiningCampaignState::Consumed;
    terminal.hardware_state = MiningHardwareState::Stopped;
    // Act
    adapter.complete_reply(&terminal);
    // Assert
    assert_eq!(
        *mining_progress::LAST.lock().expect("captured"),
        Some((generation.raw(), [5, 7]))
    );
    assert!(adapter.maybe_bwg_session.is_none());
    let mut fresh_core = ProductionMiningSession::new();
    let (fresh_generation, _reply) = start(&mut scope, &mut adapter, &mut fresh_core);
    assert_ne!(fresh_generation, generation);
    assert_eq!(
        adapter
            .maybe_bwg_session
            .as_ref()
            .expect("fresh owner")
            .expected_filter_counts,
        [0, 0]
    );
}

#[test]
fn new_owner_first_accepted_share_is_not_hidden_by_previous_pool_baseline() {
    // Arrange: real core pool counters restart at zero for each fresh runtime.
    let mut scope = TestScope::new();
    let generation = scope.link();
    assert!(revocation::admit_budget(generation, 180_000));
    let mut previous = ProductionMiningSession::new().snapshot();
    previous.campaign_state = MiningCampaignState::Consumed;
    previous.hardware_state = MiningHardwareState::Stopped;
    previous.mining.counters.accepted = 1;
    previous.mining.counters.qualified_candidates = 1;
    previous.lifetime_share_counters.accepted = 1;
    previous.lifetime_share_counters.qualified_candidates = 1;
    let mut adapter = blocked_adapter();
    let (reply, _receiver) = mpsc::sync_channel(1);
    adapter.event(
        bwg::OwnerCommand::Start {
            generation,
            worker_lease_id: "synthetic-next-lease".to_owned(),
            deadline: MiningCampaignMonotonicDeadline::new(61_000).expect("deadline"),
            pools: ProductionPoolSet {
                primary: None,
                fallback: None,
                prefer_fallback: false,
            },
            reply,
        },
        1_000,
        &previous,
        MiningCampaignLeaseId::new(2).ok(),
    );
    let mut current = previous.clone();
    current.campaign_state = MiningCampaignState::Active;
    current.lifetime_share_counters.accepted = 2;
    current.lifetime_share_counters.qualified_candidates = 2;
    // Act
    let result = adapter
        .maybe_bwg_session
        .as_ref()
        .expect("new owner")
        .publication_counts(&current);
    // Assert
    assert_eq!(result, [1, 0, 1]);
}
