use serde_json::Value;

use super::super::substance::SubstantiveEvidenceError;
use super::*;

const SESSION: &str = "0123456789abcdef0011223344556677";
const OTHER_SESSION: &str = "fedcba9876543210ffeeddccbbaa9988";
const ELIGIBLE_SUBSTANCE: &str = include_str!("../../../fixtures/phase36/substance-eligible.json");

#[derive(Debug, Clone, Copy)]
enum ExpectedMutation {
    SensorDigestChanged,
    HealthDigestChanged,
    Rejected,
}

type Mutation = (&'static str, ExpectedMutation, fn(&mut Value));

fn projection() -> Value {
    serde_json::from_str(ELIGIBLE_SUBSTANCE).expect("substantive fixture must be valid JSON")
}

fn documents(value: &Value) -> (String, String, String) {
    let json = serde_json::to_string(value).expect("projection must serialize");
    let revision = value["operatorSnapshotRevision"]
        .as_u64()
        .expect("fixture revision must be numeric");
    let session = value["bootSession"]
        .as_str()
        .expect("fixture session must be textual");
    let marker = format!("operator_snapshot session={session} revision={revision} redacted=true");
    (
        format!(
            "system_info_json: {json}\noperator_snapshot_boot_session: {session}\noperator_snapshot_revision: {revision}\n"
        ),
        format!(
            "live_websocket_json: {json}\noperator_snapshot_boot_session: {session}\noperator_snapshot_revision: {revision}\n"
        ),
        format!("{marker}\nsubstantive_snapshot_json: {json}\n"),
    )
}

fn classify(value: &Value) -> Result<SubstantiveEvidenceAdmission, SubstantiveEvidenceError> {
    let (api, websocket, retained) = documents(value);
    validate_substantive_snapshot_documents(&api, &websocket, &retained)
}

fn validated(
    value: &Value,
) -> (
    ValidatedSensorSubstance,
    ValidatedRuntimeHealthSubstance,
    SubstantiveSnapshotJoin,
) {
    let SubstantiveEvidenceAdmission::Validated { evidence } =
        classify(value).expect("projection should validate")
    else {
        panic!("projection should be substantive");
    };
    (evidence.sensors, evidence.runtime_health, evidence.join)
}

#[test]
fn phase36_substance_admits_exact_three_surface_sensor_and_health_facts() {
    // Arrange
    let value = projection();

    // Act
    let (sensors, runtime_health, join) = validated(&value);

    // Assert
    assert!(matches!(
        sensors.power.state,
        ObservationState::Fresh { .. }
    ));
    assert_eq!(sensors.power.maybe_current_milliamps, Some(1_250));
    assert_eq!(sensors.power.maybe_bus_millivolts, Some(5_100));
    assert_eq!(sensors.power.maybe_power_milliwatts, Some(6_375));
    assert_eq!(runtime_health.maybe_checkpoint_sequence, Some(14));
    assert_eq!(join.operator_snapshot_revision, 15);
    assert_eq!(sensors.claim_fact_digest.len(), 64);
    assert_eq!(runtime_health.claim_fact_digest.len(), 64);
}

#[test]
fn phase36_substance_identity_only_phase35_documents_are_typed_insufficient() {
    // Arrange
    let api = format!(
        "system_info_json: {{\"bootSession\":\"{SESSION}\",\"operatorSnapshotRevision\":7}}\noperator_snapshot_boot_session: {SESSION}\noperator_snapshot_revision: 7\n"
    );
    let websocket = format!(
        "live_websocket_json: {{\"bootSession\":\"{SESSION}\",\"operatorSnapshotRevision\":8}}\noperator_snapshot_boot_session: {SESSION}\noperator_snapshot_revision: 8\n"
    );
    let retained = format!(
        "operator_snapshot session={SESSION} revision=7 redacted=true\noperator_snapshot session={SESSION} revision=8 redacted=true\n"
    );

    // Act
    let outcome = validate_substantive_snapshot_documents(&api, &websocket, &retained)
        .expect("identity-only shape should classify");

    // Assert
    assert_eq!(
        outcome,
        SubstantiveEvidenceAdmission::Insufficient {
            component_insufficiencies: vec![
                ComponentInsufficiency::SnapshotSubstance,
                ComponentInsufficiency::RuntimeHealth,
            ],
        }
    );
}

#[test]
fn phase36_substance_each_sensor_and_health_field_is_bound_or_rejected() {
    // Arrange
    let original = projection();
    let (original_sensors, original_health, _) = validated(&original);
    let mutations: [Mutation; 21] = [
        (
            "power current",
            ExpectedMutation::SensorDigestChanged,
            |value| {
                value["current"] = serde_json::json!(1.251);
            },
        ),
        (
            "power voltage",
            ExpectedMutation::SensorDigestChanged,
            |value| {
                value["voltage"] = serde_json::json!(5.101);
            },
        ),
        (
            "power wattage",
            ExpectedMutation::SensorDigestChanged,
            |value| {
                value["power"] = serde_json::json!(6.376);
            },
        ),
        ("power state", ExpectedMutation::Rejected, |value| {
            for field in ["currentStatus", "voltageStatus", "powerStatus"] {
                value[field]["state"] = serde_json::json!("stale");
            }
        }),
        (
            "power stamp",
            ExpectedMutation::SensorDigestChanged,
            |value| {
                for field in ["currentStatus", "voltageStatus", "powerStatus"] {
                    value[field]["stamp"]["sequence"] = serde_json::json!(21);
                }
            },
        ),
        ("power reason", ExpectedMutation::Rejected, |value| {
            for field in ["currentStatus", "voltageStatus", "powerStatus"] {
                value[field]["reason"] =
                    serde_json::json!({"kind":"stale","code":"power_sample_stale"});
            }
        }),
        (
            "temperature value",
            ExpectedMutation::SensorDigestChanged,
            |value| {
                value["temp"] = serde_json::json!(55.251);
            },
        ),
        ("temperature state", ExpectedMutation::Rejected, |value| {
            value["chipTempStatus"]["state"] = serde_json::json!("unavailable");
        }),
        (
            "temperature stamp",
            ExpectedMutation::SensorDigestChanged,
            |value| {
                value["chipTempStatus"]["stamp"]["sequence"] = serde_json::json!(22);
            },
        ),
        ("temperature reason", ExpectedMutation::Rejected, |value| {
            value["chipTempStatus"]["reason"] =
                serde_json::json!({"kind":"fault","code":"read_failed"});
        }),
        (
            "tachometer value",
            ExpectedMutation::SensorDigestChanged,
            |value| {
                value["fanrpm"] = serde_json::json!(4_801);
            },
        ),
        ("tachometer state", ExpectedMutation::Rejected, |value| {
            value["fanRpmStatus"]["state"] = serde_json::json!("fault");
        }),
        (
            "tachometer stamp",
            ExpectedMutation::SensorDigestChanged,
            |value| {
                value["fanRpmStatus"]["stamp"]["sequence"] = serde_json::json!(23);
            },
        ),
        ("tachometer reason", ExpectedMutation::Rejected, |value| {
            value["fanRpmStatus"]["reason"] =
                serde_json::json!({"kind":"unavailable","code":"tachometer_unavailable"});
        }),
        (
            "health lifecycle",
            ExpectedMutation::HealthDigestChanged,
            |value| {
                value["runtimeHealth"]["selfTestState"] = serde_json::json!("blocked");
            },
        ),
        ("health supervisor", ExpectedMutation::Rejected, |value| {
            value["runtimeHealth"]["supervisorAvailability"] = serde_json::json!("unavailable");
        }),
        (
            "checkpoint category",
            ExpectedMutation::HealthDigestChanged,
            |value| {
                value["runtimeHealth"]["checkpointCategory"] = serde_json::json!("service_loop");
            },
        ),
        (
            "checkpoint sequence",
            ExpectedMutation::HealthDigestChanged,
            |value| {
                value["runtimeHealth"]["checkpointSequence"] = serde_json::json!(15);
            },
        ),
        (
            "checkpoint age",
            ExpectedMutation::HealthDigestChanged,
            |value| {
                value["runtimeHealth"]["checkpointAgeMillis"] = serde_json::json!(251);
            },
        ),
        ("checkpoint health", ExpectedMutation::Rejected, |value| {
            value["runtimeHealth"]["checkpointHealth"] = serde_json::json!("stale");
        }),
        (
            "watchdog participation",
            ExpectedMutation::Rejected,
            |value| {
                value["runtimeHealth"]["taskWatchdogParticipation"] =
                    serde_json::json!("participating");
            },
        ),
    ];

    // Act and Assert
    for (name, expected, mutate) in mutations {
        let mut changed = original.clone();
        mutate(&mut changed);
        match expected {
            ExpectedMutation::SensorDigestChanged => {
                let (sensors, _, _) = validated(&changed);
                assert_ne!(
                    sensors.claim_fact_digest, original_sensors.claim_fact_digest,
                    "sensor mutation {name} was not digest-bound"
                );
            }
            ExpectedMutation::HealthDigestChanged => {
                let (_, health, _) = validated(&changed);
                assert_ne!(
                    health.claim_fact_digest, original_health.claim_fact_digest,
                    "health mutation {name} was not digest-bound"
                );
            }
            ExpectedMutation::Rejected => {
                assert!(classify(&changed).is_err(), "mutation {name} was accepted");
            }
        }
    }
}

#[test]
fn phase36_substance_rejects_mixed_revision_and_boot_session() {
    // Arrange
    let value = projection();
    let (api, _, retained) = documents(&value);
    let mut changed = value.clone();
    changed["bootSession"] = serde_json::json!(OTHER_SESSION);
    changed["operatorSnapshotRevision"] = serde_json::json!(16);
    let (_, websocket, _) = documents(&changed);
    let retained = retained.replace(
        "substantive_snapshot_json:",
        &format!(
            "operator_snapshot session={OTHER_SESSION} revision=16 redacted=true\nsubstantive_snapshot_json:"
        ),
    );

    // Act
    let result = validate_substantive_snapshot_documents(&api, &websocket, &retained);

    // Assert
    assert_eq!(
        result,
        Err(SubstantiveEvidenceError::OperatorSnapshotIdentityInvalid)
    );
}

#[test]
fn phase36_substance_rejects_reused_unrelated_sensor_stamp() {
    // Arrange
    let mut value = projection();
    value["chipTempStatus"]["stamp"] = value["powerStatus"]["stamp"].clone();

    // Act
    let result = classify(&value);

    // Assert
    assert_eq!(
        result,
        Err(SubstantiveEvidenceError::ReusedUnrelatedObservationStamp)
    );
}

#[test]
fn phase36_substance_rejects_mixed_producer_boot_sessions() {
    // Arrange
    let mut value = projection();
    value["chipTempStatus"]["stamp"]["bootSession"] = serde_json::json!(8);

    // Act
    let result = classify(&value);

    // Assert
    assert_eq!(
        result,
        Err(SubstantiveEvidenceError::MixedSnapshotProvenance)
    );
}

#[test]
fn phase36_substance_rejects_compatibility_zero_claimed_fresh_without_stamp() {
    // Arrange
    let mut value = projection();
    value["temp"] = serde_json::json!(0.0);
    value["chipTempStatus"]
        .as_object_mut()
        .expect("status should be an object")
        .remove("stamp");

    // Act
    let result = classify(&value);

    // Assert
    assert_eq!(
        result,
        Err(SubstantiveEvidenceError::ContradictorySensorState)
    );
}

#[test]
fn phase36_substance_rejects_compatibility_zero_claimed_fresh_with_stamp() {
    // Arrange
    let mut value = projection();
    value["fanrpm"] = serde_json::json!(0);

    // Act
    let result = classify(&value);

    // Assert
    assert_eq!(
        result,
        Err(SubstantiveEvidenceError::ContradictorySensorState)
    );
}

#[test]
fn phase36_substance_admits_typed_non_fresh_states_without_compatibility_values() {
    // Arrange
    let mut value = projection();
    for numeric in ["current", "voltage", "power", "temp", "fanrpm"] {
        value[numeric] = serde_json::json!(0);
    }
    for field in ["currentStatus", "voltageStatus", "powerStatus"] {
        value[field] = serde_json::json!({
            "state": "unavailable",
            "reason": {"kind": "unavailable", "code": "power_sample_unavailable"}
        });
    }
    value["chipTempStatus"]["state"] = serde_json::json!("fault");
    value["chipTempStatus"]["reason"] =
        serde_json::json!({"kind": "fault", "code": "thermal_reading_invalid"});
    value["fanRpmStatus"]["state"] = serde_json::json!("stale");
    value["fanRpmStatus"]["reason"] =
        serde_json::json!({"kind": "stale", "code": "tachometer_stale"});

    // Act
    let (sensors, _, join) = validated(&value);

    // Assert
    assert!(matches!(
        sensors.power.state,
        ObservationState::Unavailable { .. }
    ));
    assert!(matches!(
        sensors.temperature.state,
        ObservationState::Fault { .. }
    ));
    assert!(matches!(
        sensors.tachometer.state,
        ObservationState::Stale { .. }
    ));
    assert!(join.maybe_power_stamp.is_none());
    assert!(join.maybe_temperature_stamp.is_some());
    assert!(join.maybe_tachometer_stamp.is_some());
}

#[test]
fn phase36_substance_rejects_fault_with_manufactured_current_value() {
    // Arrange
    let mut value = projection();
    for field in ["currentStatus", "voltageStatus", "powerStatus"] {
        value[field]["state"] = serde_json::json!("fault");
        value[field]["reason"] = serde_json::json!({"kind":"fault","code":"ina260_read_failed"});
    }

    // Act
    let result = classify(&value);

    // Assert
    assert_eq!(
        result,
        Err(SubstantiveEvidenceError::ContradictorySensorState)
    );
}

#[test]
fn phase36_substance_rejects_invalid_checkpoint_chronology() {
    // Arrange
    let mut value = projection();
    value["runtimeHealth"]["checkpointHealth"] = serde_json::json!("healthy");
    value["runtimeHealth"]["checkpointAgeMillis"] = serde_json::json!(5_001);

    // Act
    let result = classify(&value);

    // Assert
    assert_eq!(
        result,
        Err(SubstantiveEvidenceError::CheckpointChronologyInvalid)
    );
}

#[test]
fn phase36_substance_rejects_watchdog_claim_derived_from_supervisor_visibility() {
    // Arrange
    let mut value = projection();
    value["runtimeHealth"]["taskWatchdogParticipation"] = serde_json::json!("participating");
    value["runtimeHealth"]["taskWatchdogReason"] = Value::Null;

    // Act
    let result = classify(&value);

    // Assert
    assert_eq!(
        result,
        Err(SubstantiveEvidenceError::WatchdogObservationNotIndependent)
    );
}
