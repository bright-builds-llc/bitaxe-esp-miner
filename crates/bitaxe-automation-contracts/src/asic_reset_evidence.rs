use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, ASIC_RESET_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AsicResetSourceEvidence {
    pub initialization_projection_sha256: String,
    pub initialization_projection_current_commit: String,
    pub initialization_projection_valid: bool,
    pub source_task_sha256: String,
    pub plan_sha256: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AsicResetObservationEvidence {
    pub active_low: bool,
    pub low_duration_ms: u64,
    pub high_duration_ms: u64,
    pub reset_and_detect_completed: bool,
    pub exactly_one_chip_detected_after_reset: bool,
    pub accepted_submit_observed: bool,
    pub fail_closed_hold_low: bool,
    pub safe_stop_hold_low: bool,
    pub reset_paths_unchanged: bool,
    pub compatible_path_count: u64,
    pub adapter_semantics_admitted: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AsicResetEvidence {
    pub schema_version: String,
    pub board: u16,
    pub attempt_source_commit: String,
    pub current_source_commit: String,
    pub reference_commit: String,
    pub workflow: WorkflowIdentity,
    pub source: AsicResetSourceEvidence,
    pub reset: AsicResetObservationEvidence,
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

impl AsicResetEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != ASIC_RESET_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("ASIC reset evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::ProjectAsicResetEvidence
        {
            return Err("ASIC reset workflow identity is invalid");
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
                return Err("ASIC reset source identity is invalid");
            }
        }
        for digest in [
            self.workflow.request_sha256.as_str(),
            self.source.initialization_projection_sha256.as_str(),
            self.source.source_task_sha256.as_str(),
            self.source.plan_sha256.as_str(),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("ASIC reset digest is invalid");
            }
        }
        if !self.source.initialization_projection_valid {
            return Err("ASIC reset source projection is invalid");
        }

        let reset = &self.reset;
        if !reset.active_low
            || reset.low_duration_ms != 100
            || reset.high_duration_ms != 100
            || !reset.reset_and_detect_completed
            || !reset.exactly_one_chip_detected_after_reset
            || !reset.accepted_submit_observed
            || !reset.fail_closed_hold_low
            || !reset.safe_stop_hold_low
            || !reset.reset_paths_unchanged
            || reset.compatible_path_count != 6
            || !reset.adapter_semantics_admitted
        {
            return Err("ASIC reset observation is incomplete");
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
            return Err("ASIC reset campaign or cleanup evidence is invalid");
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

    fn evidence() -> AsicResetEvidence {
        AsicResetEvidence {
            schema_version: ASIC_RESET_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            attempt_source_commit: "a".repeat(40),
            current_source_commit: "b".repeat(40),
            reference_commit: "c".repeat(40),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::ProjectAsicResetEvidence,
                request_sha256: "d".repeat(64),
            },
            source: AsicResetSourceEvidence {
                initialization_projection_sha256: "e".repeat(64),
                initialization_projection_current_commit: "f".repeat(40),
                initialization_projection_valid: true,
                source_task_sha256: "0".repeat(64),
                plan_sha256: "1".repeat(64),
            },
            reset: AsicResetObservationEvidence {
                active_low: true,
                low_duration_ms: 100,
                high_duration_ms: 100,
                reset_and_detect_completed: true,
                exactly_one_chip_detected_after_reset: true,
                accepted_submit_observed: true,
                fail_closed_hold_low: true,
                safe_stop_hold_low: true,
                reset_paths_unchanged: true,
                compatible_path_count: 6,
                adapter_semantics_admitted: true,
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
    fn altered_pulse_duration_is_rejected() {
        // Arrange
        let mut candidate = evidence();
        candidate.reset.low_duration_ms = 99;

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(result, Err("ASIC reset observation is incomplete"));
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
            Err("ASIC reset campaign or cleanup evidence is invalid")
        );
    }
}
