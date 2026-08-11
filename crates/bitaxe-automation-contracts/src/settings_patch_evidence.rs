use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{AutomationCommand, WorkflowIdentity, SETTINGS_PATCH_EVIDENCE_SCHEMA};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct SettingsPatchObservationEvidence {
    pub hostname_baseline_sha256: String,
    pub hostname_candidate_sha256: String,
    pub rotation_baseline_sha256: String,
    pub rotation_candidate_sha256: String,
    pub mutation_request_field_count: u64,
    pub mutation_request_atomic: bool,
    pub immediate_combined_readback: bool,
    pub restoration_request_field_count: u64,
    pub restoration_request_atomic: bool,
    pub restoration_complete: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct SettingsPatchEvidence {
    pub schema_version: String,
    pub board: u16,
    pub source_commit: String,
    pub reference_commit: String,
    pub package_manifest_sha256: String,
    pub workflow: WorkflowIdentity,
    pub detector_admitted: bool,
    pub boot_observed: bool,
    pub same_origin_observed: bool,
    pub settings_patch: SettingsPatchObservationEvidence,
    pub mining_state: String,
    pub hardware_control_state: String,
    pub cleanup_complete: bool,
    pub redaction_status: String,
}

impl SettingsPatchEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != SETTINGS_PATCH_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("settings PATCH evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::CaptureSettingsPatchEvidence
        {
            return Err("settings PATCH workflow identity is invalid");
        }
        for digest in [
            self.package_manifest_sha256.as_str(),
            self.workflow.request_sha256.as_str(),
            self.settings_patch.hostname_baseline_sha256.as_str(),
            self.settings_patch.hostname_candidate_sha256.as_str(),
            self.settings_patch.rotation_baseline_sha256.as_str(),
            self.settings_patch.rotation_candidate_sha256.as_str(),
        ] {
            if digest.len() != 64
                || !digest
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            {
                return Err("settings PATCH evidence digest is invalid");
            }
        }
        let patch = &self.settings_patch;
        if patch.mutation_request_field_count != 2
            || !patch.mutation_request_atomic
            || !patch.immediate_combined_readback
            || patch.restoration_request_field_count != 2
            || !patch.restoration_request_atomic
            || !patch.restoration_complete
        {
            return Err("settings PATCH observation is incomplete");
        }
        if !self.detector_admitted
            || !self.boot_observed
            || !self.same_origin_observed
            || self.mining_state != "disabled"
            || self.hardware_control_state != "disabled"
            || !self.cleanup_complete
            || self.redaction_status != "passed"
        {
            return Err("settings PATCH safety or privacy evidence is invalid");
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{SettingsPatchEvidence, SettingsPatchObservationEvidence};
    use crate::{AutomationCommand, WorkflowIdentity};

    fn valid_evidence() -> SettingsPatchEvidence {
        SettingsPatchEvidence {
            schema_version: "bitaxe-settings-patch-evidence-v1".to_owned(),
            board: 205,
            source_commit: "a".repeat(40),
            reference_commit: "b".repeat(40),
            package_manifest_sha256: "c".repeat(64),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::CaptureSettingsPatchEvidence,
                request_sha256: "d".repeat(64),
            },
            detector_admitted: true,
            boot_observed: true,
            same_origin_observed: true,
            settings_patch: SettingsPatchObservationEvidence {
                hostname_baseline_sha256: "e".repeat(64),
                hostname_candidate_sha256: "f".repeat(64),
                rotation_baseline_sha256: "1".repeat(64),
                rotation_candidate_sha256: "2".repeat(64),
                mutation_request_field_count: 2,
                mutation_request_atomic: true,
                immediate_combined_readback: true,
                restoration_request_field_count: 2,
                restoration_request_atomic: true,
                restoration_complete: true,
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
    fn partial_mutation_is_rejected() {
        // Arrange
        let mut evidence = valid_evidence();
        evidence.settings_patch.mutation_request_field_count = 1;

        // Act
        let result = evidence.validate();

        // Assert
        assert_eq!(result, Err("settings PATCH observation is incomplete"));
    }
}
