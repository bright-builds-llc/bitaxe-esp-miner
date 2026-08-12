use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, PROTOCOL_COORDINATOR_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ProtocolCoordinatorSourceEvidence {
    pub initialization_projection_sha256: String,
    pub initialization_projection_current_commit: String,
    pub initialization_projection_valid: bool,
    pub work_send_projection_sha256: String,
    pub work_send_projection_current_commit: String,
    pub work_send_projection_valid: bool,
    pub result_parsing_projection_sha256: String,
    pub result_parsing_projection_current_commit: String,
    pub result_parsing_projection_valid: bool,
    pub socket_projection_sha256: String,
    pub socket_projection_current_commit: String,
    pub socket_projection_valid: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ProtocolCoordinatorObservationEvidence {
    pub owner_inbox_capacity: u64,
    pub readiness_reread_cadence_ms: u64,
    pub readiness_gate_count: u64,
    pub single_owner_serialization: bool,
    pub hardware_prepared_before_pool_access: bool,
    pub authorized_before_asic_dispatch: bool,
    pub qualified_result_before_submit: bool,
    pub accepted_submit_observed: bool,
    pub ordered_terminal_safe_stop: bool,
    pub watchdog_feed_in_owner_loop: bool,
    pub coordinator_modules_unchanged: bool,
    pub lifecycle_spans_compatible: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ProtocolCoordinatorEvidence {
    pub schema_version: String,
    pub board: u16,
    pub attempt_source_commit: String,
    pub current_source_commit: String,
    pub reference_commit: String,
    pub workflow: WorkflowIdentity,
    pub source: ProtocolCoordinatorSourceEvidence,
    pub coordinator: ProtocolCoordinatorObservationEvidence,
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

impl ProtocolCoordinatorEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != PROTOCOL_COORDINATOR_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("protocol coordinator evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::ProjectProtocolCoordinatorEvidence
        {
            return Err("protocol coordinator workflow identity is invalid");
        }
        for commit in [
            self.attempt_source_commit.as_str(),
            self.current_source_commit.as_str(),
            self.reference_commit.as_str(),
            self.source
                .initialization_projection_current_commit
                .as_str(),
            self.source.work_send_projection_current_commit.as_str(),
            self.source
                .result_parsing_projection_current_commit
                .as_str(),
            self.source.socket_projection_current_commit.as_str(),
        ] {
            if !is_lower_hex(commit, 40) {
                return Err("protocol coordinator source identity is invalid");
            }
        }
        for digest in [
            self.workflow.request_sha256.as_str(),
            self.source.initialization_projection_sha256.as_str(),
            self.source.work_send_projection_sha256.as_str(),
            self.source.result_parsing_projection_sha256.as_str(),
            self.source.socket_projection_sha256.as_str(),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("protocol coordinator digest is invalid");
            }
        }
        if !self.source.initialization_projection_valid
            || !self.source.work_send_projection_valid
            || !self.source.result_parsing_projection_valid
            || !self.source.socket_projection_valid
        {
            return Err("protocol coordinator source evidence is invalid");
        }

        let coordinator = &self.coordinator;
        if coordinator.owner_inbox_capacity != 16
            || coordinator.readiness_reread_cadence_ms != 1_000
            || coordinator.readiness_gate_count != 6
            || !coordinator.single_owner_serialization
            || !coordinator.hardware_prepared_before_pool_access
            || !coordinator.authorized_before_asic_dispatch
            || !coordinator.qualified_result_before_submit
            || !coordinator.accepted_submit_observed
            || !coordinator.ordered_terminal_safe_stop
            || !coordinator.watchdog_feed_in_owner_loop
            || !coordinator.coordinator_modules_unchanged
            || !coordinator.lifecycle_spans_compatible
        {
            return Err("protocol coordinator observation is incomplete");
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
            return Err("protocol coordinator campaign or cleanup evidence is invalid");
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

    fn evidence() -> ProtocolCoordinatorEvidence {
        ProtocolCoordinatorEvidence {
            schema_version: PROTOCOL_COORDINATOR_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            attempt_source_commit: "a".repeat(40),
            current_source_commit: "b".repeat(40),
            reference_commit: "c".repeat(40),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::ProjectProtocolCoordinatorEvidence,
                request_sha256: "d".repeat(64),
            },
            source: ProtocolCoordinatorSourceEvidence {
                initialization_projection_sha256: "e".repeat(64),
                initialization_projection_current_commit: "f".repeat(40),
                initialization_projection_valid: true,
                work_send_projection_sha256: "1".repeat(64),
                work_send_projection_current_commit: "2".repeat(40),
                work_send_projection_valid: true,
                result_parsing_projection_sha256: "3".repeat(64),
                result_parsing_projection_current_commit: "4".repeat(40),
                result_parsing_projection_valid: true,
                socket_projection_sha256: "5".repeat(64),
                socket_projection_current_commit: "6".repeat(40),
                socket_projection_valid: true,
            },
            coordinator: ProtocolCoordinatorObservationEvidence {
                owner_inbox_capacity: 16,
                readiness_reread_cadence_ms: 1_000,
                readiness_gate_count: 6,
                single_owner_serialization: true,
                hardware_prepared_before_pool_access: true,
                authorized_before_asic_dispatch: true,
                qualified_result_before_submit: true,
                accepted_submit_observed: true,
                ordered_terminal_safe_stop: true,
                watchdog_feed_in_owner_loop: true,
                coordinator_modules_unchanged: true,
                lifecycle_spans_compatible: true,
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
    fn incomplete_coordinator_observation_is_rejected() {
        // Arrange
        let mut candidate = evidence();
        candidate.coordinator.ordered_terminal_safe_stop = false;

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(
            result,
            Err("protocol coordinator observation is incomplete")
        );
    }

    #[test]
    fn invalid_source_or_hardware_rerun_is_rejected() {
        // Arrange
        let mut invalid_source = evidence();
        invalid_source.source.socket_projection_valid = false;
        let mut rerun = evidence();
        rerun.hardware_rerun_used = true;

        // Act / Assert
        assert_eq!(
            invalid_source.validate(),
            Err("protocol coordinator source evidence is invalid")
        );
        assert_eq!(
            rerun.validate(),
            Err("protocol coordinator campaign or cleanup evidence is invalid")
        );
    }
}
