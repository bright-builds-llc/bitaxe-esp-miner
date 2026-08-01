use super::*;

#[path = "tests/asic_diagnostics.rs"]
mod asic_diagnostics;
#[path = "tests/attestation.rs"]
mod attestation;

fn observation_admission() -> CampaignAdmission {
    CampaignAdmission {
        stage: MiningCampaignStage::Observation,
        maybe_profile: None,
        duration_seconds: 360,
        maybe_lease_id: None,
    }
}

fn live_share_admission() -> CampaignAdmission {
    CampaignAdmission {
        stage: MiningCampaignStage::LiveShare,
        maybe_profile: Some(MiningCampaignProfile::Conservative),
        duration_seconds: 600,
        maybe_lease_id: Some(7),
    }
}

fn live_share_preparing_marker() -> Vec<u8> {
    let marker = serde_json::json!({
        "schema": CAMPAIGN_MARKER_SCHEMA,
        "stage": "live-share",
        "lease_id": 7,
        "campaign_state": "preparing",
        "profile": "conservative",
        "active_ms": 0,
        "submit_outcome": "none",
        "qualified_candidate_count": 0,
        "below_pool_target_count": 0,
        "duplicate_candidate_count": 0,
        "accepted_share_count": 0,
        "rejected_share_count": 0,
        "job_transition": {
            "pool_notify_count": 0,
            "clean_jobs_notify_count": 0,
            "previous_block_change_count": 0,
            "new_block_generation_count": 0,
            "replacement_dispatch_count": 0,
            "post_transition_correlated_result_count": 0,
            "completed_transition_count": 0,
            "stale_generation_result_discard_count": 0,
            "stale_generation_submit_count": 0,
            "reconnect_count": 0,
            "latest_state": "not_observed",
        },
        "asic_bridge": {
            "poll_request_count": 0,
            "idle_completion_count": 0,
            "nonce_completion_count": 0,
            "register_read_count": 0,
            "discards": {
                "invalid_length": 0,
                "invalid_preamble": 0,
                "invalid_crc": 0,
                "job_lookup": 0,
                "core": 0,
                "address_interval": 0,
                "register_response": 0,
                "parser_invariant": 0,
            },
            "generation_invalidation_count": 0,
            "stale_completion_count": 0,
            "post_transition_poll_request_count": 0,
            "post_transition_completion_count": 0,
            "post_transition_nonce_emission_count": 0,
            "post_transition_correlation_count": 0,
            "blocked_correlation_count": 0,
            "blocked_correlations": {
                "wrong_session": 0,
                "job_lookup": 0,
                "work_stale": 0,
                "target_mismatch": 0,
                "other": 0,
            },
            "changed_block_to_replacement_dispatch_ms": null,
            "changed_block_to_first_poll_ms": null,
            "changed_block_to_first_nonce_ms": null,
            "changed_block_to_first_correlation_ms": null,
            "final_poll_state": "idle",
            "latest_event": null,
        },
        "terminal_reason": "network_unavailable",
        "safety": "fresh",
        "fresh_observation_count": 5,
        "observation_freshness": {
            "power_watts": true,
            "bus_voltage_volts": true,
            "current_amps": true,
            "chip_temp_celsius": true,
            "vr_temp_celsius": false,
            "fan_rpm": true,
        },
        "observation_requirements": {
            "power_watts": true,
            "bus_voltage_volts": true,
            "current_amps": true,
            "chip_temp_celsius": true,
            "vr_temp_celsius": false,
            "fan_rpm": true,
        },
        "pool_config": "not_read",
        "actuation": "none",
        "mineonboot": false,
        "safe_stop": "not_required",
        "failure": {
            "phase": "none",
            "step": "none",
            "detail": "none",
            "rollback_step": "none",
            "rollback_detail": "none",
        },
    });
    format!("{CAMPAIGN_MARKER_PREFIX}{marker}\n").into_bytes()
}

