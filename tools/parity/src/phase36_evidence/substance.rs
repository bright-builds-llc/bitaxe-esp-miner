//! Substantive sensor and runtime-health admission for one coherent snapshot.

use serde::Serialize;

use super::contract::ComponentInsufficiency;
use crate::operator_snapshot_evidence::validate_operator_snapshot_documents;
use crate::phase35_evidence::sha256_hex;

mod types;

pub use types::{
    CheckpointCategory, CheckpointHealth, FaultReason, ObservationStamp, ObservationState,
    RuntimeLifecycleState, StaleReason, SubstantiveEvidenceAdmission, SubstantiveEvidenceError,
    SubstantiveSnapshotJoin, SupervisorAvailability, UnavailableReason,
    ValidatedRuntimeHealthSubstance, ValidatedScalarObservation, ValidatedSensorSubstance,
    ValidatedSubstantiveComponents, ValidatedSubstantiveEvidence, WatchdogAvailability,
};
use types::{
    RawObservationReason, RawObservationState, RawObservationTruth, RawProjection, RawReasonKind,
    RawRuntimeHealth, ValidatedPowerObservation, ValidatedProjection,
};

const SYSTEM_INFO_JSON_FIELD: &str = "system_info_json";
const LIVE_WEBSOCKET_JSON_FIELD: &str = "live_websocket_json";
const RETAINED_SUBSTANCE_JSON_FIELD: &str = "substantive_snapshot_json";
const CHECKPOINT_CATEGORY_MAX_ASCII_BYTES: usize = 32;
const HEALTHY_CHECKPOINT_MAX_AGE_MILLIS: u64 = 1_500;
const STALE_CHECKPOINT_MAX_AGE_MILLIS: u64 = 5_000;

pub fn validate_substantive_snapshot_documents(
    api_document: &str,
    websocket_document: &str,
    retained_document: &str,
) -> Result<SubstantiveEvidenceAdmission, SubstantiveEvidenceError> {
    if !validate_operator_snapshot_documents(api_document, websocket_document, retained_document)
        .is_empty()
    {
        return Err(SubstantiveEvidenceError::OperatorSnapshotIdentityInvalid);
    }

    let api_json = extract_single_field(api_document, SYSTEM_INFO_JSON_FIELD)
        .ok_or(SubstantiveEvidenceError::ProjectionInvalid)?;
    let websocket_json = extract_single_field(websocket_document, LIVE_WEBSOCKET_JSON_FIELD)
        .ok_or(SubstantiveEvidenceError::ProjectionInvalid)?;
    let maybe_retained_json =
        extract_single_field(retained_document, RETAINED_SUBSTANCE_JSON_FIELD);

    let api_value = parse_value(api_json)?;
    let websocket_value = parse_value(websocket_json)?;
    let maybe_retained_value = maybe_retained_json.map(parse_value).transpose()?;
    let insufficiencies =
        substantive_insufficiencies(&api_value, &websocket_value, maybe_retained_value.as_ref());
    if !insufficiencies.is_empty() {
        return Ok(SubstantiveEvidenceAdmission::Insufficient {
            component_insufficiencies: insufficiencies,
        });
    }

    let retained_value = maybe_retained_value.ok_or(SubstantiveEvidenceError::ProjectionInvalid)?;
    let api = validate_projection(parse_projection(api_value)?, true, true)?;
    let websocket = validate_projection(parse_projection(websocket_value)?, true, true)?;
    let retained = validate_projection(parse_projection(retained_value)?, true, true)?;
    if api != websocket || api != retained {
        return Err(SubstantiveEvidenceError::MixedSnapshotProvenance);
    }

    Ok(SubstantiveEvidenceAdmission::Validated {
        evidence: Box::new(ValidatedSubstantiveEvidence {
            sensors: api
                .maybe_sensors
                .ok_or(SubstantiveEvidenceError::ProjectionInvalid)?,
            runtime_health: api
                .maybe_runtime_health
                .ok_or(SubstantiveEvidenceError::ProjectionInvalid)?,
            join: api.join,
        }),
    })
}

