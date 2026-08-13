use super::*;

#[test]
fn missing_resumable_pause_safe_stop_state_is_schema_invalid() {
    // Arrange
    let mut marker = observation_marker(CAMPAIGN_MARKER_SCHEMA);
    let field = b"\"resumable_pause_safe_stop\":\"not_required\",";
    let index = find_bytes(&marker, field).expect("resumable pause safe-stop field");
    marker.drain(index..index + field.len());

    // Act
    let capture = analyze_campaign_serial_bytes(&marker, observation_admission());

    // Assert
    assert_eq!(
        capture.outcome_detail,
        CampaignSerialOutcomeDetail::MarkerJsonInvalid
    );
    assert!(capture.markers.is_empty());
}

#[test]
fn unknown_resumable_pause_safe_stop_state_is_schema_invalid() {
    // Arrange
    let mut marker = observation_marker(CAMPAIGN_MARKER_SCHEMA);
    let state = b"\"resumable_pause_safe_stop\":\"not_required\"";
    let index = find_bytes(&marker, state).expect("resumable pause safe-stop state");
    marker.splice(
        index..index + state.len(),
        b"\"resumable_pause_safe_stop\":\"unknown\"".iter().copied(),
    );

    // Act
    let capture = analyze_campaign_serial_bytes(&marker, observation_admission());

    // Assert
    assert_eq!(
        capture.outcome_detail,
        CampaignSerialOutcomeDetail::MarkerJsonInvalid
    );
    assert!(capture.markers.is_empty());
}