fn observation_marker(schema: &str) -> Vec<u8> {
    let marker = serde_json::json!({
        "schema": schema,
        "stage": "observation",
        "lease_id": null,
        "campaign_state": "unavailable",
        "profile": "none",
        "active_ms": 0,
        "submit_outcome": "none",
        "qualified_candidate_count": 0,
        "below_pool_target_count": 0,
        "duplicate_candidate_count": 0,
        "accepted_share_count": 0,
        "rejected_share_count": 0,
        "job_transition": {
            "pool_notify_count": 0,
            "clean_jobs_notify_count": 0,
            "previous_block_change_count": 0,
            "new_block_generation_count": 0,
            "replacement_dispatch_count": 0,
            "post_transition_correlated_result_count": 0,
            "completed_transition_count": 0,
            "stale_generation_result_discard_count": 0,
            "stale_generation_submit_count": 0,
            "reconnect_count": 0,
            "latest_state": "not_observed",
        },
        "asic_bridge": {
            "poll_request_count": 0,
            "idle_completion_count": 0,
            "nonce_completion_count": 0,
            "register_read_count": 0,
            "discards": {
                "invalid_length": 0,
                "invalid_preamble": 0,
                "invalid_crc": 0,
                "job_lookup": 0,
                "core": 0,
                "address_interval": 0,
                "register_response": 0,
                "parser_invariant": 0,
            },
            "generation_invalidation_count": 0,
            "stale_completion_count": 0,
            "post_transition_poll_request_count": 0,
            "post_transition_completion_count": 0,
            "post_transition_nonce_emission_count": 0,
            "post_transition_correlation_count": 0,
            "blocked_correlation_count": 0,
            "blocked_correlations": {
                "wrong_session": 0,
                "job_lookup": 0,
                "work_stale": 0,
                "target_mismatch": 0,
                "other": 0,
            },
            "changed_block_to_replacement_dispatch_ms": null,
            "changed_block_to_first_poll_ms": null,
            "changed_block_to_first_nonce_ms": null,
            "changed_block_to_first_correlation_ms": null,
            "final_poll_state": "idle",
            "latest_event": null,
        },
        "terminal_reason": "none",
        "safety": "fresh",
        "fresh_observation_count": 5,
        "observation_freshness": {
            "power_watts": true,
            "bus_voltage_volts": true,
            "current_amps": true,
            "chip_temp_celsius": true,
            "vr_temp_celsius": false,
            "fan_rpm": true,
        },
        "observation_requirements": {
            "power_watts": true,
            "bus_voltage_volts": true,
            "current_amps": true,
            "chip_temp_celsius": true,
            "vr_temp_celsius": false,
            "fan_rpm": true,
        },
        "pool_config": "not_read",
        "actuation": "none",
        "mineonboot": false,
        "safe_stop": "not_required",
        "failure": {
            "phase": "none",
            "step": "none",
            "detail": "none",
            "rollback_step": "none",
            "rollback_detail": "none",
        },
    });
    format!("{CAMPAIGN_MARKER_PREFIX}{marker}\n").into_bytes()
}

fn preparation_progress_line(step: &str, outcome: &str) -> Vec<u8> {
    let progress = serde_json::json!({
        "schema": CAMPAIGN_PREPARATION_SCHEMA,
        "step": step,
        "outcome": outcome,
    });
    format!("{CAMPAIGN_PREPARATION_PREFIX}{progress}\n").into_bytes()
}

#[test]
fn growing_snapshots_preserve_split_prefix_and_json() {
    // Arrange
    let bytes = observation_marker(CAMPAIGN_MARKER_SCHEMA);
    let prefix_split = CAMPAIGN_MARKER_PREFIX.len().saturating_sub(3);
    let json_split = CAMPAIGN_MARKER_PREFIX.len().saturating_add(12);
    let mut analyzer = CampaignSerialAnalyzer::new(observation_admission());

    // Act
    let prefix_stop = analyzer.observe_snapshot(&bytes[..prefix_split]);
    let json_stop = analyzer.observe_snapshot(&bytes[..json_split]);
    let complete_stop = analyzer.observe_snapshot(&bytes);
    let capture = analyzer.finish();

    // Assert
    assert!(!prefix_stop);
    assert!(!json_stop);
    assert!(!complete_stop);
    assert_eq!(capture.markers.len(), 1);
    assert_eq!(capture.outcome_detail, CampaignSerialOutcomeDetail::Clean);
    assert_eq!(capture.diagnostics.marker_candidate_count, 1);
    assert_eq!(capture.diagnostics.accepted_marker_count, 1);
}