pub fn validate_substantive_snapshot_components(
    api_document: &str,
    websocket_document: &str,
    retained_document: &str,
) -> Result<ValidatedSubstantiveComponents, SubstantiveEvidenceError> {
    if !validate_operator_snapshot_documents(api_document, websocket_document, retained_document)
        .is_empty()
    {
        return Err(SubstantiveEvidenceError::OperatorSnapshotIdentityInvalid);
    }

    let api_value = parse_value(
        extract_single_field(api_document, SYSTEM_INFO_JSON_FIELD)
            .ok_or(SubstantiveEvidenceError::ProjectionInvalid)?,
    )?;
    let websocket_value = parse_value(
        extract_single_field(websocket_document, LIVE_WEBSOCKET_JSON_FIELD)
            .ok_or(SubstantiveEvidenceError::ProjectionInvalid)?,
    )?;
    let retained_value = parse_value(
        extract_single_field(retained_document, RETAINED_SUBSTANCE_JSON_FIELD)
            .ok_or(SubstantiveEvidenceError::ProjectionInvalid)?,
    )?;
    let component_insufficiencies =
        substantive_insufficiencies(&api_value, &websocket_value, Some(&retained_value));
    let require_sensors =
        !component_insufficiencies.contains(&ComponentInsufficiency::SnapshotSubstance);
    let require_health =
        !component_insufficiencies.contains(&ComponentInsufficiency::RuntimeHealth);
    let api = validate_projection(
        parse_projection(api_value)?,
        require_sensors,
        require_health,
    )?;
    let websocket = validate_projection(
        parse_projection(websocket_value)?,
        require_sensors,
        require_health,
    )?;
    let retained = validate_projection(
        parse_projection(retained_value)?,
        require_sensors,
        require_health,
    )?;
    if api != websocket || api != retained {
        return Err(SubstantiveEvidenceError::MixedSnapshotProvenance);
    }

    Ok(ValidatedSubstantiveComponents {
        maybe_sensors: api.maybe_sensors,
        maybe_runtime_health: api.maybe_runtime_health,
        join: api.join,
        component_insufficiencies,
    })
}

fn parse_value(json: &str) -> Result<serde_json::Value, SubstantiveEvidenceError> {
    serde_json::from_str(json).map_err(|_| SubstantiveEvidenceError::ProjectionInvalid)
}

fn parse_projection(value: serde_json::Value) -> Result<RawProjection, SubstantiveEvidenceError> {
    serde_json::from_value(value).map_err(|_| SubstantiveEvidenceError::ProjectionInvalid)
}

fn substantive_insufficiencies(
    api: &serde_json::Value,
    websocket: &serde_json::Value,
    maybe_retained: Option<&serde_json::Value>,
) -> Vec<ComponentInsufficiency> {
    let surfaces = [Some(api), Some(websocket), maybe_retained];
    let sensor_fields = [
        "current",
        "voltage",
        "power",
        "temp",
        "fanrpm",
        "currentStatus",
        "voltageStatus",
        "powerStatus",
        "chipTempStatus",
        "fanRpmStatus",
    ];
    let sensors_complete = surfaces
        .iter()
        .all(|maybe_surface| fields_present(*maybe_surface, &sensor_fields));
    let health_complete = surfaces
        .iter()
        .all(|maybe_surface| fields_present(*maybe_surface, &["runtimeHealth"]));

    let mut insufficiencies = Vec::new();
    if !sensors_complete {
        insufficiencies.push(ComponentInsufficiency::SnapshotSubstance);
    }
    if !health_complete {
        insufficiencies.push(ComponentInsufficiency::RuntimeHealth);
    }
    insufficiencies
}

fn fields_present(maybe_value: Option<&serde_json::Value>, fields: &[&str]) -> bool {
    let Some(object) = maybe_value.and_then(serde_json::Value::as_object) else {
        return false;
    };
    fields.iter().all(|field| object.contains_key(*field))
}

