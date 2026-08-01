use super::*;

fn job_transition_admission() -> CampaignAdmission {
    CampaignAdmission {
        stage: MiningCampaignStage::JobTransition,
        maybe_profile: Some(MiningCampaignProfile::Conservative),
        duration_seconds: 1_800,
        maybe_lease_id: Some(7),
    }
}

fn active_job_transition_marker(active_ms: u64) -> Vec<u8> {
    let marker = observation_marker(CAMPAIGN_MARKER_SCHEMA);
    let payload = marker
        .strip_prefix(CAMPAIGN_MARKER_PREFIX.as_bytes())
        .expect("fixture marker prefix");
    let mut value: serde_json::Value =
        serde_json::from_slice(payload).expect("fixture marker JSON");
    value["stage"] = serde_json::json!("job-transition");
    value["lease_id"] = serde_json::json!(7);
    value["campaign_state"] = serde_json::json!("active");
    value["profile"] = serde_json::json!("conservative");
    value["active_ms"] = serde_json::json!(active_ms);
    value["pool_config"] = serde_json::json!("local_owner_supplied");
    value["actuation"] = serde_json::json!("qualified");
    value["safe_stop"] = serde_json::json!("pending");
    format!("{CAMPAIGN_MARKER_PREFIX}{value}\n").into_bytes()
}

fn job_transition_event_marker(sequence: u64) -> Vec<u8> {
    let marker = active_job_transition_marker(sequence.saturating_mul(100));
    let payload = marker
        .strip_prefix(CAMPAIGN_MARKER_PREFIX.as_bytes())
        .expect("fixture marker prefix");
    let mut value: serde_json::Value =
        serde_json::from_slice(payload).expect("fixture marker JSON");
    value["asic_bridge"]["latest_event"] = serde_json::json!({
        "sequence": sequence,
        "monotonic_offset_ms": sequence.saturating_mul(100),
        "kind": "poll_completed",
        "generation_relation": "replacement",
        "outcome": "idle",
    });
    format!("{CAMPAIGN_MARKER_PREFIX}{value}\n").into_bytes()
}

#[test]
fn asic_transition_trace_retains_bounded_first_and_last_events() {
    // Arrange
    let mut analyzer = CampaignSerialAnalyzer::new(job_transition_admission());

    // Act
    for sequence in 1..=70 {
        analyzer.observe_chunk(&job_transition_event_marker(sequence));
    }
    let capture = analyzer.finish();

    // Assert
    let trace = capture.aggregate.asic_event_trace;
    assert_eq!(trace.observed_event_count, 70);
    assert!(trace.events_truncated);
    assert_eq!(trace.first_events.len(), 32);
    assert_eq!(
        trace.first_events.first().map(|event| event.sequence),
        Some(1)
    );
    assert_eq!(
        trace.first_events.last().map(|event| event.sequence),
        Some(32)
    );
    assert_eq!(trace.last_events.len(), 32);
    assert_eq!(
        trace.last_events.front().map(|event| event.sequence),
        Some(39)
    );
    assert_eq!(
        trace.last_events.back().map(|event| event.sequence),
        Some(70)
    );
}

#[test]
fn job_transition_rejects_active_marker_gap_over_five_seconds() {
    // Arrange
    let admission = job_transition_admission();
    let mut analyzer = CampaignSerialAnalyzer::new(admission);

    // Act
    analyzer.observe_chunk(&active_job_transition_marker(1_000));
    analyzer.observe_chunk(&active_job_transition_marker(6_001));
    let capture = analyzer.finish();
    let result = capture.aggregate.assess(admission);

    // Assert
    assert_eq!(capture.aggregate.maximum_active_marker_gap_ms, 5_001);
    assert_eq!(
        result
            .expect_err("overlong active marker gap must fail")
            .category,
        CampaignTerminalCategory::MarkerContinuityFailed
    );
}
