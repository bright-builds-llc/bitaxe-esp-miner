use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, UI_WORKFLOW_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct UiWorkflowSourceEvidence {
    pub operator_snapshot_evidence_sha256: String,
    pub browser_attestation_sha256: String,
    pub theme_evidence_sha256: String,
    pub settings_evidence_sha256: String,
    pub log_evidence_sha256: String,
    pub partition_evidence_sha256: String,
    pub rollback_evidence_sha256: String,
    pub implementation_result_sha256: String,
    pub static_ui_contract_sha256: String,
    pub prior_plan_sha256: String,
    pub prior_closure_sha256: String,
    pub current_plan_sha256: String,
    pub compatibility_source_set_sha256: String,
    pub compatibility_path_count: u16,
    pub all_source_evidence_valid: bool,
    pub joined_source_commits_ancestral: bool,
    pub attempt_source_ancestral: bool,
    pub compatibility_paths_unchanged: bool,
    pub compatibility_paths_clean: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct UiWorkflowBrowserEvidence {
    pub expected_route_count: u16,
    pub desktop_route_count: u16,
    pub mobile_route_count: u16,
    pub same_origin_requests_observed: bool,
    pub log_websocket_observed: bool,
    pub mobile_navigation_opened: bool,
    pub mobile_navigation_closed: bool,
    pub write_only_secrets_blank: bool,
    pub no_file_update_disabled: bool,
    pub otawww_unavailable: bool,
    pub console_error_count: u16,
    pub unexpected_request_failure_count: u16,
    pub desktop_viewport_observed: bool,
    pub mobile_viewport_observed: bool,
    pub browser_cleanup_complete: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct UiWorkflowEvidence {
    pub schema_version: String,
    pub board: u16,
    pub attempt_source_commit: String,
    pub projector_source_commit: String,
    pub reference_commit: String,
    pub package_manifest_sha256: String,
    pub app_elf_sha256: String,
    pub www_spiffs_sha256: String,
    pub workflow: WorkflowIdentity,
    pub sources: UiWorkflowSourceEvidence,
    pub browser: UiWorkflowBrowserEvidence,
    pub exact_package_observed: bool,
    pub normal_restart_observed: bool,
    pub mining_state: String,
    pub hardware_control_state: String,
    pub device_cleanup_complete: bool,
    pub private_modes_valid: bool,
    pub hardware_rerun_used: bool,
    pub redaction_status: String,
}

impl UiWorkflowEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != UI_WORKFLOW_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("UI workflow evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::ProjectUiWorkflowEvidence
        {
            return Err("UI workflow identity is invalid");
        }
        for commit in [
            self.attempt_source_commit.as_str(),
            self.projector_source_commit.as_str(),
            self.reference_commit.as_str(),
        ] {
            if !is_lower_hex(commit, 40) {
                return Err("UI workflow source identity is invalid");
            }
        }
        for digest in [
            self.package_manifest_sha256.as_str(),
            self.app_elf_sha256.as_str(),
            self.www_spiffs_sha256.as_str(),
            self.workflow.request_sha256.as_str(),
            self.sources.operator_snapshot_evidence_sha256.as_str(),
            self.sources.browser_attestation_sha256.as_str(),
            self.sources.theme_evidence_sha256.as_str(),
            self.sources.settings_evidence_sha256.as_str(),
            self.sources.log_evidence_sha256.as_str(),
            self.sources.partition_evidence_sha256.as_str(),
            self.sources.rollback_evidence_sha256.as_str(),
            self.sources.implementation_result_sha256.as_str(),
            self.sources.static_ui_contract_sha256.as_str(),
            self.sources.prior_plan_sha256.as_str(),
            self.sources.prior_closure_sha256.as_str(),
            self.sources.current_plan_sha256.as_str(),
            self.sources.compatibility_source_set_sha256.as_str(),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("UI workflow evidence digest is invalid");
            }
        }
        if !self.sources.all_source_evidence_valid
            || !self.sources.joined_source_commits_ancestral
            || !self.sources.attempt_source_ancestral
            || !self.sources.compatibility_paths_unchanged
            || !self.sources.compatibility_paths_clean
            || self.sources.compatibility_path_count != 10
        {
            return Err("UI workflow source evidence is incomplete");
        }
        let browser = &self.browser;
        if browser.expected_route_count != 7
            || browser.desktop_route_count != browser.expected_route_count
            || browser.mobile_route_count != browser.expected_route_count
            || !browser.same_origin_requests_observed
            || !browser.log_websocket_observed
            || !browser.mobile_navigation_opened
            || !browser.mobile_navigation_closed
            || !browser.write_only_secrets_blank
            || !browser.no_file_update_disabled
            || !browser.otawww_unavailable
            || browser.console_error_count != 0
            || browser.unexpected_request_failure_count != 0
            || !browser.desktop_viewport_observed
            || !browser.mobile_viewport_observed
            || !browser.browser_cleanup_complete
        {
            return Err("UI workflow browser evidence is incomplete");
        }
        if !self.exact_package_observed
            || !self.normal_restart_observed
            || self.mining_state != "disabled"
            || self.hardware_control_state != "disabled"
            || !self.device_cleanup_complete
            || !self.private_modes_valid
            || self.hardware_rerun_used
            || self.redaction_status != "passed"
        {
            return Err("UI workflow safety or privacy evidence is invalid");
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

    fn valid_evidence() -> UiWorkflowEvidence {
        UiWorkflowEvidence {
            schema_version: "bitaxe-ui-workflow-evidence-v1".to_owned(),
            board: 205,
            attempt_source_commit: "a".repeat(40),
            projector_source_commit: "b".repeat(40),
            reference_commit: "c".repeat(40),
            package_manifest_sha256: "d".repeat(64),
            app_elf_sha256: "e".repeat(64),
            www_spiffs_sha256: "f".repeat(64),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::ProjectUiWorkflowEvidence,
                request_sha256: "0".repeat(64),
            },
            sources: UiWorkflowSourceEvidence {
                operator_snapshot_evidence_sha256: "1".repeat(64),
                browser_attestation_sha256: "2".repeat(64),
                theme_evidence_sha256: "3".repeat(64),
                settings_evidence_sha256: "4".repeat(64),
                log_evidence_sha256: "5".repeat(64),
                partition_evidence_sha256: "6".repeat(64),
                rollback_evidence_sha256: "7".repeat(64),
                implementation_result_sha256: "8".repeat(64),
                static_ui_contract_sha256: "9".repeat(64),
                prior_plan_sha256: "a".repeat(64),
                prior_closure_sha256: "b".repeat(64),
                current_plan_sha256: "c".repeat(64),
                compatibility_source_set_sha256: "d".repeat(64),
                compatibility_path_count: 10,
                all_source_evidence_valid: true,
                joined_source_commits_ancestral: true,
                attempt_source_ancestral: true,
                compatibility_paths_unchanged: true,
                compatibility_paths_clean: true,
            },
            browser: UiWorkflowBrowserEvidence {
                expected_route_count: 7,
                desktop_route_count: 7,
                mobile_route_count: 7,
                same_origin_requests_observed: true,
                log_websocket_observed: true,
                mobile_navigation_opened: true,
                mobile_navigation_closed: true,
                write_only_secrets_blank: true,
                no_file_update_disabled: true,
                otawww_unavailable: true,
                console_error_count: 0,
                unexpected_request_failure_count: 0,
                desktop_viewport_observed: true,
                mobile_viewport_observed: true,
                browser_cleanup_complete: true,
            },
            exact_package_observed: true,
            normal_restart_observed: true,
            mining_state: "disabled".to_owned(),
            hardware_control_state: "disabled".to_owned(),
            device_cleanup_complete: true,
            private_modes_valid: true,
            hardware_rerun_used: false,
            redaction_status: "passed".to_owned(),
        }
    }

    #[test]
    fn complete_closed_projection_is_accepted() {
        // Arrange
        let evidence = valid_evidence();

        // Act
        let result = evidence.validate();

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn one_missing_mobile_route_is_rejected() {
        // Arrange
        let mut evidence = valid_evidence();
        evidence.browser.mobile_route_count = 6;

        // Act
        let result = evidence.validate();

        // Assert
        assert_eq!(result, Err("UI workflow browser evidence is incomplete"));
    }

    #[test]
    fn source_drift_and_hardware_rerun_are_rejected() {
        // Arrange
        let mut source_drift = valid_evidence();
        source_drift.sources.compatibility_paths_unchanged = false;
        let mut hardware_rerun = valid_evidence();
        hardware_rerun.hardware_rerun_used = true;

        // Act
        let results = [source_drift.validate(), hardware_rerun.validate()];

        // Assert
        assert_eq!(
            results,
            [
                Err("UI workflow source evidence is incomplete"),
                Err("UI workflow safety or privacy evidence is invalid"),
            ]
        );
    }
}
