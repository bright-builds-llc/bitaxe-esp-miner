use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AutomationCommand, DeviceSessionEvidence, WorkflowIdentity, PARTITION_LAYOUT_EVIDENCE_SCHEMA,
};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct PartitionLayoutObservationEvidence {
    pub partition_table_sha256: String,
    pub ota_image_sha256: String,
    pub required_partition_count: u64,
    pub canonical_layout_matches: bool,
    pub factory_baseline_observed: bool,
    pub ota_0_recovered: bool,
    pub ota_upload_complete: bool,
    pub ota_boot_validation_complete: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct PartitionLayoutEvidence {
    pub schema_version: String,
    pub board: u16,
    pub source_commit: String,
    pub reference_commit: String,
    pub package_manifest_sha256: String,
    pub workflow: WorkflowIdentity,
    pub detector_admitted: bool,
    pub partition_layout: PartitionLayoutObservationEvidence,
    pub ota_session: DeviceSessionEvidence,
    pub mining_state: String,
    pub hardware_control_state: String,
    pub cleanup_complete: bool,
    pub private_modes_valid: bool,
    pub redaction_status: String,
}

impl PartitionLayoutEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != PARTITION_LAYOUT_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("partition layout evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::CapturePartitionLayoutEvidence
        {
            return Err("partition layout workflow identity is invalid");
        }
        for commit in [self.source_commit.as_str(), self.reference_commit.as_str()] {
            if !is_lower_hex(commit, 40) {
                return Err("partition layout source identity is invalid");
            }
        }
        for digest in [
            self.package_manifest_sha256.as_str(),
            self.workflow.request_sha256.as_str(),
            self.partition_layout.partition_table_sha256.as_str(),
            self.partition_layout.ota_image_sha256.as_str(),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("partition layout evidence digest is invalid");
            }
        }
        let layout = &self.partition_layout;
        if layout.required_partition_count != 8
            || !layout.canonical_layout_matches
            || !layout.factory_baseline_observed
            || !layout.ota_0_recovered
            || !layout.ota_upload_complete
            || !layout.ota_boot_validation_complete
        {
            return Err("partition layout transition is incomplete");
        }
        let session = &self.ota_session;
        if session.schema_version != "esp-device-session-v1"
            || session.terminal_category != "ready"
            || session.platform_category != "macos"
            || session.board_category != "205"
            || session.request_attempt_count != 1
            || !matches!(
                session.request_outcome.as_str(),
                "response_received" | "response_missing"
            )
            || !session.same_physical_device
            || !session.stable_enumeration
            || !session.reader_armed
            || !session.pre_restart_serial_delivery
            || !session.post_restart_serial_delivery
            || !session.service_loss_observed
            || !session.trusted_origin_preserved
            || !session.application_recovered
            || !session.build_identity_matches
            || !session.boot_session_changed
            || !session.boot_ordinal_advanced_by_one
            || !session.software_reset_observed
            || !session.postcondition_matches
            || !session.cleanup_complete
        {
            return Err("partition layout OTA transaction is incomplete");
        }
        if !self.detector_admitted
            || self.mining_state != "disabled"
            || self.hardware_control_state != "disabled"
            || !self.cleanup_complete
            || !self.private_modes_valid
            || self.redaction_status != "passed"
        {
            return Err("partition layout safety or privacy evidence is invalid");
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

    fn valid_session() -> DeviceSessionEvidence {
        serde_json::from_value(serde_json::json!({
            "schema_version": "esp-device-session-v1", "terminal_category": "ready",
            "platform_category": "macos", "board_category": "205",
            "same_physical_device": true, "stable_enumeration": true, "reenumerated": true,
            "reader_armed": true, "pre_restart_serial_delivery": true,
            "post_restart_serial_delivery": true, "serial_delivery": "correlated",
            "request_outcome": "response_received", "request_attempt_count": 1,
            "service_loss_observed": true, "trusted_origin_preserved": true,
            "application_recovered": true, "build_identity_matches": true,
            "boot_session_changed": true, "boot_ordinal_advanced_by_one": true,
            "software_reset_observed": true, "postcondition_matches": true,
            "cleanup_complete": true, "usb_disappearance_count": 1,
            "enumeration_change_count": 1, "serial_byte_count": 1,
            "http_observation_count": 2, "duration_millis": 1000
        }))
        .expect("valid session fixture")
    }

    fn valid_evidence() -> PartitionLayoutEvidence {
        PartitionLayoutEvidence {
            schema_version: PARTITION_LAYOUT_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            source_commit: "a".repeat(40),
            reference_commit: "b".repeat(40),
            package_manifest_sha256: "c".repeat(64),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::CapturePartitionLayoutEvidence,
                request_sha256: "d".repeat(64),
            },
            detector_admitted: true,
            partition_layout: PartitionLayoutObservationEvidence {
                partition_table_sha256: "e".repeat(64),
                ota_image_sha256: "f".repeat(64),
                required_partition_count: 8,
                canonical_layout_matches: true,
                factory_baseline_observed: true,
                ota_0_recovered: true,
                ota_upload_complete: true,
                ota_boot_validation_complete: true,
            },
            ota_session: valid_session(),
            mining_state: "disabled".to_owned(),
            hardware_control_state: "disabled".to_owned(),
            cleanup_complete: true,
            private_modes_valid: true,
            redaction_status: "passed".to_owned(),
        }
    }

    #[test]
    fn valid_transition_is_accepted() {
        // Arrange
        let evidence = valid_evidence();

        // Act
        let result = evidence.validate();

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn missing_ota_validation_is_rejected() {
        // Arrange
        let mut evidence = valid_evidence();
        evidence.partition_layout.ota_boot_validation_complete = false;

        // Act
        let result = evidence.validate();

        // Assert
        assert_eq!(result, Err("partition layout transition is incomplete"));
    }
}