fn validate_projection(
    raw: RawProjection,
    require_sensors: bool,
    require_health: bool,
) -> Result<ValidatedProjection, SubstantiveEvidenceError> {
    if raw.boot_session.len() != 32
        || !raw
            .boot_session
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        || raw.operator_snapshot_revision == 0
    {
        return Err(SubstantiveEvidenceError::MixedSnapshotProvenance);
    }
    let maybe_sensor_values = if require_sensors {
        let (Some(current), Some(voltage), Some(power), Some(temp), Some(fan_rpm)) =
            (raw.current, raw.voltage, raw.power, raw.temp, raw.fan_rpm)
        else {
            return Err(SubstantiveEvidenceError::ProjectionInvalid);
        };
        let (
            Some(current_status),
            Some(voltage_status),
            Some(power_status),
            Some(chip_temp_status),
            Some(fan_rpm_status),
        ) = (
            raw.current_status,
            raw.voltage_status,
            raw.power_status,
            raw.chip_temp_status,
            raw.fan_rpm_status,
        )
        else {
            return Err(SubstantiveEvidenceError::ProjectionInvalid);
        };
        if current_status != voltage_status || current_status != power_status {
            return Err(SubstantiveEvidenceError::AtomicPowerObservationMismatch);
        }
        let power_state = validate_observation_state(
            &current_status,
            current == 0.0 && voltage == 0.0 && power == 0.0,
        )?;
        let temperature_state = validate_observation_state(&chip_temp_status, temp == 0.0)?;
        let tachometer_state = validate_observation_state(&fan_rpm_status, fan_rpm == 0)?;
        reject_reused_unrelated_stamps(&power_state, &temperature_state, &tachometer_state)?;
        let maybe_producer_boot_session =
            shared_producer_boot_session(&power_state, &temperature_state, &tachometer_state)?;
        let power = ValidatedPowerObservation {
            maybe_current_milliamps: fresh_milli_value(&power_state, current)?,
            maybe_bus_millivolts: fresh_milli_value(&power_state, voltage)?,
            maybe_power_milliwatts: fresh_milli_value(&power_state, power)?,
            state: power_state,
        };
        let temperature = ValidatedScalarObservation {
            maybe_value_milliunits: fresh_milli_value(&temperature_state, temp)?,
            state: temperature_state,
        };
        let tachometer = ValidatedScalarObservation {
            maybe_value_milliunits: fresh_milli_value(&tachometer_state, fan_rpm as f64)?,
            state: tachometer_state,
        };
        Some((power, temperature, tachometer, maybe_producer_boot_session))
    } else {
        None
    };
    let maybe_runtime_health = if require_health {
        Some(validate_runtime_health(
            raw.runtime_health
                .ok_or(SubstantiveEvidenceError::ProjectionInvalid)?,
        )?)
    } else {
        None
    };
    let maybe_power_stamp = maybe_sensor_values
        .as_ref()
        .and_then(|(power, _, _, _)| power.state.maybe_stamp().cloned());
    let maybe_temperature_stamp = maybe_sensor_values
        .as_ref()
        .and_then(|(_, temperature, _, _)| temperature.state.maybe_stamp().cloned());
    let maybe_tachometer_stamp = maybe_sensor_values
        .as_ref()
        .and_then(|(_, _, tachometer, _)| tachometer.state.maybe_stamp().cloned());
    let join = SubstantiveSnapshotJoin {
        operator_boot_session_digest: sha256_hex(raw.boot_session.as_bytes()),
        operator_snapshot_revision: raw.operator_snapshot_revision,
        maybe_producer_boot_session: maybe_sensor_values
            .as_ref()
            .and_then(|(_, _, _, session)| *session),
        maybe_power_stamp,
        maybe_temperature_stamp,
        maybe_tachometer_stamp,
    };
    let maybe_sensors = maybe_sensor_values
        .map(|(power, temperature, tachometer, _)| {
            let sensor_digest = digest_serializable(&(&power, &temperature, &tachometer, &join))?;
            Ok(ValidatedSensorSubstance {
                power,
                temperature,
                tachometer,
                claim_fact_digest: sensor_digest,
            })
        })
        .transpose()?;
    let maybe_runtime_health = maybe_runtime_health
        .map(|runtime_health| {
            let health_digest = digest_serializable(&(&runtime_health, &join))?;
            Ok(ValidatedRuntimeHealthSubstance {
                claim_fact_digest: health_digest,
                ..runtime_health
            })
        })
        .transpose()?;

    Ok(ValidatedProjection {
        maybe_sensors,
        maybe_runtime_health,
        join,
    })
}

