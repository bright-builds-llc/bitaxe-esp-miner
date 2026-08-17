use bitaxe_core::runtime_health::RuntimeHealthSnapshot;
use serde::{Deserialize, Serialize};

use crate::{BootSessionId, OperatorSnapshotRevision};

/// Additive passive runtime-health projection shared by system-info and live telemetry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeHealthWire {
    #[serde(rename = "selfTestState")]
    pub self_test_state: String,
    #[serde(rename = "supervisorAvailability")]
    pub supervisor_availability: String,
    #[serde(rename = "checkpointCategory")]
    pub maybe_checkpoint_category: Option<String>,
    #[serde(rename = "checkpointSequence")]
    pub maybe_checkpoint_sequence: Option<u64>,
    #[serde(rename = "checkpointAgeMillis")]
    pub maybe_checkpoint_age_millis: Option<u64>,
    #[serde(rename = "checkpointHealth")]
    pub checkpoint_health: String,
    #[serde(rename = "taskWatchdogParticipation")]
    pub task_watchdog_participation: String,
    #[serde(rename = "taskWatchdogReason")]
    pub maybe_task_watchdog_reason: Option<String>,
    #[serde(rename = "taskWatchdogFeedSequence")]
    pub maybe_task_watchdog_feed_sequence: Option<u64>,
    #[serde(rename = "taskWatchdogFeedAgeMillis")]
    pub maybe_task_watchdog_feed_age_millis: Option<u64>,
    #[serde(
        rename = "taskWatchdogReadOutcome",
        default = "uninitialized_read_outcome"
    )]
    pub task_watchdog_read_outcome: String,
    #[serde(rename = "taskWatchdogOwnerPhase", default = "unavailable_owner_phase")]
    pub task_watchdog_owner_phase: String,
    #[serde(
        rename = "taskWatchdogOwnerSubphase",
        default = "unavailable_owner_subphase"
    )]
    pub task_watchdog_owner_subphase: String,
    #[serde(rename = "taskWatchdogWaitState", default = "invalid_wait_state")]
    pub task_watchdog_wait_state: String,
}

impl From<&RuntimeHealthSnapshot> for RuntimeHealthWire {
    fn from(snapshot: &RuntimeHealthSnapshot) -> Self {
        Self {
            self_test_state: snapshot.passive_self_test_state().as_str().to_owned(),
            supervisor_availability: snapshot.supervisor_availability().as_str().to_owned(),
            maybe_checkpoint_category: snapshot.maybe_checkpoint_category().map(str::to_owned),
            maybe_checkpoint_sequence: snapshot.maybe_checkpoint_sequence(),
            maybe_checkpoint_age_millis: snapshot.maybe_checkpoint_age_millis(),
            checkpoint_health: snapshot.checkpoint_health().as_str().to_owned(),
            task_watchdog_participation: snapshot.task_watchdog_participation().as_str().to_owned(),
            maybe_task_watchdog_reason: snapshot.maybe_task_watchdog_reason().map(str::to_owned),
            maybe_task_watchdog_feed_sequence: snapshot.maybe_task_watchdog_feed_sequence(),
            maybe_task_watchdog_feed_age_millis: snapshot.maybe_task_watchdog_feed_age_millis(),
            task_watchdog_read_outcome: snapshot.task_watchdog_read_outcome().as_str().to_owned(),
            task_watchdog_owner_phase: snapshot.task_watchdog_owner_phase().as_str().to_owned(),
            task_watchdog_owner_subphase: snapshot
                .task_watchdog_owner_subphase()
                .as_str()
                .to_owned(),
            task_watchdog_wait_state: snapshot.task_watchdog_wait_state().as_str().to_owned(),
        }
    }
}

fn unavailable_owner_phase() -> String {
    "unavailable".to_owned()
}

fn unavailable_owner_subphase() -> String {
    "unavailable".to_owned()
}

fn uninitialized_read_outcome() -> String {
    "uninitialized".to_owned()
}

fn invalid_wait_state() -> String {
    "invalid_observation".to_owned()
}

/// Renders the redacted retained runtime-health record for one coherent capture.
#[must_use]
pub fn retained_runtime_health_record(
    boot_session: BootSessionId,
    operator_snapshot_revision: OperatorSnapshotRevision,
    snapshot: &RuntimeHealthSnapshot,
) -> String {
    let checkpoint_category = snapshot
        .maybe_checkpoint_category()
        .unwrap_or("unavailable");
    let checkpoint_sequence = optional_u64(snapshot.maybe_checkpoint_sequence());
    let checkpoint_age_millis = optional_u64(snapshot.maybe_checkpoint_age_millis());
    let task_watchdog_reason = snapshot
        .maybe_task_watchdog_reason()
        .unwrap_or("unavailable");
    let task_watchdog_feed_sequence = optional_u64(snapshot.maybe_task_watchdog_feed_sequence());
    let task_watchdog_feed_age_millis =
        optional_u64(snapshot.maybe_task_watchdog_feed_age_millis());

    format!(
        "runtime_health boot_session={boot_session} operator_snapshot_revision={} self_test={} supervisor={} checkpoint_category={checkpoint_category} checkpoint_sequence={checkpoint_sequence} checkpoint_age_millis={checkpoint_age_millis} checkpoint_health={} task_watchdog_participation={} task_watchdog_reason={task_watchdog_reason} task_watchdog_feed_sequence={task_watchdog_feed_sequence} task_watchdog_feed_age_millis={task_watchdog_feed_age_millis} task_watchdog_read_outcome={} task_watchdog_owner_phase={} task_watchdog_owner_subphase={} task_watchdog_wait_state={} redacted=true",
        operator_snapshot_revision.get(),
        snapshot.passive_self_test_state().as_str(),
        snapshot.supervisor_availability().as_str(),
        snapshot.checkpoint_health().as_str(),
        snapshot.task_watchdog_participation().as_str(),
        snapshot.task_watchdog_read_outcome().as_str(),
        snapshot.task_watchdog_owner_phase().as_str(),
        snapshot.task_watchdog_owner_subphase().as_str(),
        snapshot.task_watchdog_wait_state().as_str(),
    )
}

fn optional_u64(maybe_value: Option<u64>) -> String {
    maybe_value.map_or_else(|| "unavailable".to_owned(), |value| value.to_string())
}
