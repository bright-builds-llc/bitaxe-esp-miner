use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, DISPLAY_BEHAVIOR_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct DisplayBehaviorSourceEvidence {
    pub display_uat_projection_sha256: String,
    pub command_effects_projection_sha256: String,
    pub source_task_sha256: String,
    pub plan_sha256: String,
    pub source_semantics_admitted: bool,
    pub reference_semantics_admitted: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct DisplayBehaviorObservationEvidence {
    pub identify_request_count: u8,
    pub machine_render_confirmed: bool,
    pub machine_clear_confirmed: bool,
    pub operator_render_confirmed: bool,
    pub operator_clear_confirmed: bool,
    pub exact_panel_admitted: bool,
    pub supported_rotation_count: u8,
    pub inversion_state_count: u8,
    pub timeout_mode_count: u8,
    pub retained_display_owner: bool,
    pub configuration_before_first_render: bool,
    pub edge_triggered_power_commands: bool,
    pub display_failure_isolated: bool,
    pub compatible_path_count: u8,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct DisplayBehaviorEvidence {
    pub schema_version: String,
    pub board: u16,
    pub attempt_source_commit: String,
    pub current_source_commit: String,
    pub reference_commit: String,
    pub workflow: WorkflowIdentity,
    pub source: DisplayBehaviorSourceEvidence,
    pub display: DisplayBehaviorObservationEvidence,
    pub build_identity_matches: bool,
    pub usb_admission_confirmed: bool,
    pub safe_stop_confirmed: bool,
    pub cleanup_complete: bool,
    pub mining_state: String,
    pub hardware_control_state: String,
    pub hardware_rerun_used: bool,
    pub redaction_status: String,
}

impl DisplayBehaviorEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != DISPLAY_BEHAVIOR_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("display-behavior evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::ProjectDisplayBehaviorEvidence
        {
            return Err("display-behavior workflow identity is invalid");
        }
        for commit in [
            self.attempt_source_commit.as_str(),
            self.current_source_commit.as_str(),
            self.reference_commit.as_str(),
        ] {
            if !is_lower_hex(commit, 40) {
                return Err("display-behavior source identity is invalid");
            }
        }
        for digest in [
            self.workflow.request_sha256.as_str(),
            self.source.display_uat_projection_sha256.as_str(),
            self.source.command_effects_projection_sha256.as_str(),
            self.source.source_task_sha256.as_str(),
            self.source.plan_sha256.as_str(),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("display-behavior digest is invalid");
            }
        }
        if !self.source.source_semantics_admitted || !self.source.reference_semantics_admitted {
            return Err("display-behavior source admission is incomplete");
        }
        let display = &self.display;
        if display.identify_request_count != 1
            || !display.machine_render_confirmed
            || !display.machine_clear_confirmed
            || !display.operator_render_confirmed
            || !display.operator_clear_confirmed
            || !display.exact_panel_admitted
            || display.supported_rotation_count != 4
            || display.inversion_state_count != 2
            || display.timeout_mode_count != 3
            || !display.retained_display_owner
            || !display.configuration_before_first_render
            || !display.edge_triggered_power_commands
            || !display.display_failure_isolated
            || display.compatible_path_count != 5
        {
            return Err("display-behavior observation is incomplete");
        }
        if !self.build_identity_matches
            || !self.usb_admission_confirmed
            || !self.safe_stop_confirmed
            || !self.cleanup_complete
            || self.mining_state != "disabled"
            || self.hardware_control_state != "disabled"
            || self.hardware_rerun_used
            || self.redaction_status != "passed"
        {
            return Err("display-behavior campaign or cleanup evidence is invalid");
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

    fn evidence() -> DisplayBehaviorEvidence {
        DisplayBehaviorEvidence {
            schema_version: DISPLAY_BEHAVIOR_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            attempt_source_commit: "a".repeat(40),
            current_source_commit: "b".repeat(40),
            reference_commit: "c".repeat(40),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::ProjectDisplayBehaviorEvidence,
                request_sha256: "d".repeat(64),
            },
            source: DisplayBehaviorSourceEvidence {
                display_uat_projection_sha256: "e".repeat(64),
                command_effects_projection_sha256: "f".repeat(64),
                source_task_sha256: "0".repeat(64),
                plan_sha256: "1".repeat(64),
                source_semantics_admitted: true,
                reference_semantics_admitted: true,
            },
            display: DisplayBehaviorObservationEvidence {
                identify_request_count: 1,
                machine_render_confirmed: true,
                machine_clear_confirmed: true,
                operator_render_confirmed: true,
                operator_clear_confirmed: true,
                exact_panel_admitted: true,
                supported_rotation_count: 4,
                inversion_state_count: 2,
                timeout_mode_count: 3,
                retained_display_owner: true,
                configuration_before_first_render: true,
                edge_triggered_power_commands: true,
                display_failure_isolated: true,
                compatible_path_count: 5,
            },
            build_identity_matches: true,
            usb_admission_confirmed: true,
            safe_stop_confirmed: true,
            cleanup_complete: true,
            mining_state: "disabled".to_owned(),
            hardware_control_state: "disabled".to_owned(),
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
    fn incomplete_operator_quorum_is_rejected() {
        // Arrange
        let mut candidate = evidence();
        candidate.display.operator_clear_confirmed = false;

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(result, Err("display-behavior observation is incomplete"));
    }
}