fn validate_observation_state(
    raw: &RawObservationTruth,
    compatibility_value_is_zero: bool,
) -> Result<ObservationState, SubstantiveEvidenceError> {
    let state = match raw.state {
        RawObservationState::Fresh => {
            let stamp = valid_stamp(raw.stamp.as_ref())?;
            if compatibility_value_is_zero || raw.reason.is_some() {
                return Err(SubstantiveEvidenceError::ContradictorySensorState);
            }
            ObservationState::Fresh {
                stamp: stamp.clone(),
            }
        }
        RawObservationState::Stale => {
            let stamp = valid_stamp(raw.stamp.as_ref())?;
            let reason = parse_stale_reason(raw.reason.as_ref())?;
            if !compatibility_value_is_zero {
                return Err(SubstantiveEvidenceError::ContradictorySensorState);
            }
            ObservationState::Stale {
                stamp: stamp.clone(),
                reason,
            }
        }
        RawObservationState::Unavailable => {
            if raw.stamp.is_some() || !compatibility_value_is_zero {
                return Err(SubstantiveEvidenceError::ContradictorySensorState);
            }
            ObservationState::Unavailable {
                reason: parse_unavailable_reason(raw.reason.as_ref())?,
            }
        }
        RawObservationState::Fault => {
            if !compatibility_value_is_zero {
                return Err(SubstantiveEvidenceError::ContradictorySensorState);
            }
            if let Some(stamp) = raw.stamp.as_ref() {
                valid_stamp(Some(stamp))?;
            }
            ObservationState::Fault {
                maybe_stamp: raw.stamp.clone(),
                reason: parse_fault_reason(raw.reason.as_ref())?,
            }
        }
    };
    Ok(state)
}

fn valid_stamp(
    maybe_stamp: Option<&ObservationStamp>,
) -> Result<&ObservationStamp, SubstantiveEvidenceError> {
    let Some(stamp) = maybe_stamp else {
        return Err(SubstantiveEvidenceError::ContradictorySensorState);
    };
    if stamp.boot_session == 0 || stamp.sequence == 0 || stamp.acquired_at_ms == 0 {
        return Err(SubstantiveEvidenceError::ContradictorySensorState);
    }
    Ok(stamp)
}

fn parse_stale_reason(
    maybe_reason: Option<&RawObservationReason>,
) -> Result<StaleReason, SubstantiveEvidenceError> {
    let Some(reason) = maybe_reason.filter(|reason| reason.kind == RawReasonKind::Stale) else {
        return Err(SubstantiveEvidenceError::ContradictorySensorState);
    };
    match reason.code.as_str() {
        "producer_cadence_expired" => Ok(StaleReason::ProducerCadenceExpired),
        "producer_timeout" => Ok(StaleReason::ProducerTimeout),
        "power_sample_stale" => Ok(StaleReason::PowerSampleStale),
        "thermal_sample_stale" => Ok(StaleReason::ThermalSampleStale),
        "tachometer_stale" => Ok(StaleReason::TachometerStale),
        _ => Err(SubstantiveEvidenceError::ContradictorySensorState),
    }
}

fn parse_unavailable_reason(
    maybe_reason: Option<&RawObservationReason>,
) -> Result<UnavailableReason, SubstantiveEvidenceError> {
    let Some(reason) = maybe_reason.filter(|reason| reason.kind == RawReasonKind::Unavailable)
    else {
        return Err(SubstantiveEvidenceError::ContradictorySensorState);
    };
    match reason.code.as_str() {
        "not_yet_observed" => Ok(UnavailableReason::NotYetObserved),
        "producer_unavailable" => Ok(UnavailableReason::ProducerUnavailable),
        "power_sample_unavailable" => Ok(UnavailableReason::PowerSampleUnavailable),
        "thermal_reading_unavailable" => Ok(UnavailableReason::ThermalReadingUnavailable),
        "tachometer_unavailable" => Ok(UnavailableReason::TachometerUnavailable),
        _ => Err(SubstantiveEvidenceError::ContradictorySensorState),
    }
}

