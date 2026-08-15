use serde::Deserialize;

use crate::SystemInfoEvidence;

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

/// Validates disabled-state ADC values and monotonic acquisition truth without emitting them.
pub fn validate_adc_observation_inputs(
    http: &AdcObservationSnapshotInput,
    websocket: &AdcObservationWebSocketInput,
    source: &SystemInfoEvidence,
) -> Result<(), &'static str> {
    source
        .validate()
        .map_err(|_| "ADC observation source safety state is invalid")?;
    if websocket.event != "update" {
        return Err("ADC observation WebSocket envelope is invalid");
    }
    let websocket = &websocket.data;
    if !is_millivolt_wire_value(http.core_voltage_actual)
        || !is_millivolt_wire_value(websocket.core_voltage_actual)
    {
        return Err("ADC observation values are outside the millivolt wire domain");
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

fn is_millivolt_wire_value(millivolts: f64) -> bool {
    // The calibrated firmware adapter returns `u16` millivolts before API projection.
    millivolts.is_finite()
        && millivolts >= 0.0
        && millivolts <= f64::from(u16::MAX)
        && millivolts.fract() == 0.0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AutomationCommand, SystemInfoObservationEvidence, WorkflowIdentity,
        SYSTEM_INFO_EVIDENCE_SCHEMA,
    };

    fn snapshots() -> (AdcObservationSnapshotInput, AdcObservationWebSocketInput) {
        let http = r#"{"coreVoltageActual":1198.0,"coreVoltageActualStatus":{"state":"fresh","stamp":{"bootSession":9,"sequence":11,"acquiredAtMs":500}}}"#;
        let websocket = r#"{"event":"update","data":{"coreVoltageActual":1201.0,"coreVoltageActualStatus":{"state":"fresh","stamp":{"bootSession":9,"sequence":12,"acquiredAtMs":1000}}}}"#;
        (
            serde_json::from_str(http).expect("HTTP input"),
            serde_json::from_str(websocket).expect("WebSocket input"),
        )
    }

    fn disabled_source() -> SystemInfoEvidence {
        SystemInfoEvidence {
            schema_version: SYSTEM_INFO_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            source_commit: "a".repeat(40),
            reference_commit: "b".repeat(40),
            package_manifest_sha256: "c".repeat(64),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::CaptureSystemInfoEvidence,
                request_sha256: "d".repeat(64),
            },
            detector_admitted: true,
            boot_observed: true,
            same_origin_observed: true,
            system_info: SystemInfoObservationEvidence {
                boot_session_sha256: "e".repeat(64),
                http_revision: 7,
                websocket_revision: 8,
                same_boot_session: true,
                websocket_revision_not_earlier: true,
                field_contract_schema: "bitaxe-system-info-field-contract-v1".to_owned(),
                field_contract_sha256: "f".repeat(64),
                required_field_count: 94,
                unconditional_field_count: 87,
                conditional_field_count: 7,
                http_unconditional_fields_complete: true,
                websocket_unconditional_fields_complete: true,
                http_field_types_match: true,
                websocket_field_types_match: true,
                inactive_block_fields_absent: true,
                confirmed_setting_fields_present: true,
                retained_http_tuple_matches: true,
                retained_websocket_tuple_matches: true,
            },
            mining_state: "disabled".to_owned(),
            hardware_control_state: "disabled".to_owned(),
            cleanup_complete: true,
            redaction_status: "passed".to_owned(),
        }
    }

    #[test]
    fn fresh_monotonic_live_samples_are_accepted() {
        // Arrange
        let (http, websocket) = snapshots();
        let source = disabled_source();

        // Act
        let result = validate_adc_observation_inputs(&http, &websocket, &source);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn regressed_sequence_is_rejected() {
        // Arrange
        let (http, mut websocket) = snapshots();
        websocket.data.core_voltage_actual_status.stamp.sequence = 10;
        let source = disabled_source();

        // Act
        let result = validate_adc_observation_inputs(&http, &websocket, &source);

        // Assert
        assert!(result.is_err());
    }

    #[test]
    fn fresh_zero_millivolt_samples_are_accepted() {
        // Arrange
        let (mut http, mut websocket) = snapshots();
        http.core_voltage_actual = 0.0;
        websocket.data.core_voltage_actual = 0.0;
        let source = disabled_source();

        // Act
        let result = validate_adc_observation_inputs(&http, &websocket, &source);

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn non_fresh_sample_is_rejected() {
        // Arrange
        let (mut stale, websocket) = snapshots();
        stale.core_voltage_actual_status.state = "stale".to_owned();
        let source = disabled_source();

        // Act
        let stale_result = validate_adc_observation_inputs(&stale, &websocket, &source);

        // Assert
        assert!(stale_result.is_err());
    }

    #[test]
    fn negative_millivolt_sample_is_rejected() {
        // Arrange
        let (http, mut negative) = snapshots();
        negative.data.core_voltage_actual = -1.0;
        let source = disabled_source();

        // Act
        let negative_result = validate_adc_observation_inputs(&http, &negative, &source);

        // Assert
        assert!(negative_result.is_err());
    }

    #[test]
    fn fractional_millivolt_sample_is_rejected() {
        // Arrange
        let (http, mut websocket) = snapshots();
        websocket.data.core_voltage_actual = 1_200.5;
        let source = disabled_source();

        // Act
        let result = validate_adc_observation_inputs(&http, &websocket, &source);

        // Assert
        assert_eq!(
            result,
            Err("ADC observation values are outside the millivolt wire domain")
        );
    }

    #[test]
    fn sample_above_u16_millivolt_domain_is_rejected() {
        // Arrange
        let (http, mut websocket) = snapshots();
        websocket.data.core_voltage_actual = f64::from(u16::MAX) + 1.0;
        let source = disabled_source();

        // Act
        let result = validate_adc_observation_inputs(&http, &websocket, &source);

        // Assert
        assert_eq!(
            result,
            Err("ADC observation values are outside the millivolt wire domain")
        );
    }

    #[test]
    fn observation_without_validated_disabled_state_is_rejected() {
        // Arrange
        let (http, websocket) = snapshots();
        let mut source = disabled_source();
        source.hardware_control_state = "enabled".to_owned();

        // Act
        let result = validate_adc_observation_inputs(&http, &websocket, &source);

        // Assert
        assert_eq!(
            result,
            Err("ADC observation source safety state is invalid")
        );
    }
}
