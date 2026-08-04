use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, RUNTIME_HEALTH_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct RuntimeHealthObservationEvidence {
    pub boot_session_sha256: String,
    pub http_revision: u64,
    pub websocket_revision: u64,
    pub same_boot_session: bool,
    pub websocket_revision_not_earlier: bool,
    pub self_test_state: String,
    pub supervisor_availability: String,
    pub checkpoint_category: String,
    pub http_checkpoint_sequence: u64,
    pub websocket_checkpoint_sequence: u64,
    pub checkpoint_sequence_not_regressed: bool,
    pub checkpoint_health: String,
    pub checkpoint_age_bounded: bool,
    pub task_watchdog_participation: String,
    pub task_watchdog_reason: String,
    pub http_task_watchdog_feed_sequence: u64,
    pub websocket_task_watchdog_feed_sequence: u64,
    pub task_watchdog_feed_sequence_not_regressed: bool,
    pub task_watchdog_feed_age_bounded: bool,
    pub retained_http_tuple_matches: bool,
    pub retained_websocket_tuple_matches: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct RuntimeHealthEvidence {
    pub schema_version: String,
    pub board: u16,
    pub source_commit: String,
    pub reference_commit: String,
    pub package_manifest_sha256: String,
    pub workflow: WorkflowIdentity,
    pub detector_admitted: bool,
    pub boot_observed: bool,
    pub same_origin_observed: bool,
    pub runtime_health: RuntimeHealthObservationEvidence,
    pub mining_state: String,
    pub hardware_control_state: String,
    pub cleanup_complete: bool,
    pub redaction_status: String,
}

impl RuntimeHealthEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != RUNTIME_HEALTH_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("runtime health evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::CaptureRuntimeHealthEvidence
        {
            return Err("runtime health workflow identity is invalid");
        }
        for digest in [
            self.package_manifest_sha256.as_str(),
            self.workflow.request_sha256.as_str(),
            self.runtime_health.boot_session_sha256.as_str(),
        ] {
            if digest.len() != 64
                || !digest
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            {
                return Err("runtime health evidence digest is invalid");
            }
        }
        let health = &self.runtime_health;
        if health.http_revision == 0
            || health.websocket_revision < health.http_revision
            || !health.same_boot_session
            || !health.websocket_revision_not_earlier
            || health.self_test_state != "unavailable"
            || health.supervisor_availability != "available"
            || health.checkpoint_category.is_empty()
            || health.http_checkpoint_sequence == 0
            || health.websocket_checkpoint_sequence < health.http_checkpoint_sequence
            || !health.checkpoint_sequence_not_regressed
            || health.checkpoint_health != "healthy"
            || !health.checkpoint_age_bounded
            || health.task_watchdog_participation != "participating"
            || health.task_watchdog_reason != "feed_fresh"
            || health.http_task_watchdog_feed_sequence == 0
            || health.websocket_task_watchdog_feed_sequence
                < health.http_task_watchdog_feed_sequence
            || !health.task_watchdog_feed_sequence_not_regressed
            || !health.task_watchdog_feed_age_bounded
            || !health.retained_http_tuple_matches
            || !health.retained_websocket_tuple_matches
        {
            return Err("runtime health observation is incomplete");
        }
        if !self.detector_admitted
            || !self.boot_observed
            || !self.same_origin_observed
            || self.mining_state != "disabled"
            || self.hardware_control_state != "disabled"
            || !self.cleanup_complete
            || self.redaction_status != "passed"
        {
            return Err("runtime health safety or privacy evidence is invalid");
        }
        Ok(())
    }
}