fn parse_fault_reason(
    maybe_reason: Option<&RawObservationReason>,
) -> Result<FaultReason, SubstantiveEvidenceError> {
    let Some(reason) = maybe_reason.filter(|reason| reason.kind == RawReasonKind::Fault) else {
        return Err(SubstantiveEvidenceError::ContradictorySensorState);
    };
    match reason.code.as_str() {
        "read_failed" => Ok(FaultReason::ReadFailed),
        "invalid_sample" => Ok(FaultReason::InvalidSample),
        "unsafe_reading" => Ok(FaultReason::UnsafeReading),
        "ina260_read_failed" => Ok(FaultReason::Ina260ReadFailed),
        "input_voltage_unsafe" => Ok(FaultReason::InputVoltageUnsafe),
        "power_limit_exceeded" => Ok(FaultReason::PowerLimitExceeded),
        "power_reading_invalid" => Ok(FaultReason::PowerReadingInvalid),
        "thermal_reading_invalid" => Ok(FaultReason::ThermalReadingInvalid),
        _ => Err(SubstantiveEvidenceError::ContradictorySensorState),
    }
}

fn reject_reused_unrelated_stamps(
    power: &ObservationState,
    temperature: &ObservationState,
    tachometer: &ObservationState,
) -> Result<(), SubstantiveEvidenceError> {
    let stamps = [
        power.maybe_stamp(),
        temperature.maybe_stamp(),
        tachometer.maybe_stamp(),
    ];
    for left in 0..stamps.len() {
        for right in (left + 1)..stamps.len() {
            if stamps[left].is_some() && stamps[left] == stamps[right] {
                return Err(SubstantiveEvidenceError::ReusedUnrelatedObservationStamp);
            }
        }
    }
    Ok(())
}

fn shared_producer_boot_session(
    power: &ObservationState,
    temperature: &ObservationState,
    tachometer: &ObservationState,
) -> Result<Option<u64>, SubstantiveEvidenceError> {
    let mut sessions = [
        power.maybe_stamp(),
        temperature.maybe_stamp(),
        tachometer.maybe_stamp(),
    ]
    .into_iter()
    .flatten()
    .map(|stamp| stamp.boot_session);
    let maybe_session = sessions.next();
    if sessions.any(|session| Some(session) != maybe_session) {
        return Err(SubstantiveEvidenceError::MixedSnapshotProvenance);
    }
    Ok(maybe_session)
}

fn fresh_milli_value(
    state: &ObservationState,
    value: f64,
) -> Result<Option<i64>, SubstantiveEvidenceError> {
    if !matches!(state, ObservationState::Fresh { .. }) {
        return Ok(None);
    }
    if !value.is_finite() {
        return Err(SubstantiveEvidenceError::ContradictorySensorState);
    }
    let scaled = value * 1_000.0;
    if scaled < i64::MIN as f64
        || scaled > i64::MAX as f64
        || (scaled.round() - scaled).abs() > f64::EPSILON
    {
        return Err(SubstantiveEvidenceError::ContradictorySensorState);
    }
    Ok(Some(scaled as i64))
}

