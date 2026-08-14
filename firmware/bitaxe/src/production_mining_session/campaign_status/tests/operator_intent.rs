#[test]
fn lease_scoped_run_override_returns_to_paused_after_consumption() {
    // Arrange
    let profile = MiningHardwareProfilePreset::Conservative;
    let lease = MiningCampaignLease::new(
        MiningCampaignLeaseId::new(9).expect("lease id"),
        profile.profile(),
        MiningCampaignStopCondition::ActiveDuration {
            duration: MiningCampaignDuration::new(1_000).expect("duration"),
        },
    );
    let mut tracker =
        CampaignStatusTracker::new(MiningCampaignStage::Soak, Some(lease), Some(profile));

    // Act
    let during_lease = tracker.operator_intent(MiningOperatorIntent::Paused);
    tracker.note_snapshot(&snapshot(MiningCampaignState::Consumed), 1_000);
    let after_consumption = tracker.operator_intent(MiningOperatorIntent::Run);

    // Assert
    assert_eq!(during_lease, MiningOperatorIntent::Run);
    assert_eq!(after_consumption, MiningOperatorIntent::Paused);
}

#[test]
fn command_effects_stage_follows_operator_intent_after_first_active_snapshot() {
    // Arrange
    let profile = MiningHardwareProfilePreset::Conservative;
    let lease = MiningCampaignLease::new(
        MiningCampaignLeaseId::new(10).expect("lease id"),
        profile.profile(),
        MiningCampaignStopCondition::ResumableActiveEpoch {
            activation_timeout: MiningCampaignDuration::new(600_000)
                .expect("activation timeout"),
            duration: MiningCampaignDuration::new(600_000).expect("duration"),
        },
    );
    let mut tracker = CampaignStatusTracker::new(
        MiningCampaignStage::CommandEffects,
        Some(lease),
        Some(profile),
    );

    // Act
    let startup = tracker.operator_intent(MiningOperatorIntent::Paused);
    tracker.note_snapshot(&snapshot(MiningCampaignState::Active), 100);
    let paused = tracker.operator_intent(MiningOperatorIntent::Paused);
    let resumed = tracker.operator_intent(MiningOperatorIntent::Run);

    // Assert
    assert_eq!(startup, MiningOperatorIntent::Run);
    assert_eq!(paused, MiningOperatorIntent::Paused);
    assert_eq!(resumed, MiningOperatorIntent::Run);
}
