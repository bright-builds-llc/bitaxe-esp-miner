use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, SYSTEM_INFO_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct SystemInfoObservationEvidence {
    pub boot_session_sha256: String,
    pub http_revision: u64,
    pub websocket_revision: u64,
    pub same_boot_session: bool,
    pub websocket_revision_not_earlier: bool,
    pub field_contract_schema: String,
    pub field_contract_sha256: String,
    pub required_field_count: u64,
    pub unconditional_field_count: u64,
    pub conditional_field_count: u64,
    pub http_unconditional_fields_complete: bool,
    pub websocket_unconditional_fields_complete: bool,
    pub http_field_types_match: bool,
    pub websocket_field_types_match: bool,
    pub inactive_block_fields_absent: bool,
    pub confirmed_setting_fields_present: bool,
    pub retained_http_tuple_matches: bool,
    pub retained_websocket_tuple_matches: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct SystemInfoEvidence {
    pub schema_version: String,
    pub board: u16,
    pub source_commit: String,
    pub reference_commit: String,
    pub package_manifest_sha256: String,
    pub workflow: WorkflowIdentity,
    pub detector_admitted: bool,
    pub boot_observed: bool,
    pub same_origin_observed: bool,
    pub system_info: SystemInfoObservationEvidence,
    pub mining_state: String,
    pub hardware_control_state: String,
    pub cleanup_complete: bool,
    pub redaction_status: String,
}

impl SystemInfoEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != SYSTEM_INFO_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("system info evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::CaptureSystemInfoEvidence
        {
            return Err("system info workflow identity is invalid");
        }
        for digest in [
            self.package_manifest_sha256.as_str(),
            self.workflow.request_sha256.as_str(),
            self.system_info.boot_session_sha256.as_str(),
            self.system_info.field_contract_sha256.as_str(),
        ] {
            if digest.len() != 64
                || !digest
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            {
                return Err("system info evidence digest is invalid");
            }
        }
        let observation = &self.system_info;
        if observation.http_revision == 0
            || observation.websocket_revision < observation.http_revision
            || !observation.same_boot_session
            || !observation.websocket_revision_not_earlier
            || observation.field_contract_schema != "bitaxe-system-info-field-contract-v1"
            || observation.required_field_count != 94
            || observation.unconditional_field_count != 87
            || observation.conditional_field_count != 7
            || !observation.http_unconditional_fields_complete
            || !observation.websocket_unconditional_fields_complete
            || !observation.http_field_types_match
            || !observation.websocket_field_types_match
            || !observation.inactive_block_fields_absent
            || !observation.confirmed_setting_fields_present
            || !observation.retained_http_tuple_matches
            || !observation.retained_websocket_tuple_matches
        {
            return Err("system info observation is incomplete");
        }
        if !self.detector_admitted
            || !self.boot_observed
            || !self.same_origin_observed
            || self.mining_state != "disabled"
            || self.hardware_control_state != "disabled"
            || !self.cleanup_complete
            || self.redaction_status != "passed"
        {
            return Err("system info safety or privacy evidence is invalid");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{SystemInfoEvidence, SystemInfoObservationEvidence};
    use crate::{AutomationCommand, WorkflowIdentity};

    fn valid_evidence() -> SystemInfoEvidence {
        SystemInfoEvidence {
            schema_version: "bitaxe-system-info-evidence-v1".to_owned(),
            board: 205,
            source_commit: "a".repeat(40),
            reference_commit: "b".repeat(40),
            package_manifest_sha256: "c".repeat(64),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::CaptureSystemInfoEvidence,
                request_sha256: "d".repeat(64),
            },
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

    #[test]
    fn valid_closed_projection_is_accepted() {
        // Arrange
        let evidence = valid_evidence();

        // Act
        let result = evidence.validate();

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn incomplete_field_contract_is_rejected() {
        // Arrange
        let mut evidence = valid_evidence();
        evidence.system_info.required_field_count = 93;

        // Act
        let result = evidence.validate();

        // Assert
        assert_eq!(result, Err("system info observation is incomplete"));
    }
}