#[test]
fn production_chunks_preserve_split_prefix_and_json() {
    // Arrange
    let bytes = observation_marker(CAMPAIGN_MARKER_SCHEMA);
    let prefix_split = CAMPAIGN_MARKER_PREFIX.len().saturating_sub(3);
    let json_split = CAMPAIGN_MARKER_PREFIX.len().saturating_add(12);
    let mut analyzer = CampaignSerialAnalyzer::new(observation_admission());

    // Act
    let first_stop = analyzer.observe_chunk(&bytes[..prefix_split]);
    let second_stop = analyzer.observe_chunk(&bytes[prefix_split..json_split]);
    let final_stop = analyzer.observe_chunk(&bytes[json_split..]);
    let capture = analyzer.finish();

    // Assert
    assert!(!first_stop);
    assert!(!second_stop);
    assert!(!final_stop);
    assert_eq!(capture.aggregate.marker_count, 1);
}

#[test]
fn invalid_utf8_inside_marker_payload_is_typed() {
    // Arrange
    let mut bytes = CAMPAIGN_MARKER_PREFIX.as_bytes().to_vec();
    bytes.extend_from_slice(&[0xff, b'\n']);

    // Act
    let capture = analyze_campaign_serial_bytes(&bytes, observation_admission());

    // Assert
    assert_eq!(
        capture.outcome_detail,
        CampaignSerialOutcomeDetail::MarkerPayloadInvalidUtf8
    );
    assert_eq!(
        capture.maybe_failure,
        Some(CampaignTerminalCategory::MarkerInvalid)
    );
    assert_eq!(capture.diagnostics.marker_invalid_encoding_count, 1);
}

#[test]
fn malformed_marker_json_is_typed() {
    // Arrange
    let bytes = format!("{CAMPAIGN_MARKER_PREFIX}{{]\n").into_bytes();

    // Act
    let capture = analyze_campaign_serial_bytes(&bytes, observation_admission());

    // Assert
    assert_eq!(
        capture.outcome_detail,
        CampaignSerialOutcomeDetail::MarkerJsonInvalid
    );
    assert_eq!(capture.diagnostics.marker_invalid_json_count, 1);
}

#[test]
fn wrong_marker_schema_is_typed() {
    // Arrange
    let bytes = observation_marker("mining-campaign-status-v0");

    // Act
    let capture = analyze_campaign_serial_bytes(&bytes, observation_admission());

    // Assert
    assert_eq!(
        capture.outcome_detail,
        CampaignSerialOutcomeDetail::MarkerSchemaInvalid
    );
    assert_eq!(capture.diagnostics.marker_invalid_schema_count, 1);
}

#[test]
fn contradictory_freshness_count_is_typed_as_schema_invalid() {
    // Arrange
    let mut marker = observation_marker(CAMPAIGN_MARKER_SCHEMA);
    let count = b"\"fresh_observation_count\":5";
    let index = find_bytes(&marker, count).expect("fresh observation count");
    marker.splice(
        index..index + count.len(),
        b"\"fresh_observation_count\":4".iter().copied(),
    );

    // Act
    let capture = analyze_campaign_serial_bytes(&marker, observation_admission());

    // Assert
    assert_eq!(
        capture.outcome_detail,
        CampaignSerialOutcomeDetail::MarkerSchemaInvalid
    );
    assert_eq!(capture.diagnostics.marker_invalid_schema_count, 1);
    assert!(capture.markers.is_empty());
}

#[test]
fn contradictory_ultra205_observation_requirements_are_schema_invalid() {
    // Arrange
    let mut marker = observation_marker(CAMPAIGN_MARKER_SCHEMA);
    let requirements = b"\"observation_requirements\":{\"bus_voltage_volts\":true,\"chip_temp_celsius\":true,\"current_amps\":true,\"fan_rpm\":true,\"power_watts\":true,\"vr_temp_celsius\":false}";
    let index = find_bytes(&marker, requirements).expect("observation requirements");
    let contradiction = b"\"observation_requirements\":{\"bus_voltage_volts\":true,\"chip_temp_celsius\":true,\"current_amps\":true,\"fan_rpm\":true,\"power_watts\":true,\"vr_temp_celsius\":true}";
    marker.splice(
        index..index + requirements.len(),
        contradiction.iter().copied(),
    );

    // Act
    let capture = analyze_campaign_serial_bytes(&marker, observation_admission());

    // Assert
    assert_eq!(
        capture.outcome_detail,
        CampaignSerialOutcomeDetail::MarkerSchemaInvalid
    );
    assert_eq!(capture.diagnostics.marker_invalid_schema_count, 1);
    assert!(capture.markers.is_empty());
}

