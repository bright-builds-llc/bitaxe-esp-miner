use serde::Deserialize;

const MIN_PLAUSIBLE_TEMPERATURE_CELSIUS: f64 = -40.0;
const ASIC_THROTTLE_TEMPERATURE_CELSIUS: f64 = 75.0;

/// Protected HTTP thermal input consumed only by the lossless validator.
#[derive(Debug, Deserialize)]
pub struct Emc2101ThermalSnapshotInput {
    temp: f64,
    #[serde(rename = "chipTempStatus")]
    chip_temp_status: Emc2101ThermalStatusInput,
}

/// Protected WebSocket thermal envelope consumed only by the lossless validator.
#[derive(Debug, Deserialize)]
pub struct Emc2101ThermalWebSocketInput {
    event: String,
    data: Emc2101ThermalSnapshotInput,
}

#[derive(Debug, Deserialize)]
struct Emc2101ThermalStatusInput {
    state: String,
    stamp: Emc2101ThermalStampInput,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct Emc2101ThermalStampInput {
    #[serde(rename = "bootSession")]
    boot_session: u64,
    sequence: u64,
    #[serde(rename = "acquiredAtMs")]
    acquired_at_ms: u64,
}

/// Validates exact thermal values and acquisition stamps without emitting them.
pub fn validate_emc2101_thermal_inputs(
    http: &Emc2101ThermalSnapshotInput,
    websocket: &Emc2101ThermalWebSocketInput,
) -> Result<(), &'static str> {
    if websocket.event != "update" {
        return Err("EMC2101 WebSocket envelope is invalid");
    }
    let websocket = &websocket.data;
    if !admitted_temperature(http.temp)
        || !admitted_temperature(websocket.temp)
        || http.temp.to_bits() != websocket.temp.to_bits()
    {
        return Err("EMC2101 thermal values are unsafe or unequal");
    }
    if http.chip_temp_status.state != "fresh"
        || websocket.chip_temp_status.state != "fresh"
        || http.chip_temp_status.stamp != websocket.chip_temp_status.stamp
    {
        return Err("EMC2101 thermal states or stamps are incomplete");
    }
    Ok(())
}

fn admitted_temperature(temperature: f64) -> bool {
    temperature.is_finite()
        && (MIN_PLAUSIBLE_TEMPERATURE_CELSIUS..ASIC_THROTTLE_TEMPERATURE_CELSIUS)
            .contains(&temperature)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn snapshots(boot_session: u64) -> (Emc2101ThermalSnapshotInput, Emc2101ThermalWebSocketInput) {
        let http = format!(
            r#"{{"temp":50.0,"chipTempStatus":{{"state":"fresh","stamp":{{"bootSession":{boot_session},"sequence":11,"acquiredAtMs":500}}}}}}"#,
        );
        let websocket = format!(r#"{{"event":"update","data":{http}}}"#);
        (
            serde_json::from_str(&http).expect("HTTP input"),
            serde_json::from_str(&websocket).expect("WebSocket input"),
        )
    }

    #[test]
    fn equal_maximum_width_stamps_are_accepted() {
        // Arrange
        let (http, websocket) = snapshots(u64::MAX);

        // Act
        let result = validate_emc2101_thermal_inputs(&http, &websocket);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn mismatched_wide_stamps_are_rejected() {
        // Arrange
        let (http, mut websocket) = snapshots(u64::MAX);
        websocket.data.chip_temp_status.stamp.boot_session -= 1;

        // Act
        let result = validate_emc2101_thermal_inputs(&http, &websocket);

        // Assert
        assert_eq!(
            result,
            Err("EMC2101 thermal states or stamps are incomplete")
        );
    }

    #[test]
    fn invalid_integer_encodings_are_rejected_during_deserialization() {
        for invalid in ["-1", "1.5", "18446744073709551616", "\"9\""] {
            // Arrange
            let document = format!(
                r#"{{"temp":50.0,"chipTempStatus":{{"state":"fresh","stamp":{{"bootSession":{invalid},"sequence":11,"acquiredAtMs":500}}}}}}"#,
            );

            // Act
            let result = serde_json::from_str::<Emc2101ThermalSnapshotInput>(&document);

            // Assert
            assert!(result.is_err());
        }
    }

    #[test]
    fn stale_input_is_rejected() {
        // Arrange
        let (mut stale, websocket) = snapshots(9);
        stale.chip_temp_status.state = "stale".to_owned();

        // Act
        let result = validate_emc2101_thermal_inputs(&stale, &websocket);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn threshold_temperature_is_rejected() {
        // Arrange
        let (http, mut websocket) = snapshots(9);
        websocket.data.temp = 75.0;

        // Act
        let result = validate_emc2101_thermal_inputs(&http, &websocket);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn wrong_websocket_envelope_is_rejected() {
        // Arrange
        let (http, mut websocket) = snapshots(9);
        websocket.event = "other".to_owned();

        // Act
        let result = validate_emc2101_thermal_inputs(&http, &websocket);

        // Assert
        assert!(result.is_err());
    }
}
