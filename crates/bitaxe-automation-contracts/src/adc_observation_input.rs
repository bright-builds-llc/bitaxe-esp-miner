use serde::Deserialize;

const MIN_PLAUSIBLE_MILLIVOLTS: f64 = 400.0;
const MAX_PLAUSIBLE_MILLIVOLTS: f64 = 2_000.0;

/// Protected HTTP ADC input consumed only by the lossless validator.
#[derive(Debug, Deserialize)]
pub struct AdcObservationSnapshotInput {
    #[serde(rename = "coreVoltageActual")]
    core_voltage_actual: f64,
    #[serde(rename = "coreVoltageActualStatus")]
    core_voltage_actual_status: AdcObservationStatusInput,
}

/// Protected WebSocket ADC envelope consumed only by the lossless validator.
#[derive(Debug, Deserialize)]
pub struct AdcObservationWebSocketInput {
    event: String,
    data: AdcObservationSnapshotInput,
}

#[derive(Debug, Deserialize)]
struct AdcObservationStatusInput {
    state: String,
    stamp: AdcObservationStampInput,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct AdcObservationStampInput {
    #[serde(rename = "bootSession")]
    boot_session: u64,
    sequence: u64,
    #[serde(rename = "acquiredAtMs")]
    acquired_at_ms: u64,
}

/// Validates live ADC values and monotonic acquisition truth without emitting them.
pub fn validate_adc_observation_inputs(
    http: &AdcObservationSnapshotInput,
    websocket: &AdcObservationWebSocketInput,
) -> Result<(), &'static str> {
    if websocket.event != "update" {
        return Err("ADC observation WebSocket envelope is invalid");
    }
    let websocket = &websocket.data;
    if !admitted_millivolts(http.core_voltage_actual)
        || !admitted_millivolts(websocket.core_voltage_actual)
    {
        return Err("ADC observation values are outside the admitted range");
    }
    let http_status = &http.core_voltage_actual_status;
    let websocket_status = &websocket.core_voltage_actual_status;
    if http_status.state != "fresh"
        || websocket_status.state != "fresh"
        || http_status.stamp.boot_session != websocket_status.stamp.boot_session
        || http_status.stamp.sequence == 0
        || websocket_status.stamp.sequence < http_status.stamp.sequence
        || websocket_status.stamp.acquired_at_ms < http_status.stamp.acquired_at_ms
    {
        return Err("ADC observation states or stamps are incomplete");
    }
    if websocket_status.stamp.sequence == http_status.stamp.sequence
        && (websocket_status.stamp != http_status.stamp
            || websocket.core_voltage_actual.to_bits() != http.core_voltage_actual.to_bits())
    {
        return Err("ADC observation equal-sequence snapshots are incoherent");
    }
    Ok(())
}

fn admitted_millivolts(millivolts: f64) -> bool {
    millivolts.is_finite()
        && (MIN_PLAUSIBLE_MILLIVOLTS..=MAX_PLAUSIBLE_MILLIVOLTS).contains(&millivolts)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshots() -> (AdcObservationSnapshotInput, AdcObservationWebSocketInput) {
        let http = r#"{"coreVoltageActual":1198.0,"coreVoltageActualStatus":{"state":"fresh","stamp":{"bootSession":9,"sequence":11,"acquiredAtMs":500}}}"#;
        let websocket = r#"{"event":"update","data":{"coreVoltageActual":1201.0,"coreVoltageActualStatus":{"state":"fresh","stamp":{"bootSession":9,"sequence":12,"acquiredAtMs":1000}}}}"#;
        (
            serde_json::from_str(http).expect("HTTP input"),
            serde_json::from_str(websocket).expect("WebSocket input"),
        )
    }

    #[test]
    fn fresh_monotonic_live_samples_are_accepted() {
        // Arrange
        let (http, websocket) = snapshots();

        // Act
        let result = validate_adc_observation_inputs(&http, &websocket);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn regressed_sequence_is_rejected() {
        // Arrange
        let (http, mut websocket) = snapshots();
        websocket.data.core_voltage_actual_status.stamp.sequence = 10;

        // Act
        let result = validate_adc_observation_inputs(&http, &websocket);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn non_fresh_or_implausible_samples_are_rejected() {
        // Arrange
        let (mut stale, websocket) = snapshots();
        stale.core_voltage_actual_status.state = "stale".to_owned();
        let (http, mut implausible) = snapshots();
        implausible.data.core_voltage_actual = 0.0;

        // Act
        let stale_result = validate_adc_observation_inputs(&stale, &websocket);
        let implausible_result = validate_adc_observation_inputs(&http, &implausible);

        // Assert
        assert!(stale_result.is_err());
        assert!(implausible_result.is_err());
    }
}
