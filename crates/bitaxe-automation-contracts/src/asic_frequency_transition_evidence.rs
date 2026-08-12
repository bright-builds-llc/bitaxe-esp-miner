use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, ASIC_FREQUENCY_TRANSITION_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AsicFrequencyTransitionSourceEvidence {
    pub initialization_projection_sha256: String,
    pub initialization_projection_current_commit: String,
    pub initialization_projection_valid: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AsicFrequencyTransitionObservationEvidence {
    pub profile: String,
    pub start_frequency_mhz: u64,
    pub target_frequency_mhz: u64,
    pub step_quarter_mhz: u64,
    pub set_frequency_command_count: u64,
    pub inter_step_delay_count: u64,
    pub inter_step_delay_ms: u64,
    pub increasing: bool,
    pub production_ramp_option_enabled: bool,
    pub all_frequency_actions_completed: bool,
    pub live_initialized_work_observed: bool,
    pub accepted_submit_observed: bool,
    pub ramp_modules_unchanged: bool,
    pub executor_span_compatible: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AsicFrequencyTransitionEvidence {
    pub schema_version: String,
    pub board: u16,
    pub attempt_source_commit: String,
    pub current_source_commit: String,
    pub reference_commit: String,
    pub workflow: WorkflowIdentity,
    pub source: AsicFrequencyTransitionSourceEvidence,
    pub frequency_transition: AsicFrequencyTransitionObservationEvidence,
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

impl AsicFrequencyTransitionEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != ASIC_FREQUENCY_TRANSITION_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("ASIC frequency-transition evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::ProjectAsicFrequencyTransitionEvidence
        {
            return Err("ASIC frequency-transition workflow identity is invalid");
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
                return Err("ASIC frequency-transition source identity is invalid");
            }
        }
        for digest in [
            self.workflow.request_sha256.as_str(),
            self.source.initialization_projection_sha256.as_str(),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("ASIC frequency-transition digest is invalid");
            }
        }
        if !self.source.initialization_projection_valid {
            return Err("ASIC frequency-transition source evidence is invalid");
        }

        let transition = &self.frequency_transition;
        if transition.profile != "conservative"
            || transition.start_frequency_mhz != 50
            || transition.target_frequency_mhz != 400
            || transition.step_quarter_mhz != 25
            || transition.set_frequency_command_count != 56
            || transition.inter_step_delay_count != 56
            || transition.inter_step_delay_ms != 100
            || !transition.increasing
            || !transition.production_ramp_option_enabled
            || !transition.all_frequency_actions_completed
            || !transition.live_initialized_work_observed
            || !transition.accepted_submit_observed
            || !transition.ramp_modules_unchanged
            || !transition.executor_span_compatible
        {
            return Err("ASIC frequency-transition observation is incomplete");
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
            return Err("ASIC frequency-transition campaign or cleanup evidence is invalid");
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

    fn evidence() -> AsicFrequencyTransitionEvidence {
        AsicFrequencyTransitionEvidence {
            schema_version: ASIC_FREQUENCY_TRANSITION_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            attempt_source_commit: "a".repeat(40),
            current_source_commit: "b".repeat(40),
            reference_commit: "c".repeat(40),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::ProjectAsicFrequencyTransitionEvidence,
                request_sha256: "d".repeat(64),
            },
            source: AsicFrequencyTransitionSourceEvidence {
                initialization_projection_sha256: "e".repeat(64),
                initialization_projection_current_commit: "f".repeat(40),
                initialization_projection_valid: true,
            },
            frequency_transition: AsicFrequencyTransitionObservationEvidence {
                profile: "conservative".to_owned(),
                start_frequency_mhz: 50,
                target_frequency_mhz: 400,
                step_quarter_mhz: 25,
                set_frequency_command_count: 56,
                inter_step_delay_count: 56,
                inter_step_delay_ms: 100,
                increasing: true,
                production_ramp_option_enabled: true,
                all_frequency_actions_completed: true,
                live_initialized_work_observed: true,
                accepted_submit_observed: true,
                ramp_modules_unchanged: true,
                executor_span_compatible: true,
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
    fn incomplete_transition_is_rejected() {
        // Arrange
        let mut candidate = evidence();
        candidate.frequency_transition.set_frequency_command_count = 55;

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(
            result,
            Err("ASIC frequency-transition observation is incomplete")
        );
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
            Err("ASIC frequency-transition campaign or cleanup evidence is invalid")
        );
    }
}
