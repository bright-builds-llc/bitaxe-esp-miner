use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AutomationCommand, SystemInfoEvidence, WorkflowIdentity, ULTRA205_DEFAULTS_EVIDENCE_SCHEMA,
};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct Ultra205DefaultsObservationEvidence {
    pub configured_default_field_count: u16,
    pub firmware_matching_field_count: u16,
    pub firmware_all_defaults_match: bool,
    pub api_visible_default_field_count: u16,
    pub http_defaults_match: bool,
    pub websocket_defaults_match: bool,
    pub retained_attestation_matches: bool,
    pub mining_on_boot_disabled: bool,
    pub exact_seed_fixture_sha256: String,
    pub system_info_evidence_sha256: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct Ultra205DefaultsEvidence {
    pub schema_version: String,
    pub board: u16,
    pub source_commit: String,
    pub reference_commit: String,
    pub package_manifest_sha256: String,
    pub workflow: WorkflowIdentity,
    pub detector_admitted: bool,
    pub boot_observed: bool,
    pub system_info: SystemInfoEvidence,
    pub defaults: Ultra205DefaultsObservationEvidence,
    pub mining_state: String,
    pub hardware_control_state: String,
    pub cleanup_complete: bool,
    pub private_modes_valid: bool,
    pub redaction_status: String,
}

impl Ultra205DefaultsEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != ULTRA205_DEFAULTS_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("Ultra 205 defaults evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::CaptureUltra205DefaultsEvidence
        {
            return Err("Ultra 205 defaults workflow identity is invalid");
        }
        self.system_info.validate()?;
        if self.source_commit != self.system_info.source_commit
            || self.reference_commit != self.system_info.reference_commit
            || self.package_manifest_sha256 != self.system_info.package_manifest_sha256
        {
            return Err("Ultra 205 defaults system info identity is inconsistent");
        }
        for digest in [
            self.package_manifest_sha256.as_str(),
            self.workflow.request_sha256.as_str(),
            self.defaults.exact_seed_fixture_sha256.as_str(),
            self.defaults.system_info_evidence_sha256.as_str(),
        ] {
            if digest.len() != 64
                || !digest
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            {
                return Err("Ultra 205 defaults evidence digest is invalid");
            }
        }
        let defaults = &self.defaults;
        if defaults.configured_default_field_count != 27
            || defaults.firmware_matching_field_count != 27
            || !defaults.firmware_all_defaults_match
            || defaults.api_visible_default_field_count != 23
            || !defaults.http_defaults_match
            || !defaults.websocket_defaults_match
            || !defaults.retained_attestation_matches
            || !defaults.mining_on_boot_disabled
        {
            return Err("Ultra 205 configured defaults are incomplete");
        }
        if !self.detector_admitted
            || !self.boot_observed
            || self.mining_state != "disabled"
            || self.hardware_control_state != "disabled"
            || !self.cleanup_complete
            || !self.private_modes_valid
            || self.redaction_status != "passed"
        {
            return Err("Ultra 205 defaults safety or privacy evidence is invalid");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        AutomationCommand, SystemInfoEvidence, SystemInfoObservationEvidence, WorkflowIdentity,
    };

    use super::{Ultra205DefaultsEvidence, Ultra205DefaultsObservationEvidence};

    fn workflow(command: AutomationCommand, value: char) -> WorkflowIdentity {
        WorkflowIdentity {
            schema_version: "bitaxe-workflow-identity-v1".to_owned(),
            command,
            request_sha256: value.to_string().repeat(64),
        }
    }

    fn system_info() -> SystemInfoEvidence {
        SystemInfoEvidence {
            schema_version: "bitaxe-system-info-evidence-v1".to_owned(),
            board: 205,
            source_commit: "a".repeat(40),
            reference_commit: "b".repeat(40),
            package_manifest_sha256: "c".repeat(64),
            workflow: workflow(AutomationCommand::CaptureSystemInfoEvidence, 'd'),
            detector_admitted: true,
            boot_observed: true,
            same_origin_observed: true,
            system_info: SystemInfoObservationEvidence {
                boot_session_sha256: "e".repeat(64),
                http_revision: 7,
                websocket_revision: 8,
                same_boot_session: true,
                websocket_revision_not_earlier: true,
                field_contract_schema: "bitaxe-system-info-field-contract-v1".to_owned(),
                field_contract_sha256: "f".repeat(64),
                required_field_count: 94,
                unconditional_field_count: 87,
                conditional_field_count: 7,
                http_unconditional_fields_complete: true,
                websocket_unconditional_fields_complete: true,
                http_field_types_match: true,
                websocket_field_types_match: true,
                inactive_block_fields_absent: true,
                confirmed_setting_fields_present: true,
                retained_http_tuple_matches: true,
                retained_websocket_tuple_matches: true,
            },
            mining_state: "disabled".to_owned(),
            hardware_control_state: "disabled".to_owned(),
            cleanup_complete: true,
            redaction_status: "passed".to_owned(),
        }
    }

    fn valid_evidence() -> Ultra205DefaultsEvidence {
        Ultra205DefaultsEvidence {
            schema_version: "bitaxe-ultra205-defaults-evidence-v1".to_owned(),
            board: 205,
            source_commit: "a".repeat(40),
            reference_commit: "b".repeat(40),
            package_manifest_sha256: "c".repeat(64),
            workflow: workflow(AutomationCommand::CaptureUltra205DefaultsEvidence, '1'),
            detector_admitted: true,
            boot_observed: true,
            system_info: system_info(),
            defaults: Ultra205DefaultsObservationEvidence {
                configured_default_field_count: 27,
                firmware_matching_field_count: 27,
                firmware_all_defaults_match: true,
                api_visible_default_field_count: 23,
                http_defaults_match: true,
                websocket_defaults_match: true,
                retained_attestation_matches: true,
                mining_on_boot_disabled: true,
                exact_seed_fixture_sha256: "2".repeat(64),
                system_info_evidence_sha256: "3".repeat(64),
            },
            mining_state: "disabled".to_owned(),
            hardware_control_state: "disabled".to_owned(),
            cleanup_complete: true,
            private_modes_valid: true,
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
    fn partial_firmware_match_is_rejected() {
        // Arrange
        let mut evidence = valid_evidence();
        evidence.defaults.firmware_matching_field_count = 26;

        // Act
        let result = evidence.validate();

        // Assert
        assert_eq!(result, Err("Ultra 205 configured defaults are incomplete"));
    }
}
