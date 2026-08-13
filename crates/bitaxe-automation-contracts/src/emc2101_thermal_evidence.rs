use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, EMC2101_THERMAL_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct Emc2101ThermalSourceEvidence {
    pub system_info_projection_sha256: String,
    pub api_snapshot_sha256: String,
    pub websocket_snapshot_sha256: String,
    pub plan_sha256: String,
    pub system_info_projection_valid: bool,
    pub protected_modes_valid: bool,
    pub production_source_current: bool,
    pub source_semantics_admitted: bool,
    pub compatible_path_count: u64,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct Emc2101ThermalObservationEvidence {
    pub i2c_address: u8,
    pub internal_temperature_register: u8,
    pub temperature_offset_celsius: i8,
    pub read_only_acquisition: bool,
    pub http_fresh_sample: bool,
    pub websocket_fresh_sample: bool,
    pub finite_plausible_range: bool,
    pub below_throttle_threshold: bool,
    pub same_temperature: bool,
    pub same_state: bool,
    pub same_acquisition_stamp: bool,
    pub same_boot_session: bool,
    pub exact_package_identity: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct Emc2101ThermalEvidence {
    pub schema_version: String,
    pub board: u16,
    pub attempt_ordinal: u16,
    pub source_commit: String,
    pub reference_commit: String,
    pub package_manifest_sha256: String,
    pub workflow: WorkflowIdentity,
    pub source: Emc2101ThermalSourceEvidence,
    pub thermal: Emc2101ThermalObservationEvidence,
    pub detector_admitted: bool,
    pub boot_observed: bool,
    pub mining_state: String,
    pub hardware_control_state: String,
    pub cleanup_complete: bool,
    pub recovery_used: bool,
    pub redaction_status: String,
}

impl Emc2101ThermalEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != EMC2101_THERMAL_EVIDENCE_SCHEMA
            || self.board != 205
            || self.attempt_ordinal != 1
        {
            return Err("EMC2101 thermal evidence schema, board, or attempt is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::CaptureEmc2101ThermalEvidence
        {
            return Err("EMC2101 thermal workflow identity is invalid");
        }
        for commit in [self.source_commit.as_str(), self.reference_commit.as_str()] {
            if !is_lower_hex(commit, 40) {
                return Err("EMC2101 thermal source identity is invalid");
            }
        }
        for digest in [
            self.package_manifest_sha256.as_str(),
            self.workflow.request_sha256.as_str(),
            self.source.system_info_projection_sha256.as_str(),
            self.source.api_snapshot_sha256.as_str(),
            self.source.websocket_snapshot_sha256.as_str(),
            self.source.plan_sha256.as_str(),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("EMC2101 thermal evidence digest is invalid");
            }
        }
        if !self.source.system_info_projection_valid
            || !self.source.protected_modes_valid
            || !self.source.production_source_current
            || !self.source.source_semantics_admitted
            || self.source.compatible_path_count != 7
        {
            return Err("EMC2101 thermal source evidence is incomplete");
        }

        let thermal = &self.thermal;
        if thermal.i2c_address != 0x4c
            || thermal.internal_temperature_register != 0x00
            || thermal.temperature_offset_celsius != 5
            || !thermal.read_only_acquisition
            || !thermal.http_fresh_sample
            || !thermal.websocket_fresh_sample
            || !thermal.finite_plausible_range
            || !thermal.below_throttle_threshold
            || !thermal.same_temperature
            || !thermal.same_state
            || !thermal.same_acquisition_stamp
            || !thermal.same_boot_session
            || !thermal.exact_package_identity
        {
            return Err("EMC2101 thermal observation is incomplete");
        }
        if !self.detector_admitted
            || !self.boot_observed
            || self.mining_state != "disabled"
            || self.hardware_control_state != "disabled"
            || !self.cleanup_complete
            || self.recovery_used
            || self.redaction_status != "passed"
        {
            return Err("EMC2101 thermal safety, cleanup, or privacy evidence is invalid");
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

    fn evidence() -> Emc2101ThermalEvidence {
        Emc2101ThermalEvidence {
            schema_version: EMC2101_THERMAL_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            attempt_ordinal: 1,
            source_commit: "a".repeat(40),
            reference_commit: "b".repeat(40),
            package_manifest_sha256: "c".repeat(64),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::CaptureEmc2101ThermalEvidence,
                request_sha256: "d".repeat(64),
            },
            source: Emc2101ThermalSourceEvidence {
                system_info_projection_sha256: "e".repeat(64),
                api_snapshot_sha256: "f".repeat(64),
                websocket_snapshot_sha256: "1".repeat(64),
                plan_sha256: "2".repeat(64),
                system_info_projection_valid: true,
                protected_modes_valid: true,
                production_source_current: true,
                source_semantics_admitted: true,
                compatible_path_count: 7,
            },
            thermal: Emc2101ThermalObservationEvidence {
                i2c_address: 0x4c,
                internal_temperature_register: 0x00,
                temperature_offset_celsius: 5,
                read_only_acquisition: true,
                http_fresh_sample: true,
                websocket_fresh_sample: true,
                finite_plausible_range: true,
                below_throttle_threshold: true,
                same_temperature: true,
                same_state: true,
                same_acquisition_stamp: true,
                same_boot_session: true,
                exact_package_identity: true,
            },
            detector_admitted: true,
            boot_observed: true,
            mining_state: "disabled".to_owned(),
            hardware_control_state: "disabled".to_owned(),
            cleanup_complete: true,
            recovery_used: false,
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
    fn stale_or_at_threshold_observation_is_rejected() {
        // Arrange
        let mut stale = evidence();
        stale.thermal.http_fresh_sample = false;
        let mut at_threshold = evidence();
        at_threshold.thermal.below_throttle_threshold = false;

        // Act
        let results = [stale.validate(), at_threshold.validate()];

        // Assert
        assert_eq!(
            results,
            [
                Err("EMC2101 thermal observation is incomplete"),
                Err("EMC2101 thermal observation is incomplete"),
            ]
        );
    }
}
