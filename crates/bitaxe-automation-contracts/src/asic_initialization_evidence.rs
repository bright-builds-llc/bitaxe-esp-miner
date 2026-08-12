use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, ASIC_INITIALIZATION_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AsicInitializationAttemptEvidence {
    pub campaign_result_sha256: String,
    pub diagnostics_sha256: String,
    pub observations_sha256: String,
    pub result_seal_valid: bool,
    pub private_digests_valid: bool,
    pub protected_modes_valid: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AsicInitializationObservationEvidence {
    pub planned_step_count: u64,
    pub accepted_preparation_event_count: u64,
    pub invalid_preparation_event_count: u64,
    pub terminal_preparation_step: String,
    pub terminal_preparation_outcome: String,
    pub all_preparation_steps_completed: bool,
    pub exactly_one_chip_detected: bool,
    pub mining_ready_initialization_completed: bool,
    pub production_uart_retained: bool,
    pub live_initialized_work_observed: bool,
    pub initialization_paths_unchanged: bool,
    pub compatible_path_count: u64,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AsicInitializationEvidence {
    pub schema_version: String,
    pub board: u16,
    pub attempt_source_commit: String,
    pub current_source_commit: String,
    pub reference_commit: String,
    pub source_task_sha256: String,
    pub workflow: WorkflowIdentity,
    pub attempt: AsicInitializationAttemptEvidence,
    pub initialization: AsicInitializationObservationEvidence,
    pub package_admitted: bool,
    pub runtime_identity: String,
    pub runtime_attestation_status: String,
    pub serial_outcome_detail: String,
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

impl AsicInitializationEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != ASIC_INITIALIZATION_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("ASIC initialization evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::ProjectAsicInitializationEvidence
        {
            return Err("ASIC initialization workflow identity is invalid");
        }
        for commit in [
            self.attempt_source_commit.as_str(),
            self.current_source_commit.as_str(),
            self.reference_commit.as_str(),
        ] {
            if !is_lower_hex(commit, 40) {
                return Err("ASIC initialization source identity is invalid");
            }
        }
        for digest in [
            self.source_task_sha256.as_str(),
            self.workflow.request_sha256.as_str(),
            self.attempt.campaign_result_sha256.as_str(),
            self.attempt.diagnostics_sha256.as_str(),
            self.attempt.observations_sha256.as_str(),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("ASIC initialization digest is invalid");
            }
        }
        if !self.attempt.result_seal_valid
            || !self.attempt.private_digests_valid
            || !self.attempt.protected_modes_valid
        {
            return Err("ASIC initialization protected evidence is invalid");
        }

        let initialization = &self.initialization;
        if initialization.planned_step_count != 9
            || initialization.accepted_preparation_event_count != 18
            || initialization.invalid_preparation_event_count != 0
            || initialization.terminal_preparation_step != "retain_production_uart"
            || initialization.terminal_preparation_outcome != "completed"
            || !initialization.all_preparation_steps_completed
            || !initialization.exactly_one_chip_detected
            || !initialization.mining_ready_initialization_completed
            || !initialization.production_uart_retained
            || !initialization.live_initialized_work_observed
            || !initialization.initialization_paths_unchanged
            || initialization.compatible_path_count != 7
        {
            return Err("ASIC initialization observation is incomplete");
        }
        if !self.package_admitted
            || self.runtime_identity != "trusted"
            || self.runtime_attestation_status != "trusted"
            || self.serial_outcome_detail != "clean"
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
            return Err("ASIC initialization campaign or cleanup evidence is invalid");
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

    fn evidence() -> AsicInitializationEvidence {
        AsicInitializationEvidence {
            schema_version: ASIC_INITIALIZATION_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            attempt_source_commit: "a".repeat(40),
            current_source_commit: "b".repeat(40),
            reference_commit: "c".repeat(40),
            source_task_sha256: "d".repeat(64),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::ProjectAsicInitializationEvidence,
                request_sha256: "e".repeat(64),
            },
            attempt: AsicInitializationAttemptEvidence {
                campaign_result_sha256: "f".repeat(64),
                diagnostics_sha256: "0".repeat(64),
                observations_sha256: "1".repeat(64),
                result_seal_valid: true,
                private_digests_valid: true,
                protected_modes_valid: true,
            },
            initialization: AsicInitializationObservationEvidence {
                planned_step_count: 9,
                accepted_preparation_event_count: 18,
                invalid_preparation_event_count: 0,
                terminal_preparation_step: "retain_production_uart".to_owned(),
                terminal_preparation_outcome: "completed".to_owned(),
                all_preparation_steps_completed: true,
                exactly_one_chip_detected: true,
                mining_ready_initialization_completed: true,
                production_uart_retained: true,
                live_initialized_work_observed: true,
                initialization_paths_unchanged: true,
                compatible_path_count: 7,
            },
            package_admitted: true,
            runtime_identity: "trusted".to_owned(),
            runtime_attestation_status: "trusted".to_owned(),
            serial_outcome_detail: "clean".to_owned(),
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
    fn incomplete_preparation_is_rejected() {
        // Arrange
        let mut candidate = evidence();
        candidate.initialization.accepted_preparation_event_count = 17;

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(result, Err("ASIC initialization observation is incomplete"));
    }

    #[test]
    fn hardware_rerun_claim_is_rejected() {
        // Arrange
        let mut candidate = evidence();
        candidate.hardware_rerun_used = true;

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(
            result,
            Err("ASIC initialization campaign or cleanup evidence is invalid")
        );
    }
}
