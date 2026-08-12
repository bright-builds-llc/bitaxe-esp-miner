use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, INA260_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct Ina260SourceEvidence {
    pub system_info_projection_sha256: String,
    pub api_snapshot_sha256: String,
    pub websocket_snapshot_sha256: String,
    pub final_evidence_sha256: String,
    pub system_info_projection_valid: bool,
    pub protected_modes_valid: bool,
    pub plan_sha256: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct Ina260ObservationEvidence {
    pub i2c_address: u8,
    pub current_register: u8,
    pub bus_voltage_register: u8,
    pub power_register: u8,
    pub complete_register_set: bool,
    pub read_only_acquisition: bool,
    pub http_complete_fresh_sample: bool,
    pub websocket_complete_fresh_sample: bool,
    pub finite_safe_ranges: bool,
    pub same_values: bool,
    pub same_states: bool,
    pub same_acquisition_stamps: bool,
    pub same_boot_session: bool,
    pub exact_package_identity: bool,
    pub source_paths_compatible: bool,
    pub compatible_path_count: u64,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct Ina260Evidence {
    pub schema_version: String,
    pub board: u16,
    pub attempt_source_commit: String,
    pub current_source_commit: String,
    pub reference_commit: String,
    pub package_manifest_sha256: String,
    pub workflow: WorkflowIdentity,
    pub source: Ina260SourceEvidence,
    pub telemetry: Ina260ObservationEvidence,
    pub detector_admitted: bool,
    pub boot_observed: bool,
    pub mining_state: String,
    pub hardware_control_state: String,
    pub cleanup_complete: bool,
    pub hardware_rerun_used: bool,
    pub redaction_status: String,
}

impl Ina260Evidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != INA260_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("INA260 evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::ProjectIna260Evidence
        {
            return Err("INA260 workflow identity is invalid");
        }
        for commit in [
            self.attempt_source_commit.as_str(),
            self.current_source_commit.as_str(),
            self.reference_commit.as_str(),
        ] {
            if !is_lower_hex(commit, 40) {
                return Err("INA260 source identity is invalid");
            }
        }
        for digest in [
            self.package_manifest_sha256.as_str(),
            self.workflow.request_sha256.as_str(),
            self.source.system_info_projection_sha256.as_str(),
            self.source.api_snapshot_sha256.as_str(),
            self.source.websocket_snapshot_sha256.as_str(),
            self.source.final_evidence_sha256.as_str(),
            self.source.plan_sha256.as_str(),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("INA260 evidence digest is invalid");
            }
        }
        if !self.source.system_info_projection_valid || !self.source.protected_modes_valid {
            return Err("INA260 source evidence is incomplete");
        }

        let telemetry = &self.telemetry;
        if telemetry.i2c_address != 0x40
            || telemetry.current_register != 0x01
            || telemetry.bus_voltage_register != 0x02
            || telemetry.power_register != 0x03
            || !telemetry.complete_register_set
            || !telemetry.read_only_acquisition
            || !telemetry.http_complete_fresh_sample
            || !telemetry.websocket_complete_fresh_sample
            || !telemetry.finite_safe_ranges
            || !telemetry.same_values
            || !telemetry.same_states
            || !telemetry.same_acquisition_stamps
            || !telemetry.same_boot_session
            || !telemetry.exact_package_identity
            || !telemetry.source_paths_compatible
            || telemetry.compatible_path_count != 9
        {
            return Err("INA260 telemetry evidence is incomplete");
        }
        if !self.detector_admitted
            || !self.boot_observed
            || self.mining_state != "disabled"
            || self.hardware_control_state != "disabled"
            || !self.cleanup_complete
            || self.hardware_rerun_used
            || self.redaction_status != "passed"
        {
            return Err("INA260 safety, cleanup, or redaction evidence is invalid");
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

    fn evidence() -> Ina260Evidence {
        Ina260Evidence {
            schema_version: INA260_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            attempt_source_commit: "a".repeat(40),
            current_source_commit: "b".repeat(40),
            reference_commit: "c".repeat(40),
            package_manifest_sha256: "d".repeat(64),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::ProjectIna260Evidence,
                request_sha256: "e".repeat(64),
            },
            source: Ina260SourceEvidence {
                system_info_projection_sha256: "f".repeat(64),
                api_snapshot_sha256: "1".repeat(64),
                websocket_snapshot_sha256: "2".repeat(64),
                final_evidence_sha256: "3".repeat(64),
                system_info_projection_valid: true,
                protected_modes_valid: true,
                plan_sha256: "4".repeat(64),
            },
            telemetry: Ina260ObservationEvidence {
                i2c_address: 0x40,
                current_register: 0x01,
                bus_voltage_register: 0x02,
                power_register: 0x03,
                complete_register_set: true,
                read_only_acquisition: true,
                http_complete_fresh_sample: true,
                websocket_complete_fresh_sample: true,
                finite_safe_ranges: true,
                same_values: true,
                same_states: true,
                same_acquisition_stamps: true,
                same_boot_session: true,
                exact_package_identity: true,
                source_paths_compatible: true,
                compatible_path_count: 9,
            },
            detector_admitted: true,
            boot_observed: true,
            mining_state: "disabled".to_owned(),
            hardware_control_state: "disabled".to_owned(),
            cleanup_complete: true,
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
    fn incomplete_freshness_or_correlation_is_rejected() {
        // Arrange
        let mut stale = evidence();
        stale.telemetry.http_complete_fresh_sample = false;
        let mut uncorrelated = evidence();
        uncorrelated.telemetry.same_acquisition_stamps = false;

        // Act
        let results = [stale.validate(), uncorrelated.validate()];

        // Assert
        assert_eq!(
            results,
            [
                Err("INA260 telemetry evidence is incomplete"),
                Err("INA260 telemetry evidence is incomplete"),
            ]
        );
    }

    #[test]
    fn raw_register_or_hardware_rerun_drift_is_rejected() {
        // Arrange
        let mut register_drift = evidence();
        register_drift.telemetry.power_register = 0x04;
        let mut rerun = evidence();
        rerun.hardware_rerun_used = true;

        // Act
        let results = [register_drift.validate(), rerun.validate()];

        // Assert
        assert_eq!(
            results,
            [
                Err("INA260 telemetry evidence is incomplete"),
                Err("INA260 safety, cleanup, or redaction evidence is invalid"),
            ]
        );
    }
}
