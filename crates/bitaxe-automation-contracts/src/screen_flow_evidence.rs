use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, SCREEN_FLOW_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ScreenFlowSourceEvidence {
    pub display_uat_projection_sha256: String,
    pub command_effects_projection_sha256: String,
    pub source_task_sha256: String,
    pub plan_sha256: String,
    pub source_semantics_admitted: bool,
    pub reference_semantics_admitted: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ScreenFlowObservationEvidence {
    pub identify_request_count: u8,
    pub machine_render_confirmed: bool,
    pub machine_clear_confirmed: bool,
    pub operator_render_confirmed: bool,
    pub operator_clear_confirmed: bool,
    pub priority_page_count: u8,
    pub intro_page_count: u8,
    pub carousel_page_count: u8,
    pub screen_update_ms: u16,
    pub intro_delay_ms: u16,
    pub carousel_delay_ms: u16,
    pub notification_mask_count: u8,
    pub paused_notification_admitted: bool,
    pub identify_override_admitted: bool,
    pub new_block_statistics_pin_admitted: bool,
    pub bounded_private_frame_admitted: bool,
    pub side_effect_free_projection_admitted: bool,
    pub retained_screen_owner: bool,
    pub change_only_rendering: bool,
    pub priority_power_visibility_admitted: bool,
    pub display_failure_isolated: bool,
    pub compatible_path_count: u8,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct ScreenFlowEvidence {
    pub schema_version: String,
    pub board: u16,
    pub attempt_source_commit: String,
    pub current_source_commit: String,
    pub reference_commit: String,
    pub workflow: WorkflowIdentity,
    pub source: ScreenFlowSourceEvidence,
    pub screen_flow: ScreenFlowObservationEvidence,
    pub build_identity_matches: bool,
    pub usb_admission_confirmed: bool,
    pub safe_stop_confirmed: bool,
    pub cleanup_complete: bool,
    pub mining_state: String,
    pub hardware_control_state: String,
    pub hardware_rerun_used: bool,
    pub redaction_status: String,
}

impl ScreenFlowEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != SCREEN_FLOW_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("screen-flow evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::ProjectScreenFlowEvidence
        {
            return Err("screen-flow workflow identity is invalid");
        }
        for commit in [
            self.attempt_source_commit.as_str(),
            self.current_source_commit.as_str(),
            self.reference_commit.as_str(),
        ] {
            if !is_lower_hex(commit, 40) {
                return Err("screen-flow source identity is invalid");
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
                return Err("screen-flow digest is invalid");
            }
        }
        if !self.source.source_semantics_admitted || !self.source.reference_semantics_admitted {
            return Err("screen-flow source admission is incomplete");
        }
        let screen = &self.screen_flow;
        if screen.identify_request_count != 1
            || !screen.machine_render_confirmed
            || !screen.machine_clear_confirmed
            || !screen.operator_render_confirmed
            || !screen.operator_clear_confirmed
            || screen.priority_page_count != 6
            || screen.intro_page_count != 2
            || screen.carousel_page_count != 4
            || screen.screen_update_ms != 500
            || screen.intro_delay_ms != 3_000
            || screen.carousel_delay_ms != 10_000
            || screen.notification_mask_count != 8
            || !screen.paused_notification_admitted
            || !screen.identify_override_admitted
            || !screen.new_block_statistics_pin_admitted
            || !screen.bounded_private_frame_admitted
            || !screen.side_effect_free_projection_admitted
            || !screen.retained_screen_owner
            || !screen.change_only_rendering
            || !screen.priority_power_visibility_admitted
            || !screen.display_failure_isolated
            || screen.compatible_path_count != 5
        {
            return Err("screen-flow observation is incomplete");
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
            return Err("screen-flow campaign or cleanup evidence is invalid");
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

    fn evidence() -> ScreenFlowEvidence {
        ScreenFlowEvidence {
            schema_version: SCREEN_FLOW_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            attempt_source_commit: "a".repeat(40),
            current_source_commit: "b".repeat(40),
            reference_commit: "c".repeat(40),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::ProjectScreenFlowEvidence,
                request_sha256: "d".repeat(64),
            },
            source: ScreenFlowSourceEvidence {
                display_uat_projection_sha256: "e".repeat(64),
                command_effects_projection_sha256: "f".repeat(64),
                source_task_sha256: "0".repeat(64),
                plan_sha256: "1".repeat(64),
                source_semantics_admitted: true,
                reference_semantics_admitted: true,
            },
            screen_flow: ScreenFlowObservationEvidence {
                identify_request_count: 1,
                machine_render_confirmed: true,
                machine_clear_confirmed: true,
                operator_render_confirmed: true,
                operator_clear_confirmed: true,
                priority_page_count: 6,
                intro_page_count: 2,
                carousel_page_count: 4,
                screen_update_ms: 500,
                intro_delay_ms: 3_000,
                carousel_delay_ms: 10_000,
                notification_mask_count: 8,
                paused_notification_admitted: true,
                identify_override_admitted: true,
                new_block_statistics_pin_admitted: true,
                bounded_private_frame_admitted: true,
                side_effect_free_projection_admitted: true,
                retained_screen_owner: true,
                change_only_rendering: true,
                priority_power_visibility_admitted: true,
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
    fn incomplete_screen_flow_is_rejected() {
        // Arrange
        let mut candidate = evidence();
        candidate.screen_flow.carousel_page_count = 3;

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(result, Err("screen-flow observation is incomplete"));
    }
}
