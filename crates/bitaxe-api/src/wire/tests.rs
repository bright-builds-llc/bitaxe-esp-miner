use bitaxe_safety::observation::{
    BootSessionId, FaultReason, MonotonicMillis, Observation, ObservationSequence, StaleReason,
    UnavailableReason,
};
use serde::Deserialize;
use serde_json::{json, Value};

use bitaxe_core::runtime_health::{
    CheckpointObservation, PassiveSelfTestState, RuntimeHealthSnapshot, TaskWatchdogObservation,
};

use super::{require_wire_keys, retained_runtime_health_record};
use crate::{
    ApiSnapshot, SafeTelemetrySnapshot, SystemAsicWire, SystemInfoBlockSnapshot,
    SystemInfoCoinbaseOutput, SystemInfoWire, TelemetryObservations,
};

const SYSTEM_INFO_FIELD_CONTRACT: &str =
    include_str!("../../fixtures/api/system-info-contract-v1.json");

#[derive(Deserialize)]
struct FieldContract {
    schema_version: String,
    fields: std::collections::BTreeMap<String, FieldRule>,
}

#[derive(Debug, Deserialize)]
struct FieldRule {
    #[serde(rename = "type")]
    value_type: String,
    presence: String,
}

fn field_contract() -> FieldContract {
    serde_json::from_str(SYSTEM_INFO_FIELD_CONTRACT).expect("field contract should parse")
}