fn validate_runtime_health(
    raw: RawRuntimeHealth,
) -> Result<ValidatedRuntimeHealthSubstance, SubstantiveEvidenceError> {
    let lifecycle_state = match raw.self_test_state.as_str() {
        "idle" => RuntimeLifecycleState::Idle,
        "blocked" => RuntimeLifecycleState::Blocked,
        "running" => RuntimeLifecycleState::Running,
        "passed" => RuntimeLifecycleState::Passed,
        "failed" => RuntimeLifecycleState::Failed,
        "canceled" => RuntimeLifecycleState::Canceled,
        "unavailable" => RuntimeLifecycleState::Unavailable,
        _ => return Err(SubstantiveEvidenceError::RuntimeHealthInvalid),
    };
    let checkpoint_health = match raw.checkpoint_health.as_str() {
        "healthy" => CheckpointHealth::Healthy,
        "stale" => CheckpointHealth::Stale,
        "unhealthy" => CheckpointHealth::Unhealthy,
        "unavailable" => CheckpointHealth::Unavailable,
        _ => return Err(SubstantiveEvidenceError::RuntimeHealthInvalid),
    };
    let supervisor_availability = match raw.supervisor_availability.as_str() {
        "available" => SupervisorAvailability::Available,
        "unavailable" => SupervisorAvailability::Unavailable,
        _ => return Err(SubstantiveEvidenceError::RuntimeHealthInvalid),
    };
    let maybe_checkpoint_category = raw
        .maybe_checkpoint_category
        .map(CheckpointCategory::new)
        .transpose()?;
    validate_checkpoint_fields(
        supervisor_availability,
        maybe_checkpoint_category.as_ref(),
        raw.maybe_checkpoint_sequence,
        raw.maybe_checkpoint_age_millis,
        checkpoint_health,
    )?;
    let watchdog_availability = if raw.task_watchdog_participation == "unavailable"
        && raw.maybe_task_watchdog_reason.as_deref() == Some("unproved")
    {
        WatchdogAvailability::Unproved
    } else {
        return Err(SubstantiveEvidenceError::WatchdogObservationNotIndependent);
    };

    Ok(ValidatedRuntimeHealthSubstance {
        lifecycle_state,
        supervisor_availability,
        maybe_checkpoint_category,
        maybe_checkpoint_sequence: raw.maybe_checkpoint_sequence,
        maybe_checkpoint_age_millis: raw.maybe_checkpoint_age_millis,
        checkpoint_health,
        watchdog_availability,
        claim_fact_digest: String::new(),
    })
}

impl CheckpointCategory {
    fn new(value: String) -> Result<Self, SubstantiveEvidenceError> {
        if value.is_empty()
            || !value.is_ascii()
            || value.len() > CHECKPOINT_CATEGORY_MAX_ASCII_BYTES
        {
            return Err(SubstantiveEvidenceError::RuntimeHealthInvalid);
        }
        Ok(Self(value))
    }
}

fn validate_checkpoint_fields(
    supervisor: SupervisorAvailability,
    maybe_category: Option<&CheckpointCategory>,
    maybe_sequence: Option<u64>,
    maybe_age_millis: Option<u64>,
    health: CheckpointHealth,
) -> Result<(), SubstantiveEvidenceError> {
    match supervisor {
        SupervisorAvailability::Unavailable => {
            if maybe_category.is_none()
                && maybe_sequence.is_none()
                && maybe_age_millis.is_none()
                && health == CheckpointHealth::Unavailable
            {
                return Ok(());
            }
            Err(SubstantiveEvidenceError::RuntimeHealthInvalid)
        }
        SupervisorAvailability::Available => {
            let Some(sequence) = maybe_sequence else {
                return Err(SubstantiveEvidenceError::RuntimeHealthInvalid);
            };
            let Some(age_millis) = maybe_age_millis else {
                return Err(SubstantiveEvidenceError::RuntimeHealthInvalid);
            };
            if maybe_category.is_none() || sequence == 0 {
                return Err(SubstantiveEvidenceError::RuntimeHealthInvalid);
            }
            let chronological = match health {
                CheckpointHealth::Healthy => age_millis <= HEALTHY_CHECKPOINT_MAX_AGE_MILLIS,
                CheckpointHealth::Stale => {
                    age_millis > HEALTHY_CHECKPOINT_MAX_AGE_MILLIS
                        && age_millis <= STALE_CHECKPOINT_MAX_AGE_MILLIS
                }
                CheckpointHealth::Unhealthy => age_millis > STALE_CHECKPOINT_MAX_AGE_MILLIS,
                CheckpointHealth::Unavailable => false,
            };
            if !chronological {
                return Err(SubstantiveEvidenceError::CheckpointChronologyInvalid);
            }
            Ok(())
        }
    }
}

fn extract_single_field<'a>(document: &'a str, field: &str) -> Option<&'a str> {
    let prefix = format!("{field}:");
    let mut values = document
        .lines()
        .filter_map(|line| line.trim().strip_prefix(&prefix))
        .map(str::trim);
    let value = values.next()?;
    if value.is_empty() || values.next().is_some() {
        return None;
    }
    Some(value)
}

fn digest_serializable(value: &impl Serialize) -> Result<String, SubstantiveEvidenceError> {
    let bytes =
        serde_json::to_vec(value).map_err(|_| SubstantiveEvidenceError::ProjectionInvalid)?;
    Ok(sha256_hex(&bytes))
}
