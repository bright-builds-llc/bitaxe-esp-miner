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
    let requires_bootstrap = tracker.requires_requested_run_bootstrap();
    let startup = tracker.operator_intent(MiningOperatorIntent::Run);
    tracker.note_snapshot(&snapshot(MiningCampaignState::Active), 100);
    let paused = tracker.operator_intent(MiningOperatorIntent::Paused);
    let resumed = tracker.operator_intent(MiningOperatorIntent::Run);

    // Assert
    assert!(requires_bootstrap);
    assert_eq!(startup, MiningOperatorIntent::Run);
    assert_eq!(paused, MiningOperatorIntent::Paused);
    assert_eq!(resumed, MiningOperatorIntent::Run);
}

#[test]
fn ordinary_campaign_does_not_request_a_command_effects_bootstrap() {
    // Arrange
    let profile = MiningHardwareProfilePreset::Conservative;
    let lease = MiningCampaignLease::new(
        MiningCampaignLeaseId::new(11).expect("lease id"),
        profile.profile(),
        MiningCampaignStopCondition::ActiveDuration {
            duration: MiningCampaignDuration::new(600_000).expect("duration"),
        },
    );
    let tracker = CampaignStatusTracker::new(
        MiningCampaignStage::JobTransition,
        Some(lease),
        Some(profile),
    );

    // Act
    let requires_bootstrap = tracker.requires_requested_run_bootstrap();

    // Assert
    assert!(!requires_bootstrap);
}

#[test]
fn consumed_command_effects_lease_disables_bootstrap_and_forces_pause() {
    // Arrange
    let profile = MiningHardwareProfilePreset::Conservative;
    let lease = MiningCampaignLease::new(
        MiningCampaignLeaseId::new(12).expect("lease id"),
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
    tracker.note_snapshot(&snapshot(MiningCampaignState::Consumed), 600_000);
    let requires_bootstrap = tracker.requires_requested_run_bootstrap();
    let terminal_intent = tracker.operator_intent(MiningOperatorIntent::Run);

    // Assert
    assert!(!requires_bootstrap);
    assert_eq!(terminal_intent, MiningOperatorIntent::Paused);
}