#[test]
fn trailing_partial_marker_is_typed() {
    // Arrange
    let bytes = format!("{CAMPAIGN_MARKER_PREFIX}{{\"schema\":").into_bytes();

    // Act
    let capture = analyze_campaign_serial_bytes(&bytes, observation_admission());

    // Assert
    assert_eq!(
        capture.outcome_detail,
        CampaignSerialOutcomeDetail::MarkerTruncated
    );
    assert_eq!(capture.diagnostics.trailing_partial_count, 1);
    assert_eq!(capture.diagnostics.marker_truncated_count, 1);
    assert_eq!(
        capture.maybe_failure,
        Some(CampaignTerminalCategory::MarkerInvalid)
    );
}

#[test]
fn no_candidate_is_distinct_from_invalid_candidate() {
    // Arrange
    let bytes = b"ordinary boot output\n";

    // Act
    let capture = analyze_campaign_serial_bytes(bytes, observation_admission());

    // Assert
    assert_eq!(
        capture.outcome_detail,
        CampaignSerialOutcomeDetail::MarkerMissing
    );
    assert_eq!(capture.diagnostics.marker_candidate_count, 0);
    assert_eq!(capture.maybe_failure, None);
}

#[test]
fn non_utf8_noise_around_markers_is_ignored_and_counted() {
    // Arrange
    let marker = observation_marker(CAMPAIGN_MARKER_SCHEMA);
    let mut bytes = vec![0xff, 0xfe];
    bytes.extend_from_slice(&marker);
    bytes.extend_from_slice(&[0xf8, b'\n']);
    bytes.extend_from_slice(&marker);
    bytes.extend_from_slice(&[0x80, 0x81, b'\n']);

    // Act
    let capture = analyze_campaign_serial_bytes(&bytes, observation_admission());

    // Assert
    assert_eq!(capture.markers.len(), 2);
    assert_eq!(capture.outcome_detail, CampaignSerialOutcomeDetail::Clean);
    assert_eq!(capture.diagnostics.non_utf8_line_count, 3);
    assert_eq!(capture.diagnostics.ignored_invalid_byte_count, 5);
    assert_eq!(capture.diagnostics.line_count, 4);
}

#[test]
fn typed_trace_retains_first_and_last_events() {
    // Arrange
    let bytes = [0xff, b'\n'].repeat(70);

    // Act
    let capture = analyze_campaign_serial_bytes(&bytes, observation_admission());
    let events = &capture.diagnostics.events;

    // Assert
    assert_eq!(capture.diagnostics.event_count, 70);
    assert!(capture.diagnostics.events_truncated);
    assert_eq!(events.len(), 64);
    assert_eq!(events.first().map(|event| event.sequence), Some(1));
    assert_eq!(events.get(31).map(|event| event.sequence), Some(32));
    assert_eq!(events.get(32).map(|event| event.sequence), Some(39));
    assert_eq!(events.last().map(|event| event.sequence), Some(70));
}

#[test]
fn earliest_contract_failure_precedes_later_marker_parse_failure() {
    // Arrange
    let mut wrong_stage = observation_marker(CAMPAIGN_MARKER_SCHEMA);
    let stage = b"\"stage\":\"observation\"";
    let index = find_bytes(&wrong_stage, stage).expect("stage field");
    wrong_stage.splice(
        index..index + stage.len(),
        b"\"stage\":\"live-share\"".iter().copied(),
    );
    let mut bytes = wrong_stage;
    bytes.extend_from_slice(format!("{CAMPAIGN_MARKER_PREFIX}{{]\n").as_bytes());

    // Act
    let capture = analyze_campaign_serial_bytes(&bytes, observation_admission());

    // Assert
    assert_eq!(
        capture.maybe_failure,
        Some(CampaignTerminalCategory::StageMismatch)
    );
    assert_eq!(
        capture.outcome_detail,
        CampaignSerialOutcomeDetail::MarkerJsonInvalid
    );
}

#[test]
fn observation_contract_failure_does_not_shorten_capture_window() {
    // Arrange
    let mut marker = observation_marker(CAMPAIGN_MARKER_SCHEMA);
    let paused = b"\"mineonboot\":false";
    let index = find_bytes(&marker, paused).expect("mineonboot field");
    marker.splice(
        index..index + paused.len(),
        b"\"mineonboot\":true".iter().copied(),
    );
    let mut analyzer = CampaignSerialAnalyzer::new(observation_admission());

    // Act
    let should_stop = analyzer.observe_snapshot(&marker);
    let capture = analyzer.finish();

    // Assert
    assert!(!should_stop);
    assert_eq!(
        capture.maybe_failure,
        Some(CampaignTerminalCategory::MineOnBootEnabled)
    );
}

