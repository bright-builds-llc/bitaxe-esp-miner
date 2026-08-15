use super::*;
use crate::campaign::markers::{OperatorSensorOutcomeMarker, OperatorSensorStageMarker};

fn observation_marker_with_operator_sensor(operator_sensor: serde_json::Value) -> Vec<u8> {
    let bytes = observation_marker(CAMPAIGN_MARKER_SCHEMA);
    let line = std::str::from_utf8(&bytes).expect("fixture marker should be utf8");
    let payload = line
        .trim_end()
        .strip_prefix(CAMPAIGN_MARKER_PREFIX)
        .expect("fixture marker should have prefix");
    let mut marker: serde_json::Value =
        serde_json::from_str(payload).expect("fixture marker should be json");
    marker["operator_sensor"] = operator_sensor;
    format!("{CAMPAIGN_MARKER_PREFIX}{marker}\n").into_bytes()
}

#[test]
fn redacted_pressure_is_preserved_in_the_typed_marker() {
    // Arrange
    let bytes = observation_marker_with_operator_sensor(serde_json::json!({
        "available": true,
        "boot_session": 17,
        "revision": 3,
        "stage": "display",
        "outcome": "budget_exhausted",
        "duration_bucket": "under_500_ms",
    }));

    // Act
    let capture = analyze_campaign_serial_bytes(&bytes, observation_admission());

    // Assert
    assert_eq!(capture.outcome_detail, CampaignSerialOutcomeDetail::Clean);
    let terminal = capture
        .aggregate
        .terminal
        .expect("valid marker should be retained");
    assert!(terminal.operator_sensor.available);
    assert_eq!(terminal.operator_sensor.revision, 3);
    assert_eq!(
        terminal.operator_sensor.stage,
        OperatorSensorStageMarker::Display
    );
    assert_eq!(
        terminal.operator_sensor.outcome,
        OperatorSensorOutcomeMarker::BudgetExhausted
    );
}

#[test]
fn contradictory_availability_is_marker_invalid() {
    // Arrange
    let bytes = observation_marker_with_operator_sensor(serde_json::json!({
        "available": false,
        "boot_session": 17,
        "revision": 3,
        "stage": "display",
        "outcome": "budget_exhausted",
        "duration_bucket": "under_500_ms",
    }));

    // Act
    let capture = analyze_campaign_serial_bytes(&bytes, observation_admission());

    // Assert
    assert_eq!(
        capture.maybe_failure,
        Some(CampaignTerminalCategory::MarkerInvalid)
    );
}
