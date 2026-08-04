use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, OPERATOR_SNAPSHOT_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct OperatorSnapshotEpochEvidence {
    pub boot_session_sha256: String,
    pub http_snapshot_observed: bool,
    pub websocket_snapshot_observed: bool,
    pub same_boot_session: bool,
    pub http_revision: u64,
    pub websocket_revision: u64,
    pub websocket_revision_not_earlier: bool,
    pub retained_log_marker_matches_http: bool,
    pub retained_log_marker_matches_websocket: bool,
    pub substantive_fields_present: bool,
    pub stable_fields_match: bool,
    pub safe_operator_state_confirmed: bool,
    pub substantive_projection_sha256: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct DeviceSessionEvidence {
    pub schema_version: String,
    pub terminal_category: String,
    pub platform_category: String,
    pub board_category: String,
    pub same_physical_device: bool,
    pub stable_enumeration: bool,
    pub reenumerated: bool,
    pub reader_armed: bool,
    pub pre_restart_serial_delivery: bool,
    pub post_restart_serial_delivery: bool,
    pub serial_delivery: String,
    pub request_outcome: String,
    pub request_attempt_count: u64,
    pub service_loss_observed: bool,
    pub trusted_origin_preserved: bool,
    pub application_recovered: bool,
    pub build_identity_matches: bool,
    pub boot_session_changed: bool,
    pub boot_ordinal_advanced_by_one: bool,
    pub software_reset_observed: bool,
    pub postcondition_matches: bool,
    pub cleanup_complete: bool,
    pub usb_disappearance_count: u64,
    pub enumeration_change_count: u64,
    pub serial_byte_count: u64,
    pub http_observation_count: u64,
    pub duration_millis: u64,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct OperatorSnapshotEvidence {
    pub schema_version: String,
    pub board: u16,
    pub source_commit: String,
    pub reference_commit: String,
    pub package_manifest_sha256: String,
    pub workflow: WorkflowIdentity,
    pub baseline_epoch: OperatorSnapshotEpochEvidence,
    pub post_restart_epoch: OperatorSnapshotEpochEvidence,
    pub distinct_boot_sessions: bool,
    pub restart_session: DeviceSessionEvidence,
    pub mining_state: String,
    pub hardware_control_state: String,
    pub cleanup_complete: bool,
    pub redaction_status: String,
}

impl OperatorSnapshotEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != OPERATOR_SNAPSHOT_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("operator snapshot evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::CaptureOperatorSnapshotEvidence
        {
            return Err("operator snapshot workflow identity is invalid");
        }
        for digest in [
            self.package_manifest_sha256.as_str(),
            self.workflow.request_sha256.as_str(),
            self.baseline_epoch.boot_session_sha256.as_str(),
            self.baseline_epoch.substantive_projection_sha256.as_str(),
            self.post_restart_epoch.boot_session_sha256.as_str(),
            self.post_restart_epoch
                .substantive_projection_sha256
                .as_str(),
        ] {
            if digest.len() != 64
                || !digest
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            {
                return Err("operator snapshot evidence digest is invalid");
            }
        }
        if !self.distinct_boot_sessions
            || self.baseline_epoch.boot_session_sha256
                == self.post_restart_epoch.boot_session_sha256
        {
            return Err("operator snapshot boot epochs are not distinct");
        }
        for epoch in [&self.baseline_epoch, &self.post_restart_epoch] {
            if epoch.http_revision == 0
                || epoch.websocket_revision < epoch.http_revision
                || !epoch.http_snapshot_observed
                || !epoch.websocket_snapshot_observed
                || !epoch.same_boot_session
                || !epoch.websocket_revision_not_earlier
                || !epoch.retained_log_marker_matches_http
                || !epoch.retained_log_marker_matches_websocket
                || !epoch.substantive_fields_present
                || !epoch.stable_fields_match
                || !epoch.safe_operator_state_confirmed
            {
                return Err("operator snapshot boot epoch is incomplete");
            }
        }
        let restart = &self.restart_session;
        if restart.schema_version != "esp-device-session-v1"
            || restart.terminal_category != "ready"
            || restart.platform_category != "macos"
            || restart.board_category != "205"
            || restart.request_attempt_count != 1
            || !matches!(
                restart.request_outcome.as_str(),
                "response_received" | "response_missing"
            )
            || !restart.same_physical_device
            || !restart.reader_armed
            || !restart.trusted_origin_preserved
            || !restart.application_recovered
            || !restart.build_identity_matches
            || !restart.boot_session_changed
            || !restart.boot_ordinal_advanced_by_one
            || !restart.software_reset_observed
            || !restart.postcondition_matches
            || !restart.cleanup_complete
        {
            return Err("operator snapshot restart transaction is incomplete");
        }
        if self.mining_state != "disabled"
            || self.hardware_control_state != "disabled"
            || !self.cleanup_complete
            || self.redaction_status != "passed"
        {
            return Err("operator snapshot safety or redaction evidence is invalid");
        }
        Ok(())
    }
}