#[test]
fn preparation_progress_preserves_the_latest_closed_boundary() {
    // Arrange
    let mut bytes = preparation_progress_line("reset_and_detect_exactly_one_chip", "started");
    bytes.extend_from_slice(&preparation_progress_line(
        "reset_and_detect_exactly_one_chip",
        "completed",
    ));
    bytes.extend_from_slice(&observation_marker(CAMPAIGN_MARKER_SCHEMA));

    // Act
    let capture = analyze_campaign_serial_bytes(&bytes, observation_admission());

    // Assert
    assert_eq!(capture.diagnostics.preparation_candidate_count, 2);
    assert_eq!(capture.diagnostics.accepted_preparation_event_count, 2);
    assert_eq!(
        capture.diagnostics.latest_preparation_event,
        Some(CampaignPreparationProgress {
            schema: CAMPAIGN_PREPARATION_SCHEMA.to_owned(),
            step: CampaignFailureStepMarker::ResetAndDetectExactlyOneChip,
            outcome: CampaignPreparationOutcome::Completed,
        })
    );
}

#[test]
fn incomplete_live_preparation_overrides_stale_preparation_marker_state() {
    // Arrange
    let mut bytes = live_share_preparing_marker();
    bytes.extend_from_slice(&preparation_progress_line(
        "set_fan_duty_to_100_percent",
        "started",
    ));

    // Act
    let capture = analyze_campaign_serial_bytes(&bytes, live_share_admission());

    // Assert
    assert_eq!(
        capture.maybe_failure,
        Some(CampaignTerminalCategory::HardwarePreparationFailed)
    );
    assert_eq!(capture.diagnostics.accepted_preparation_event_count, 1);
}

#[test]
fn completed_live_preparation_does_not_synthesize_a_failure() {
    // Arrange
    let mut bytes = live_share_preparing_marker();
    bytes.extend_from_slice(&preparation_progress_line(
        "retain_production_uart",
        "completed",
    ));

    // Act
    let capture = analyze_campaign_serial_bytes(&bytes, live_share_admission());

    // Assert
    assert_eq!(capture.maybe_failure, None);
}

#[test]
fn malformed_preparation_progress_fails_closed_without_replacing_marker_detail() {
    // Arrange
    let mut bytes = format!("{CAMPAIGN_PREPARATION_PREFIX}{{]\n").into_bytes();
    bytes.extend_from_slice(&observation_marker(CAMPAIGN_MARKER_SCHEMA));

    // Act
    let capture = analyze_campaign_serial_bytes(&bytes, observation_admission());

    // Assert
    assert_eq!(
        capture.maybe_failure,
        Some(CampaignTerminalCategory::ObservationFailed)
    );
    assert_eq!(capture.outcome_detail, CampaignSerialOutcomeDetail::Clean);
    assert_eq!(capture.diagnostics.preparation_invalid_json_count, 1);
    assert_eq!(capture.diagnostics.accepted_marker_count, 1);
}

#[test]
fn chunk_stream_larger_than_sixteen_mib_reaches_terminal_marker_with_bounded_artifacts() {
    // Arrange
    let mut analyzer = CampaignSerialAnalyzer::new(observation_admission());
    let chunk = vec![b'x'; 64 * 1_024];

    // Act
    for _ in 0..257 {
        assert!(!analyzer.observe_chunk(&chunk));
        assert!(!analyzer.observe_chunk(b"\n"));
    }
    analyzer.observe_chunk(&observation_marker(CAMPAIGN_MARKER_SCHEMA));
    let capture = analyzer.finish();
    let aggregate_bytes = serde_json::to_vec(&capture.aggregate).expect("aggregate JSON");
    let diagnostic_bytes = serde_json::to_vec(&capture.diagnostics).expect("diagnostic JSON");

    // Assert
    assert!(capture.diagnostics.total_bytes > 16 * 1_024 * 1_024);
    assert_eq!(capture.aggregate.marker_count, 1);
    assert!(capture.aggregate.terminal.is_some());
    assert!(aggregate_bytes.len() < 64 * 1_024);
    assert!(diagnostic_bytes.len() < 64 * 1_024);
}
