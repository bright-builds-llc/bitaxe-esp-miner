use super::*;

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
