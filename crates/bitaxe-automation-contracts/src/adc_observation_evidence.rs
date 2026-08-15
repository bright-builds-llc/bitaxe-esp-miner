use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, ADC_OBSERVATION_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AdcObservationSourceEvidence {
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
pub struct AdcObservationQuorum {
    pub adc_unit: u8,
    pub adc_channel: u8,
    pub gpio: u8,
    pub attenuation_db: u8,
    pub default_resolution: bool,
    pub curve_calibration: bool,
    pub producer_cadence_ms: u16,
    pub read_only_acquisition: bool,
    pub http_fresh_sample: bool,
    pub websocket_fresh_sample: bool,
    pub finite_positive_millivolts: bool,
    pub plausible_millivolt_range: bool,
    pub sequence_not_regressed: bool,
    pub acquisition_time_not_regressed: bool,
    pub same_boot_session: bool,
    pub exact_public_correlation: bool,
    pub exact_package_identity: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AdcObservationEvidence {
    pub schema_version: String,
    pub board: u16,
    pub attempt_ordinal: u16,
    pub source_commit: String,
    pub reference_commit: String,
    pub package_manifest_sha256: String,
    pub workflow: WorkflowIdentity,
    pub source: AdcObservationSourceEvidence,
    pub adc: AdcObservationQuorum,
    pub detector_admitted: bool,
    pub boot_observed: bool,
    pub mining_state: String,
    pub hardware_control_state: String,
    pub cleanup_complete: bool,
    pub recovery_used: bool,
    pub redaction_status: String,
}

impl AdcObservationEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != ADC_OBSERVATION_EVIDENCE_SCHEMA
            || self.board != 205
            || self.attempt_ordinal != 1
        {
            return Err("ADC observation evidence schema, board, or attempt is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::CaptureAdcObservationEvidence
        {
            return Err("ADC observation workflow identity is invalid");
        }
        for commit in [self.source_commit.as_str(), self.reference_commit.as_str()] {
            if !is_lower_hex(commit, 40) {
                return Err("ADC observation source identity is invalid");
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
                return Err("ADC observation evidence digest is invalid");
            }
        }
        if !self.source.system_info_projection_valid
            || !self.source.protected_modes_valid
            || !self.source.production_source_current
            || !self.source.source_semantics_admitted
            || self.source.compatible_path_count != 7
        {
            return Err("ADC observation source evidence is incomplete");
        }

        let adc = &self.adc;
        if adc.adc_unit != 1
            || adc.adc_channel != 1
            || adc.gpio != 2
            || adc.attenuation_db != 12
            || !adc.default_resolution
            || !adc.curve_calibration
            || adc.producer_cadence_ms != 500
            || !adc.read_only_acquisition
            || !adc.http_fresh_sample
            || !adc.websocket_fresh_sample
            || !adc.finite_positive_millivolts
            || !adc.plausible_millivolt_range
            || !adc.sequence_not_regressed
            || !adc.acquisition_time_not_regressed
            || !adc.same_boot_session
            || !adc.exact_public_correlation
            || !adc.exact_package_identity
        {
            return Err("ADC observation quorum is incomplete");
        }
        if !self.detector_admitted
            || !self.boot_observed
            || self.mining_state != "disabled"
            || self.hardware_control_state != "disabled"
            || !self.cleanup_complete
            || self.recovery_used
            || self.redaction_status != "passed"
        {
            return Err("ADC observation safety, cleanup, or privacy evidence is invalid");
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

    fn evidence() -> AdcObservationEvidence {
        AdcObservationEvidence {
            schema_version: ADC_OBSERVATION_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            attempt_ordinal: 1,
            source_commit: "a".repeat(40),
            reference_commit: "b".repeat(40),
            package_manifest_sha256: "c".repeat(64),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::CaptureAdcObservationEvidence,
                request_sha256: "d".repeat(64),
            },
            source: AdcObservationSourceEvidence {
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
            adc: AdcObservationQuorum {
                adc_unit: 1,
                adc_channel: 1,
                gpio: 2,
                attenuation_db: 12,
                default_resolution: true,
                curve_calibration: true,
                producer_cadence_ms: 500,
                read_only_acquisition: true,
                http_fresh_sample: true,
                websocket_fresh_sample: true,
                finite_positive_millivolts: true,
                plausible_millivolt_range: true,
                sequence_not_regressed: true,
                acquisition_time_not_regressed: true,
                same_boot_session: true,
                exact_public_correlation: true,
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
    fn missing_live_correlation_is_rejected() {
        // Arrange
        let mut candidate = evidence();
        candidate.adc.sequence_not_regressed = false;

        // Act
        let result = candidate.validate();

        // Assert
        assert_eq!(result, Err("ADC observation quorum is incomplete"));
    }
}
