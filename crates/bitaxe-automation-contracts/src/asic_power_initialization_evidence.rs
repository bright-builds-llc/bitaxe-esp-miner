use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, ASIC_POWER_INITIALIZATION_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AsicPowerInitializationSourceEvidence {
    pub initialization_projection_sha256: String,
    pub initialization_projection_current_commit: String,
    pub initialization_projection_valid: bool,
    pub source_task_sha256: String,
    pub plan_sha256: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AsicPowerInitializationObservationEvidence {
    pub profile: String,
    pub frequency_mhz: u16,
    pub core_voltage_command_mv: u16,
    pub fan_duty_command_percent: u8,
    pub preparation_step_count: u8,
    pub accepted_preparation_event_count: u8,
    pub fresh_safety_required_before_effects: bool,
    pub fan_full_commanded_before_voltage: bool,
    pub post_command_nonzero_fan_rpm_required: bool,
    pub core_voltage_stabilization_ms: u16,
    pub asic_enable_active_low: bool,
    pub reset_and_detect_completed: bool,
    pub exactly_one_chip_detected_after_reset: bool,
    pub mining_ready_initialization_completed: bool,
    pub production_uart_retained: bool,
    pub accepted_submit_observed: bool,
    pub rollback_step_count: u8,
    pub rollback_attempts_all_steps: bool,
    pub initial_preparation_failure_primary: bool,
    pub safe_stop_asic_disable_commanded: bool,
    pub unchanged_path_count: u8,
    pub semantic_path_count: u8,
    pub source_semantics_admitted: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AsicPowerInitializationEvidence {
    pub schema_version: String,
    pub board: u16,
    pub attempt_source_commit: String,
    pub current_source_commit: String,
    pub reference_commit: String,
    pub workflow: WorkflowIdentity,
    pub source: AsicPowerInitializationSourceEvidence,
    pub power_initialization: AsicPowerInitializationObservationEvidence,
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

impl AsicPowerInitializationEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != ASIC_POWER_INITIALIZATION_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("ASIC power initialization evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::ProjectAsicPowerInitializationEvidence
        {
            return Err("ASIC power initialization workflow identity is invalid");
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
                return Err("ASIC power initialization source identity is invalid");
            }
        }
        for digest in [
            self.workflow.request_sha256.as_str(),
            self.source.initialization_projection_sha256.as_str(),
            self.source.source_task_sha256.as_str(),
            self.source.plan_sha256.as_str(),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("ASIC power initialization digest is invalid");
            }
        }
        if !self.source.initialization_projection_valid {
            return Err("ASIC power initialization source projection is invalid");
        }

        let power = &self.power_initialization;
        if power.profile != "conservative"
            || power.frequency_mhz != 400
            || power.core_voltage_command_mv != 1_100
            || power.fan_duty_command_percent != 100
            || power.preparation_step_count != 9
            || power.accepted_preparation_event_count != 18
            || !power.fresh_safety_required_before_effects
            || !power.fan_full_commanded_before_voltage
            || !power.post_command_nonzero_fan_rpm_required
            || power.core_voltage_stabilization_ms != 500
            || !power.asic_enable_active_low
            || !power.reset_and_detect_completed
            || !power.exactly_one_chip_detected_after_reset
            || !power.mining_ready_initialization_completed
            || !power.production_uart_retained
            || !power.accepted_submit_observed
            || power.rollback_step_count != 8
            || !power.rollback_attempts_all_steps
            || !power.initial_preparation_failure_primary
            || !power.safe_stop_asic_disable_commanded
            || power.unchanged_path_count != 6
            || power.semantic_path_count != 3
            || !power.source_semantics_admitted
        {
            return Err("ASIC power initialization observation is incomplete");
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
            return Err("ASIC power initialization campaign or cleanup evidence is invalid");
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

    fn evidence() -> AsicPowerInitializationEvidence {
        AsicPowerInitializationEvidence {
            schema_version: ASIC_POWER_INITIALIZATION_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            attempt_source_commit: "a".repeat(40),
            current_source_commit: "b".repeat(40),
            reference_commit: "c".repeat(40),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::ProjectAsicPowerInitializationEvidence,
                request_sha256: "d".repeat(64),
            },
            source: AsicPowerInitializationSourceEvidence {
                initialization_projection_sha256: "e".repeat(64),
                initialization_projection_current_commit: "f".repeat(40),
                initialization_projection_valid: true,
                source_task_sha256: "0".repeat(64),
                plan_sha256: "1".repeat(64),
            },
            power_initialization: AsicPowerInitializationObservationEvidence {
                profile: "conservative".to_owned(),
                frequency_mhz: 400,
                core_voltage_command_mv: 1_100,
                fan_duty_command_percent: 100,
                preparation_step_count: 9,
                accepted_preparation_event_count: 18,
                fresh_safety_required_before_effects: true,
                fan_full_commanded_before_voltage: true,
                post_command_nonzero_fan_rpm_required: true,
                core_voltage_stabilization_ms: 500,
                asic_enable_active_low: true,
                reset_and_detect_completed: true,
                exactly_one_chip_detected_after_reset: true,
                mining_ready_initialization_completed: true,
                production_uart_retained: true,
                accepted_submit_observed: true,
                rollback_step_count: 8,
                rollback_attempts_all_steps: true,
                initial_preparation_failure_primary: true,
                safe_stop_asic_disable_commanded: true,
                unchanged_path_count: 6,
                semantic_path_count: 3,
                source_semantics_admitted: true,
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
    fn altered_stabilization_boundary_is_rejected() {
        // Arrange
        let mut candidate = evidence();
        candidate.power_initialization.core_voltage_stabilization_ms = 499;

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(
            result,
            Err("ASIC power initialization observation is incomplete")
        );
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
            Err("ASIC power initialization campaign or cleanup evidence is invalid")
        );
    }
}
