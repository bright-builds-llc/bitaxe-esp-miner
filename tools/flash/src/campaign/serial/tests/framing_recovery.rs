use super::*;

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
fn later_valid_marker_recovers_transient_invalid_utf8_payload() {
    // Arrange
    let mut bytes = CAMPAIGN_MARKER_PREFIX.as_bytes().to_vec();
    bytes.extend_from_slice(&[0xff, b'\n']);
    bytes.extend_from_slice(&observation_marker(CAMPAIGN_MARKER_SCHEMA));

    // Act
    let capture = analyze_campaign_serial_bytes(&bytes, observation_admission());

    // Assert
    assert_eq!(capture.maybe_failure, None);
    assert_eq!(capture.outcome_detail, CampaignSerialOutcomeDetail::Clean);
    assert_eq!(capture.diagnostics.marker_invalid_encoding_count, 1);
    assert_eq!(capture.diagnostics.accepted_marker_count, 1);
}
