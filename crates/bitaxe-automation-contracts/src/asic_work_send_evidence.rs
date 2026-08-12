use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, ASIC_WORK_SEND_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AsicWorkSendSourceEvidence {
    pub initialization_projection_sha256: String,
    pub initialization_projection_current_commit: String,
    pub initialization_projection_valid: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AsicWorkSendObservationEvidence {
    pub payload_length_bytes: u64,
    pub frame_length_bytes: u64,
    pub job_id_step: u64,
    pub job_id_modulus: u64,
    pub typed_write_frame_action: bool,
    pub production_ready_gate_required: bool,
    pub live_work_observed: bool,
    pub qualified_result_observed: bool,
    pub accepted_submit_observed: bool,
    pub production_uart_retained: bool,
    pub core_paths_unchanged: bool,
    pub compatible_core_path_count: u64,
    pub dispatch_spans_unchanged: bool,
    pub uart_write_span_unchanged: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AsicWorkSendEvidence {
    pub schema_version: String,
    pub board: u16,
    pub attempt_source_commit: String,
    pub current_source_commit: String,
    pub reference_commit: String,
    pub workflow: WorkflowIdentity,
    pub source: AsicWorkSendSourceEvidence,
    pub work_send: AsicWorkSendObservationEvidence,
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

impl AsicWorkSendEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != ASIC_WORK_SEND_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("ASIC work-send evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::ProjectAsicWorkSendEvidence
        {
            return Err("ASIC work-send workflow identity is invalid");
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
                return Err("ASIC work-send source identity is invalid");
            }
        }
        for digest in [
            self.workflow.request_sha256.as_str(),
            self.source.initialization_projection_sha256.as_str(),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("ASIC work-send digest is invalid");
            }
        }
        if !self.source.initialization_projection_valid {
            return Err("ASIC work-send source evidence is invalid");
        }

        let work_send = &self.work_send;
        if work_send.payload_length_bytes != 82
            || work_send.frame_length_bytes != 88
            || work_send.job_id_step != 8
            || work_send.job_id_modulus != 128
            || !work_send.typed_write_frame_action
            || !work_send.production_ready_gate_required
            || !work_send.live_work_observed
            || !work_send.qualified_result_observed
            || !work_send.accepted_submit_observed
            || !work_send.production_uart_retained
            || !work_send.core_paths_unchanged
            || work_send.compatible_core_path_count != 3
            || !work_send.dispatch_spans_unchanged
            || !work_send.uart_write_span_unchanged
        {
            return Err("ASIC work-send observation is incomplete");
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
            return Err("ASIC work-send campaign or cleanup evidence is invalid");
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

    fn evidence() -> AsicWorkSendEvidence {
        AsicWorkSendEvidence {
            schema_version: ASIC_WORK_SEND_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            attempt_source_commit: "a".repeat(40),
            current_source_commit: "b".repeat(40),
            reference_commit: "c".repeat(40),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::ProjectAsicWorkSendEvidence,
                request_sha256: "d".repeat(64),
            },
            source: AsicWorkSendSourceEvidence {
                initialization_projection_sha256: "e".repeat(64),
                initialization_projection_current_commit: "f".repeat(40),
                initialization_projection_valid: true,
            },
            work_send: AsicWorkSendObservationEvidence {
                payload_length_bytes: 82,
                frame_length_bytes: 88,
                job_id_step: 8,
                job_id_modulus: 128,
                typed_write_frame_action: true,
                production_ready_gate_required: true,
                live_work_observed: true,
                qualified_result_observed: true,
                accepted_submit_observed: true,
                production_uart_retained: true,
                core_paths_unchanged: true,
                compatible_core_path_count: 3,
                dispatch_spans_unchanged: true,
                uart_write_span_unchanged: true,
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
    fn incompatible_dispatch_span_is_rejected() {
        // Arrange
        let mut candidate = evidence();
        candidate.work_send.dispatch_spans_unchanged = false;

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(result, Err("ASIC work-send observation is incomplete"));
    }

    #[test]
    fn nonaccepted_submit_is_rejected() {
        // Arrange
        let mut candidate = evidence();
        candidate.submit_outcome = "rejected".to_owned();

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(
            result,
            Err("ASIC work-send campaign or cleanup evidence is invalid")
        );
    }
}
