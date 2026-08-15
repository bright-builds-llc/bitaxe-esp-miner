#[test]
fn command_effects_marker_closes_resumable_pause_only_after_hardware_stops() {
    // Arrange
    let profile = MiningHardwareProfilePreset::Conservative;
    let lease = MiningCampaignLease::new(
        MiningCampaignLeaseId::new(9).expect("lease id"),
        profile.profile(),
        MiningCampaignStopCondition::FirstSubmitResponse {
            timeout: MiningCampaignDuration::new(600_000).expect("duration"),
        },
    );
    let mut tracker = CampaignStatusTracker::new(
        MiningCampaignStage::CommandEffects,
        Some(lease),
        Some(profile),
    );
    let mut active = snapshot(MiningCampaignState::Active);
    active.hardware_state = MiningHardwareState::Ready;
    tracker.note_snapshot(&active, 100);
    let mut stopping = snapshot(MiningCampaignState::SafeStopping);
    stopping.hardware_state = MiningHardwareState::SafeStopping;
    stopping.mining.operator_intent = MiningOperatorIntent::Paused;
    let mut stopped = snapshot(MiningCampaignState::Armed);
    stopped.hardware_state = MiningHardwareState::Stopped;
    stopped.mining.operator_intent = MiningOperatorIntent::Paused;
    let mut resumed = stopped.clone();
    resumed.mining.operator_intent = MiningOperatorIntent::Run;

    // Act
    let markers = [&stopping, &stopped, &resumed].map(|snapshot| {
        tracker.marker(
            snapshot,
            1_000,
            true,
            CampaignObservationFreshness::all_ultra205_supported_fresh(),
            false,
            true,
            "ready",
            readiness_transition(),
        )
    });
    let states = markers.map(|marker| {
        let value: Value = serde_json::from_str(&marker).expect("marker should be JSON");
        value["resumable_pause_safe_stop"].clone()
    });

    // Assert
    assert_eq!(
        states,
        [
            serde_json::json!("pending"),
            serde_json::json!("confirmed"),
            serde_json::json!("not_required"),
        ]
    );
    assert!(tracker.authorizes_actuation());
}

#[test]
fn command_effects_marker_accumulates_only_active_segments() {
    // Arrange
    let profile = MiningHardwareProfilePreset::Conservative;
    let lease = MiningCampaignLease::new(
        MiningCampaignLeaseId::new(10).expect("lease id"),
        profile.profile(),
        MiningCampaignStopCondition::ResumableActiveEpoch {
            activation_timeout: MiningCampaignDuration::new(600_000).expect("timeout"),
            duration: MiningCampaignDuration::new(600_000).expect("duration"),
        },
    );
    let mut tracker = CampaignStatusTracker::new(
        MiningCampaignStage::CommandEffects,
        Some(lease),
        Some(profile),
    );
    let active = snapshot(MiningCampaignState::Active);
    let stopped = snapshot(MiningCampaignState::Armed);

    // Act
    tracker.note_snapshot(&active, 100);
    tracker.note_snapshot(&stopped, 200);
    tracker.note_snapshot(&active, 1_000);
    let marker = tracker.marker(
        &active,
        1_050,
        true,
        CampaignObservationFreshness::all_ultra205_supported_fresh(),
        false,
        true,
        "ready",
        readiness_transition(),
    );
    let value: Value = serde_json::from_str(&marker).expect("marker should be JSON");

    // Assert
    assert_eq!(value["active_ms"], 150);
}
