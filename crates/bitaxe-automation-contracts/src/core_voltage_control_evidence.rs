use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, CORE_VOLTAGE_CONTROL_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct CoreVoltageControlSourceEvidence {
    pub power_initialization_projection_sha256: String,
    pub power_initialization_projection_current_commit: String,
    pub power_initialization_projection_valid: bool,
    pub source_task_sha256: String,
    pub plan_sha256: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct CoreVoltageControlObservationEvidence {
    pub target_millivolts: u16,
    pub i2c_address: u8,
    pub output_register: u8,
    pub register_code: u8,
    pub register_write_count: u8,
    pub typed_command_routed: bool,
    pub stabilization_millis: u16,
    pub stabilization_before_asic_enable: bool,
    pub zero_voltage_skips_ds4432u_write: bool,
    pub active_low_disable: bool,
    pub successful_initialized_work_observed: bool,
    pub accepted_submit_observed: bool,
    pub compatible_path_count: u8,
    pub reference_semantics_admitted: bool,
    pub source_semantics_admitted: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct CoreVoltageControlEvidence {
    pub schema_version: String,
    pub board: u16,
    pub attempt_source_commit: String,
    pub current_source_commit: String,
    pub reference_commit: String,
    pub workflow: WorkflowIdentity,
    pub source: CoreVoltageControlSourceEvidence,
    pub voltage_control: CoreVoltageControlObservationEvidence,
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

impl CoreVoltageControlEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != CORE_VOLTAGE_CONTROL_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("core-voltage-control evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::ProjectCoreVoltageControlEvidence
        {
            return Err("core-voltage-control workflow identity is invalid");
        }
        for commit in [
            self.attempt_source_commit.as_str(),
            self.current_source_commit.as_str(),
            self.reference_commit.as_str(),
            self.source
                .power_initialization_projection_current_commit
                .as_str(),
        ] {
            if !is_lower_hex(commit, 40) {
                return Err("core-voltage-control source identity is invalid");
            }
        }
        for digest in [
            self.workflow.request_sha256.as_str(),
            self.source.power_initialization_projection_sha256.as_str(),
            self.source.source_task_sha256.as_str(),
            self.source.plan_sha256.as_str(),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("core-voltage-control digest is invalid");
            }
        }
        if !self.source.power_initialization_projection_valid {
            return Err("core-voltage-control source projection is invalid");
        }

        let voltage = &self.voltage_control;
        if voltage.target_millivolts != 1_100
            || voltage.i2c_address != 0x48
            || voltage.output_register != 0xf8
            || voltage.register_code != 0xe1
            || voltage.register_write_count != 1
            || !voltage.typed_command_routed
            || voltage.stabilization_millis != 500
            || !voltage.stabilization_before_asic_enable
            || !voltage.zero_voltage_skips_ds4432u_write
            || !voltage.active_low_disable
            || !voltage.successful_initialized_work_observed
            || !voltage.accepted_submit_observed
            || voltage.compatible_path_count != 5
            || !voltage.reference_semantics_admitted
            || !voltage.source_semantics_admitted
        {
            return Err("core-voltage-control observation is incomplete");
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
            return Err("core-voltage-control campaign or cleanup evidence is invalid");
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

    fn evidence() -> CoreVoltageControlEvidence {
        CoreVoltageControlEvidence {
            schema_version: CORE_VOLTAGE_CONTROL_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            attempt_source_commit: "a".repeat(40),
            current_source_commit: "b".repeat(40),
            reference_commit: "c".repeat(40),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::ProjectCoreVoltageControlEvidence,
                request_sha256: "d".repeat(64),
            },
            source: CoreVoltageControlSourceEvidence {
                power_initialization_projection_sha256: "e".repeat(64),
                power_initialization_projection_current_commit: "f".repeat(40),
                power_initialization_projection_valid: true,
                source_task_sha256: "0".repeat(64),
                plan_sha256: "1".repeat(64),
            },
            voltage_control: CoreVoltageControlObservationEvidence {
                target_millivolts: 1_100,
                i2c_address: 0x48,
                output_register: 0xf8,
                register_code: 0xe1,
                register_write_count: 1,
                typed_command_routed: true,
                stabilization_millis: 500,
                stabilization_before_asic_enable: true,
                zero_voltage_skips_ds4432u_write: true,
                active_low_disable: true,
                successful_initialized_work_observed: true,
                accepted_submit_observed: true,
                compatible_path_count: 5,
                reference_semantics_admitted: true,
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
    fn altered_register_code_is_rejected() {
        // Arrange
        let mut candidate = evidence();
        candidate.voltage_control.register_code = 0xe0;

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(
            result,
            Err("core-voltage-control observation is incomplete")
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
            Err("core-voltage-control campaign or cleanup evidence is invalid")
        );
    }
}
