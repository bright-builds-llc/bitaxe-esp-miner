use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, STRATUM_SOCKET_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct StratumSocketSourceEvidence {
    pub initialization_projection_sha256: String,
    pub initialization_projection_current_commit: String,
    pub initialization_projection_valid: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct StratumSocketObservationEvidence {
    pub command_capacity: u64,
    pub connect_timeout_ms: u64,
    pub read_timeout_ms: u64,
    pub write_timeout_ms: u64,
    pub read_buffer_bytes: u64,
    pub tcp_nodelay_enabled: bool,
    pub typed_connect_write_close_commands: bool,
    pub typed_connected_bytes_failed_closed_events: bool,
    pub transport_epoch_isolation: bool,
    pub authorized_session_required_before_submit: bool,
    pub accepted_submit_observed: bool,
    pub transport_module_unchanged: bool,
    pub owner_and_lifecycle_spans_compatible: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct StratumSocketEvidence {
    pub schema_version: String,
    pub board: u16,
    pub attempt_source_commit: String,
    pub current_source_commit: String,
    pub reference_commit: String,
    pub workflow: WorkflowIdentity,
    pub source: StratumSocketSourceEvidence,
    pub socket: StratumSocketObservationEvidence,
    pub package_admitted: bool,
    pub runtime_identity: String,
    pub runtime_attestation_status: String,
    pub campaign_terminal_category: String,
    pub submit_outcome: String,
    pub safety_status: String,
    pub mine_on_boot_disabled: bool,
    pub safe_stop_confirmed: bool,
    pub lease_cleanup_confirmed: bool,
    pub usb_cleanup_ready: bool,
    pub hardware_rerun_used: bool,
    pub redaction_status: String,
}

impl StratumSocketEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != STRATUM_SOCKET_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("Stratum socket evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::ProjectStratumSocketEvidence
        {
            return Err("Stratum socket workflow identity is invalid");
        }
        for commit in [
            self.attempt_source_commit.as_str(),
            self.current_source_commit.as_str(),
            self.reference_commit.as_str(),
            self.source
                .initialization_projection_current_commit
                .as_str(),
        ] {
            if !is_lower_hex(commit, 40) {
                return Err("Stratum socket source identity is invalid");
            }
        }
        for digest in [
            self.workflow.request_sha256.as_str(),
            self.source.initialization_projection_sha256.as_str(),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("Stratum socket digest is invalid");
            }
        }
        if !self.source.initialization_projection_valid {
            return Err("Stratum socket source evidence is invalid");
        }

        let socket = &self.socket;
        if socket.command_capacity != 8
            || socket.connect_timeout_ms != 5_000
            || socket.read_timeout_ms != 50
            || socket.write_timeout_ms != 2_000
            || socket.read_buffer_bytes != 2_048
            || !socket.tcp_nodelay_enabled
            || !socket.typed_connect_write_close_commands
            || !socket.typed_connected_bytes_failed_closed_events
            || !socket.transport_epoch_isolation
            || !socket.authorized_session_required_before_submit
            || !socket.accepted_submit_observed
            || !socket.transport_module_unchanged
            || !socket.owner_and_lifecycle_spans_compatible
        {
            return Err("Stratum socket observation is incomplete");
        }
        if !self.package_admitted
            || self.runtime_identity != "trusted"
            || self.runtime_attestation_status != "trusted"
            || self.campaign_terminal_category != "submit_response_observed"
            || self.submit_outcome != "accepted"
            || self.safety_status != "fresh"
            || !self.mine_on_boot_disabled
            || !self.safe_stop_confirmed
            || !self.lease_cleanup_confirmed
            || !self.usb_cleanup_ready
            || self.hardware_rerun_used
            || self.redaction_status != "passed"
        {
            return Err("Stratum socket campaign or cleanup evidence is invalid");
        }
        Ok(())
    }
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence() -> StratumSocketEvidence {
        StratumSocketEvidence {
            schema_version: STRATUM_SOCKET_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            attempt_source_commit: "a".repeat(40),
            current_source_commit: "b".repeat(40),
            reference_commit: "c".repeat(40),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::ProjectStratumSocketEvidence,
                request_sha256: "d".repeat(64),
            },
            source: StratumSocketSourceEvidence {
                initialization_projection_sha256: "e".repeat(64),
                initialization_projection_current_commit: "f".repeat(40),
                initialization_projection_valid: true,
            },
            socket: StratumSocketObservationEvidence {
                command_capacity: 8,
                connect_timeout_ms: 5_000,
                read_timeout_ms: 50,
                write_timeout_ms: 2_000,
                read_buffer_bytes: 2_048,
                tcp_nodelay_enabled: true,
                typed_connect_write_close_commands: true,
                typed_connected_bytes_failed_closed_events: true,
                transport_epoch_isolation: true,
                authorized_session_required_before_submit: true,
                accepted_submit_observed: true,
                transport_module_unchanged: true,
                owner_and_lifecycle_spans_compatible: true,
            },
            package_admitted: true,
            runtime_identity: "trusted".to_owned(),
            runtime_attestation_status: "trusted".to_owned(),
            campaign_terminal_category: "submit_response_observed".to_owned(),
            submit_outcome: "accepted".to_owned(),
            safety_status: "fresh".to_owned(),
            mine_on_boot_disabled: true,
            safe_stop_confirmed: true,
            lease_cleanup_confirmed: true,
            usb_cleanup_ready: true,
            hardware_rerun_used: false,
            redaction_status: "passed".to_owned(),
        }
    }

    #[test]
    fn complete_closed_projection_is_accepted() {
        // Arrange
        let candidate = evidence();

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn incomplete_socket_observation_is_rejected() {
        // Arrange
        let mut candidate = evidence();
        candidate.socket.tcp_nodelay_enabled = false;

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(result, Err("Stratum socket observation is incomplete"));
    }

    #[test]
    fn hardware_rerun_is_rejected() {
        // Arrange
        let mut candidate = evidence();
        candidate.hardware_rerun_used = true;

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(
            result,
            Err("Stratum socket campaign or cleanup evidence is invalid")
        );
    }
}
