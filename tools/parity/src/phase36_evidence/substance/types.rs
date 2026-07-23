use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::super::contract::ComponentInsufficiency;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubstantiveEvidenceAdmission {
    Validated {
        evidence: Box<ValidatedSubstantiveEvidence>,
    },
    Insufficient {
        component_insufficiencies: Vec<ComponentInsufficiency>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ValidatedSubstantiveEvidence {
    pub sensors: ValidatedSensorSubstance,
    pub runtime_health: ValidatedRuntimeHealthSubstance,
    pub join: SubstantiveSnapshotJoin,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidatedSensorSubstance {
    pub power: ValidatedPowerObservation,
    pub temperature: ValidatedScalarObservation,
    pub tachometer: ValidatedScalarObservation,
    pub claim_fact_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidatedRuntimeHealthSubstance {
    pub lifecycle_state: RuntimeLifecycleState,
    pub supervisor_availability: SupervisorAvailability,
    pub maybe_checkpoint_category: Option<CheckpointCategory>,
    pub maybe_checkpoint_sequence: Option<u64>,
    pub maybe_checkpoint_age_millis: Option<u64>,
    pub checkpoint_health: CheckpointHealth,
    pub watchdog_availability: WatchdogAvailability,
    pub claim_fact_digest: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SubstantiveSnapshotJoin {
    pub operator_boot_session_digest: String,
    pub operator_snapshot_revision: u64,
    pub maybe_producer_boot_session: Option<u64>,
    pub maybe_power_stamp: Option<ObservationStamp>,
    pub maybe_temperature_stamp: Option<ObservationStamp>,
    pub maybe_tachometer_stamp: Option<ObservationStamp>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidatedPowerObservation {
    pub state: ObservationState,
    pub maybe_current_milliamps: Option<i64>,
    pub maybe_bus_millivolts: Option<i64>,
    pub maybe_power_milliwatts: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ValidatedScalarObservation {
    pub state: ObservationState,
    pub maybe_value_milliunits: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case", tag = "state")]
pub enum ObservationState {
    Fresh {
        stamp: ObservationStamp,
    },
    Stale {
        stamp: ObservationStamp,
        reason: StaleReason,
    },
    Unavailable {
        reason: UnavailableReason,
    },
    Fault {
        maybe_stamp: Option<ObservationStamp>,
        reason: FaultReason,
    },
}

impl ObservationState {
    pub(super) fn maybe_stamp(&self) -> Option<&ObservationStamp> {
        match self {
            Self::Fresh { stamp } | Self::Stale { stamp, .. } => Some(stamp),
            Self::Unavailable { .. } => None,
            Self::Fault { maybe_stamp, .. } => maybe_stamp.as_ref(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ObservationStamp {
    pub boot_session: u64,
    pub sequence: u64,
    pub acquired_at_ms: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum StaleReason {
    ProducerCadenceExpired,
    ProducerTimeout,
    PowerSampleStale,
    ThermalSampleStale,
    TachometerStale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum UnavailableReason {
    NotYetObserved,
    ProducerUnavailable,
    PowerSampleUnavailable,
    ThermalReadingUnavailable,
    TachometerUnavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FaultReason {
    ReadFailed,
    InvalidSample,
    UnsafeReading,
    Ina260ReadFailed,
    InputVoltageUnsafe,
    PowerLimitExceeded,
    PowerReadingInvalid,
    ThermalReadingInvalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeLifecycleState {
    Idle,
    Blocked,
    Running,
    Passed,
    Failed,
    Canceled,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SupervisorAvailability {
    Available,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(transparent)]
pub struct CheckpointCategory(pub(super) String);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointHealth {
    Healthy,
    Stale,
    Unhealthy,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WatchdogAvailability {
    Unproved,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum SubstantiveEvidenceError {
    #[error("operator_snapshot_identity_invalid")]
    OperatorSnapshotIdentityInvalid,
    #[error("substantive_projection_invalid")]
    ProjectionInvalid,
    #[error("contradictory_sensor_state")]
    ContradictorySensorState,
    #[error("atomic_power_observation_mismatch")]
    AtomicPowerObservationMismatch,
    #[error("mixed_snapshot_provenance")]
    MixedSnapshotProvenance,
    #[error("reused_unrelated_observation_stamp")]
    ReusedUnrelatedObservationStamp,
    #[error("runtime_health_invalid")]
    RuntimeHealthInvalid,
    #[error("checkpoint_chronology_invalid")]
    CheckpointChronologyInvalid,
    #[error("watchdog_observation_not_independent")]
    WatchdogObservationNotIndependent,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RawProjection {
    pub(super) boot_session: String,
    pub(super) operator_snapshot_revision: u64,
    pub(super) current: f64,
    pub(super) voltage: f64,
    pub(super) power: f64,
    pub(super) temp: f64,
    #[serde(rename = "fanrpm")]
    pub(super) fan_rpm: u64,
    pub(super) current_status: RawObservationTruth,
    pub(super) voltage_status: RawObservationTruth,
    pub(super) power_status: RawObservationTruth,
    pub(super) chip_temp_status: RawObservationTruth,
    pub(super) fan_rpm_status: RawObservationTruth,
    pub(super) runtime_health: RawRuntimeHealth,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct RawObservationTruth {
    pub(super) state: RawObservationState,
    #[serde(default)]
    pub(super) stamp: Option<ObservationStamp>,
    #[serde(default)]
    pub(super) reason: Option<RawObservationReason>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum RawObservationState {
    Fresh,
    Stale,
    Unavailable,
    Fault,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct RawObservationReason {
    pub(super) kind: RawReasonKind,
    pub(super) code: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum RawReasonKind {
    Stale,
    Unavailable,
    Fault,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct RawRuntimeHealth {
    pub(super) self_test_state: String,
    pub(super) supervisor_availability: String,
    #[serde(rename = "checkpointCategory")]
    pub(super) maybe_checkpoint_category: Option<String>,
    #[serde(rename = "checkpointSequence")]
    pub(super) maybe_checkpoint_sequence: Option<u64>,
    #[serde(rename = "checkpointAgeMillis")]
    pub(super) maybe_checkpoint_age_millis: Option<u64>,
    pub(super) checkpoint_health: String,
    pub(super) task_watchdog_participation: String,
    #[serde(rename = "taskWatchdogReason")]
    pub(super) maybe_task_watchdog_reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ValidatedProjection {
    pub(super) sensors: ValidatedSensorSubstance,
    pub(super) runtime_health: ValidatedRuntimeHealthSubstance,
    pub(super) join: SubstantiveSnapshotJoin,
}