fn json_type(value: &Value) -> &'static str {
    match value {
        Value::Null => "null",
        Value::Bool(_) => "boolean",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[test]
fn system_info_wire_serializes_upstream_field_names_and_encodings() {
    // Arrange
    let snapshot = ApiSnapshot::safe_ultra_205();
    let wire = SystemInfoWire::from_snapshot(&snapshot);

    // Act
    let value = serde_json::to_value(wire).expect("system info should serialize");

    // Assert
    assert!(value.get("ASICModel").is_some());
    assert!(value.get("hashRate_1m").is_some());
    assert_eq!(value["hashrateMonitor"]["asics"], json!([]));
    assert_eq!(value["errorPercentage"], json!(0.0));
    assert!(value.get("fanspeed").is_some());
    assert!(value.get("fanrpm").is_some());
    assert_eq!(value.get("miningPaused"), Some(&Value::Bool(false)));
    assert_eq!(value.get("startMiningOnBoot"), Some(&Value::Bool(true)));
    assert_eq!(value.get("miningActivity"), Some(&json!("safe_blocked")));
    assert_eq!(value.get("apEnabled"), Some(&json!(0)));
    assert_eq!(value.get("autofanspeed"), Some(&json!(1)));
    assert_eq!(value.get("showNewBlock"), Some(&Value::Bool(false)));
    assert_eq!(value.get("version"), Some(&json!("000000000000-dev")));
    assert_eq!(value.get("semanticVersion"), Some(&json!("0.0.0-safe")));
    assert_eq!(value.get("sourceCommit"), Some(&json!("0".repeat(40))));
    assert_eq!(value.get("referenceCommit"), Some(&json!("0".repeat(40))));
    assert_eq!(value.get("appElfSha256"), Some(&json!("0".repeat(64))));
    assert_eq!(value.get("buildTimestampUtc"), Some(&json!("Unavailable")));
    assert_eq!(value.get("buildChannel"), Some(&json!("dev")));
    assert_eq!(value.get("sourceDirty"), Some(&Value::Bool(false)));
    assert_eq!(value.get("releaseTag"), Some(&Value::Null));
    assert_eq!(value.get("bootSession"), Some(&json!("0".repeat(32))));
    assert_eq!(value.get("bootOrdinal"), Some(&json!(0)));
    assert_eq!(
        value.get("resetReasonCategory"),
        Some(&json!("unavailable"))
    );
    assert_eq!(value.get("operatorSnapshotRevision"), Some(&json!(1)));
    assert_eq!(value["runtimeHealth"]["selfTestState"], "unavailable");
    assert_eq!(
        value["runtimeHealth"]["taskWatchdogParticipation"],
        "unavailable"
    );
    assert_eq!(value["runtimeHealth"]["taskWatchdogReason"], "unproved");
    assert_eq!(
        value["platformIdentity"]["uptimeMilliseconds"]["state"],
        "unavailable"
    );
    assert_eq!(
        value["platformIdentity"]["uptimeMilliseconds"]["reason"],
        "fixture_only"
    );
    assert!(require_wire_keys(
        &value,
        &[
            "ASICModel",
            "hashRate_1m",
            "hashrateMonitor",
            "errorPercentage",
            "fanspeed",
            "fanrpm",
            "miningPaused",
            "apEnabled",
        ],
    )
    .is_ok());
}

#[test]
fn safe_system_info_contains_every_unconditional_openapi_field() {
    // Arrange
    let contract = field_contract();
    let value = serde_json::to_value(SystemInfoWire::from_snapshot(&ApiSnapshot::safe_ultra_205()))
        .expect("system info should serialize");

    // Act
    let mismatched = contract
        .fields
        .iter()
        .filter(|(_, rule)| rule.presence == "always")
        .filter(|(field, rule)| {
            value
                .get(field.as_str())
                .is_none_or(|candidate| json_type(candidate) != rule.value_type)
        })
        .collect::<Vec<_>>();

    // Assert
    assert_eq!(
        contract.schema_version,
        "bitaxe-system-info-field-contract-v1"
    );
    assert_eq!(contract.fields.len(), 94);
    assert!(
        mismatched.is_empty(),
        "invalid unconditional fields: {mismatched:?}"
    );
    for (field, _rule) in contract
        .fields
        .iter()
        .filter(|(_, rule)| rule.presence == "block_found")
    {
        assert!(value.get(field).is_none(), "inactive block emitted {field}");
    }
}

#[test]
fn positive_block_snapshot_contains_every_openapi_required_field() {
    // Arrange
    let contract = field_contract();
    let mut snapshot = ApiSnapshot::safe_ultra_205();
    snapshot.maybe_block = Some(Box::new(SystemInfoBlockSnapshot {
        height: 840_000,
        script_sig: "script-canary".to_owned(),
        network_difficulty: 83_000_000_000_000.0,
        coinbase_value_total_satoshis: 312_500_000,
        coinbase_value_user_satoshis: 300_000_000,
        signals: vec!["signal-canary".to_owned()],
        coinbase_outputs: vec![SystemInfoCoinbaseOutput {
            value_satoshis: 300_000_000,
            address: "address-canary".to_owned(),
        }],
    }));

    // Act
    let value = serde_json::to_value(SystemInfoWire::from_snapshot(&snapshot))
        .expect("system info should serialize");
    let mismatched = contract
        .fields
        .iter()
        .filter(|(field, rule)| {
            value
                .get(field.as_str())
                .is_none_or(|candidate| json_type(candidate) != rule.value_type)
        })
        .collect::<Vec<_>>();

    // Assert
    assert!(
        mismatched.is_empty(),
        "invalid required fields: {mismatched:?}"
    );
    assert_eq!(value["coinbaseOutputs"][0]["value"], 300_000_000);
    assert_eq!(value["coinbaseOutputs"][0]["address"], "address-canary");
}

#[test]
fn system_info_debug_never_exposes_response_or_block_identity_values() {
    // Arrange
    let mut snapshot = ApiSnapshot::safe_ultra_205();
    snapshot.system_info_settings.primary_pool.url = "pool-debug-canary".to_owned();
    snapshot.system_info_settings.primary_pool.user = "worker-debug-canary".to_owned();
    snapshot.platform.hostname = "hostname-debug-canary".to_owned();
    snapshot.maybe_block = Some(Box::new(SystemInfoBlockSnapshot {
        height: 840_000,
        script_sig: "script-debug-canary".to_owned(),
        network_difficulty: 1.0,
        coinbase_value_total_satoshis: 2,
        coinbase_value_user_satoshis: 1,
        signals: vec!["signal-debug-canary".to_owned()],
        coinbase_outputs: vec![SystemInfoCoinbaseOutput {
            value_satoshis: 1,
            address: "address-debug-canary".to_owned(),
        }],
    }));

    // Act
    let snapshot_debug = format!("{snapshot:?}");
    let wire_debug = format!("{:?}", SystemInfoWire::from_snapshot(&snapshot));

    // Assert
    for canary in [
        "pool-debug-canary",
        "worker-debug-canary",
        "script-debug-canary",
        "signal-debug-canary",
        "address-debug-canary",
    ] {
        assert!(!snapshot_debug.contains(canary));
        assert!(!wire_debug.contains(canary));
    }
    assert!(!wire_debug.contains("hostname-debug-canary"));
}

#[test]
fn runtime_health_serializes_exact_passive_values() {
    // Arrange
    let latest =
        CheckpointObservation::new("telemetry", 9, 1_000).expect("checkpoint should be valid");
    let mut snapshot = ApiSnapshot::safe_ultra_205();
    snapshot.runtime_health = RuntimeHealthSnapshot::evaluate(
        PassiveSelfTestState::Idle,
        None,
        Some(&latest),
        None,
        Some(TaskWatchdogObservation::fed(11, 1_050)),
        1_100,
        500,
    );

    // Act
    let value = serde_json::to_value(SystemInfoWire::from_snapshot(&snapshot))
        .expect("system info should serialize");

    // Assert
    assert_eq!(value["runtimeHealth"]["selfTestState"], "idle");
    assert_eq!(
        value["runtimeHealth"]["supervisorAvailability"],
        "available"
    );
    assert_eq!(value["runtimeHealth"]["checkpointCategory"], "telemetry");
    assert_eq!(value["runtimeHealth"]["checkpointSequence"], 9);
    assert_eq!(value["runtimeHealth"]["checkpointAgeMillis"], 100);
    assert_eq!(value["runtimeHealth"]["checkpointHealth"], "healthy");
    assert_eq!(
        value["runtimeHealth"]["taskWatchdogParticipation"],
        "participating"
    );
    assert_eq!(value["runtimeHealth"]["taskWatchdogReason"], "feed_fresh");
    assert_eq!(value["runtimeHealth"]["taskWatchdogFeedSequence"], 11);
    assert_eq!(value["runtimeHealth"]["taskWatchdogFeedAgeMillis"], 50);
}

#[test]
fn retained_runtime_health_record_is_correlated_and_redacted() {
    // Arrange
    let latest =
        CheckpointObservation::new("telemetry", 9, 1_000).expect("checkpoint should be valid");
    let snapshot = RuntimeHealthSnapshot::evaluate(
        PassiveSelfTestState::Idle,
        None,
        Some(&latest),
        None,
        Some(TaskWatchdogObservation::fed(11, 1_050)),
        1_100,
        500,
    );
    let identity = ApiSnapshot::safe_ultra_205().operator_snapshot_identity;

    // Act
    let record =
        retained_runtime_health_record(identity.boot_session(), identity.revision(), &snapshot);

    // Assert
    assert_eq!(
        record,
        "runtime_health boot_session=00000000000000000000000000000000 operator_snapshot_revision=1 self_test=idle supervisor=available checkpoint_category=telemetry checkpoint_sequence=9 checkpoint_age_millis=100 checkpoint_health=healthy task_watchdog_participation=participating task_watchdog_reason=feed_fresh task_watchdog_feed_sequence=11 task_watchdog_feed_age_millis=50 redacted=true"
    );
    for prohibited in [
        "credential",
        "ssid",
        "ipv4",
        "mac_addr",
        "device_id",
        "target",
        "secret",
    ] {
        assert!(!record.contains(prohibited));
    }
}

#[test]
fn safety_telemetry_system_info_exposes_exact_seven_truth_fields() {
    // Arrange
    let snapshot = ApiSnapshot::safe_ultra_205();

    // Act
    let value = serde_json::to_value(SystemInfoWire::from_snapshot(&snapshot))
        .expect("system info should serialize");
    let status_fields = [
        "chipTempStatus",
        "coreVoltageActualStatus",
        "currentStatus",
        "fanRpmStatus",
        "powerStatus",
        "voltageStatus",
        "vrTempStatus",
    ];

    // Assert
    for field in status_fields {
        assert_eq!(value[field]["state"], "unavailable");
    }
    for unsupported_field in [
        "fanSpeedStatus",
        "fan2RpmStatus",
        "chipTemp2Status",
        "coreVoltageStatus",
    ] {
        assert!(value.get(unsupported_field).is_none());
    }
}

#[test]
fn system_info_wire_uses_block_found_notification_snapshot() {
    // Arrange
    let mut snapshot = ApiSnapshot::safe_ultra_205();
    snapshot.block_found.block_found = 840_000;
    snapshot.block_found.show_new_block = true;

    // Act
    let wire = SystemInfoWire::from_snapshot(&snapshot);

    // Assert
    assert_eq!(wire.block_found, 840_000);
    assert!(wire.show_new_block);
}

#[test]
fn system_info_wire_uses_runtime_config_snapshot() {
    // Arrange
    let mut snapshot = ApiSnapshot::safe_ultra_205();
    snapshot.config.asic_frequency_mhz = 500.0;
    snapshot.config.asic_voltage_mv = 1_250;
    snapshot.config.auto_fan_speed = false;

    // Act
    let wire = SystemInfoWire::from_snapshot(&snapshot);

    // Assert
    assert_eq!(wire.frequency, 500.0);
    assert_eq!(wire.core_voltage, 1_250);
    assert_eq!(wire.auto_fan_speed, 0);
}

#[test]
fn safety_telemetry_projection_system_info_reads_safe_telemetry_values() {
    // Arrange
    let mut snapshot = ApiSnapshot::safe_ultra_205();
    snapshot.safe_telemetry =
        SafeTelemetrySnapshot::from_observations(&fresh_telemetry_observations());

    // Act
    let wire = SystemInfoWire::from_snapshot(&snapshot);

    // Assert
    assert_eq!(wire.power, 11.5);
    assert_eq!(wire.voltage_millivolts, 5_100.0);
    assert_eq!(wire.current_milliamps, 2_250.0);
    assert_eq!(wire.fan_rpm, 3_200);
    assert_eq!(wire.temp, 56.0);
    assert_eq!(wire.vr_temp, 45.0);
    assert_eq!(wire.power_status.state, crate::ObservationStateWire::Fresh);
    assert!(wire.power_status.stamp.is_some());
}

#[test]
fn system_info_serializes_legacy_electrical_milli_units() {
    // Arrange
    let mut snapshot = ApiSnapshot::safe_ultra_205();
    snapshot.safe_telemetry =
        SafeTelemetrySnapshot::from_observations(&fresh_telemetry_observations());

    // Act
    let value = serde_json::to_value(SystemInfoWire::from_snapshot(&snapshot))
        .expect("system info should serialize");

    // Assert
    assert_eq!(snapshot.safe_telemetry.voltage_volts, 5.1);
    assert_eq!(snapshot.safe_telemetry.current_amps, 2.25);
    assert_eq!(value["voltage"], 5_100.0);
    assert_eq!(value["current"], 2_250.0);
    assert_eq!(value["coreVoltageActual"], 1_198.0);
    assert_eq!(value["power"], 11.5);
    assert_eq!(value["nominalVoltage"], 5);
}

#[test]
fn system_info_wire_rejects_nonfresh_truth_numeric_claims_even_with_fresh_aggregate() {
    // Arrange
    let mut snapshot = ApiSnapshot::safe_ultra_205();
    snapshot.safe_telemetry =
        SafeTelemetrySnapshot::from_observations(&fresh_telemetry_observations());
    let stale_voltage = fresh_f64(5.1, 2)
        .mark_stale(StaleReason::PowerSampleStale)
        .expect("fresh voltage can become stale");
    let unavailable_power =
        Observation::<f64>::unavailable(UnavailableReason::PowerSampleUnavailable);
    let fault_current = fresh_f64(2.25, 3).record_fault(FaultReason::ReadFailed);
    snapshot.safe_telemetry.power_status = (&unavailable_power).into();
    snapshot.safe_telemetry.voltage_status = (&stale_voltage).into();
    snapshot.safe_telemetry.current_status = (&fault_current).into();
    assert_eq!(
        snapshot.safe_telemetry.status,
        crate::SafetyTelemetryStatus::Fresh
    );

    // Act
    let value = serde_json::to_value(SystemInfoWire::from_snapshot(&snapshot))
        .expect("system info should serialize");

    // Assert
    assert_eq!(value["power"], 0.0);
    assert_eq!(value["voltage"], 0.0);
    assert_eq!(value["current"], 0.0);
    assert_eq!(value["powerStatus"]["state"], "unavailable");
    assert_eq!(value["voltageStatus"]["state"], "stale");
    assert_eq!(value["currentStatus"]["state"], "fault");
    assert_eq!(value["temp"], 56.0);
    assert_eq!(value["chipTempStatus"]["state"], "fresh");
}

#[test]
fn system_asic_wire_serializes_upstream_asic_contract_names() {
    // Arrange
    let snapshot = ApiSnapshot::safe_ultra_205();
    let wire = SystemAsicWire::from_snapshot(&snapshot);

    // Act
    let value = serde_json::to_value(wire).expect("system asic should serialize");

    // Assert
    assert_eq!(value.get("ASICModel"), Some(&json!("BM1366")));
    assert_eq!(value.get("deviceModel"), Some(&json!("Ultra")));
    assert_eq!(value.get("swarmColor"), Some(&json!("purple")));
    assert_eq!(value.get("asicCount"), Some(&json!(1)));
}

fn fresh_telemetry_observations() -> TelemetryObservations {
    TelemetryObservations {
        power_watts: fresh_f64(11.5, 1),
        bus_voltage_volts: fresh_f64(5.1, 2),
        current_amps: fresh_f64(2.25, 3),
        core_voltage_actual_mv: fresh_f64(1_198.0, 7),
        chip_temp_celsius: fresh_f64(56.0, 4),
        vr_temp_celsius: fresh_f64(45.0, 5),
        fan_rpm: fresh_u16(3_200, 6),
    }
}

fn fresh_f64(value: f64, prior_sequence: u64) -> Observation<f64> {
    Observation::record_success(
        value,
        BootSessionId::new(7),
        ObservationSequence::new(prior_sequence),
        MonotonicMillis::new(250),
    )
    .expect("fixture sequence should advance")
    .0
}

fn fresh_u16(value: u16, prior_sequence: u64) -> Observation<u16> {
    Observation::record_success(
        value,
        BootSessionId::new(7),
        ObservationSequence::new(prior_sequence),
        MonotonicMillis::new(250),
    )
    .expect("fixture sequence should advance")
    .0
}

#[test]
fn wire_system_info_fixture_from_reference_safe_ultra_205_defaults_round_trips() {
    // Arrange
    let fixture = include_str!("../../fixtures/api/system-info-ultra205-safe.json");
    let original: Value =
        serde_json::from_str(fixture).expect("system info fixture should be valid JSON");

    // Act
    let parsed: SystemInfoWire =
        serde_json::from_str(fixture).expect("system info fixture should parse");
    let round_trip = serde_json::to_value(parsed).expect("system info fixture should serialize");

    // Assert
    assert_eq!(round_trip, original);
}

#[test]
fn wire_system_info_fixture_preserves_mixed_numeric_and_boolean_encodings() {
    // Arrange
    let fixture = include_str!("../../fixtures/api/system-info-ultra205-safe.json");

    // Act
    let value: Value =
        serde_json::from_str(fixture).expect("system info fixture should be valid JSON");

    // Assert
    assert!(value["apEnabled"].is_number());
    assert!(value["autofanspeed"].is_number());
    assert_eq!(value["miningPaused"], Value::Bool(false));
    assert_eq!(value["startMiningOnBoot"], Value::Bool(true));
    assert_eq!(value["showNewBlock"], Value::Bool(false));
}

#[test]
fn wire_system_info_fixture_keeps_phase_6_hardware_telemetry_safe() {
    // Arrange
    let fixture = include_str!("../../fixtures/api/system-info-ultra205-safe.json");

    // Act
    let value: Value =
        serde_json::from_str(fixture).expect("system info fixture should be valid JSON");

    // Assert
    assert_eq!(value["power"], json!(0.0));
    assert_eq!(value["voltage"], json!(0.0));
    assert_eq!(value["current"], json!(0.0));
    assert_eq!(value["temp"], json!(0.0));
    assert_eq!(value["fanspeed"], json!(0));
    assert_eq!(value["fanrpm"], json!(0));
    assert_eq!(value["actualFrequency"], json!(0.0));
    assert_eq!(value["expectedHashrate"], json!(0.0));
}

#[test]
fn wire_system_asic_fixture_from_reference_safe_ultra_205_defaults_round_trips() {
    // Arrange
    let fixture = include_str!("../../fixtures/api/asic-settings-ultra205.json");
    let original: Value =
        serde_json::from_str(fixture).expect("ASIC settings fixture should be valid JSON");

    // Act
    let parsed: SystemAsicWire =
        serde_json::from_str(fixture).expect("ASIC settings fixture should parse");
    let round_trip = serde_json::to_value(parsed).expect("ASIC settings fixture should serialize");

    // Assert
    assert_eq!(round_trip, original);
}
