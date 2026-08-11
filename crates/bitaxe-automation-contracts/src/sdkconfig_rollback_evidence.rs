use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    AutomationCommand, DeviceSessionEvidence, WorkflowIdentity, SDKCONFIG_ROLLBACK_EVIDENCE_SCHEMA,
};

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct SdkconfigRollbackObservationEvidence {
    pub sdkconfig_sha256: String,
    pub rollback_enabled: bool,
    pub anti_rollback_disabled: bool,
    pub rollback_probe_isolated: bool,
    pub interrupted_upload_attempt_count: u64,
    pub interrupted_upload_prefix_bytes: u64,
    pub interruption_protocol_abort_observed: bool,
    pub baseline_boot_session_unchanged: bool,
    pub baseline_boot_ordinal_unchanged: bool,
    pub baseline_build_unchanged: bool,
    pub probe_pending_validation_observed: bool,
    pub probe_running_partition_ota_0: bool,
    pub rollback_running_partition_factory: bool,
    pub final_normal_build_restored: bool,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct SdkconfigRollbackEvidence {
    pub schema_version: String,
    pub board: u16,
    pub source_commit: String,
    pub reference_commit: String,
    pub package_manifest_sha256: String,
    pub rollback_probe_image_sha256: String,
    pub rollback_probe_metadata_sha256: String,
    pub workflow: WorkflowIdentity,
    pub detector_admitted: bool,
    pub rollback: SdkconfigRollbackObservationEvidence,
    pub probe_boot_session: DeviceSessionEvidence,
    pub rollback_session: DeviceSessionEvidence,
    pub mining_state: String,
    pub hardware_control_state: String,
    pub cleanup_complete: bool,
    pub normal_package_restored: bool,
    pub recovery_flash_used: bool,
    pub private_modes_valid: bool,
    pub redaction_status: String,
}

impl SdkconfigRollbackEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != SDKCONFIG_ROLLBACK_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("SDK config rollback evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::CaptureSdkconfigRollbackEvidence
        {
            return Err("SDK config rollback workflow identity is invalid");
        }
        for commit in [self.source_commit.as_str(), self.reference_commit.as_str()] {
            if !is_lower_hex(commit, 40) {
                return Err("SDK config rollback source identity is invalid");
            }
        }
        for digest in [
            self.package_manifest_sha256.as_str(),
            self.rollback_probe_image_sha256.as_str(),
            self.rollback_probe_metadata_sha256.as_str(),
            self.workflow.request_sha256.as_str(),
            self.rollback.sdkconfig_sha256.as_str(),
        ] {
            if !is_lower_hex(digest, 64) {
                return Err("SDK config rollback digest is invalid");
            }
        }
        let rollback = &self.rollback;
        if !rollback.rollback_enabled
            || !rollback.anti_rollback_disabled
            || !rollback.rollback_probe_isolated
            || rollback.interrupted_upload_attempt_count != 1
            || rollback.interrupted_upload_prefix_bytes == 0
            || !rollback.interruption_protocol_abort_observed
            || !rollback.baseline_boot_session_unchanged
            || !rollback.baseline_boot_ordinal_unchanged
            || !rollback.baseline_build_unchanged
            || !rollback.probe_pending_validation_observed
            || !rollback.probe_running_partition_ota_0
            || !rollback.rollback_running_partition_factory
            || !rollback.final_normal_build_restored
        {
            return Err("SDK config interruption or rollback observation is incomplete");
        }
        for session in [&self.probe_boot_session, &self.rollback_session] {
            validate_ready_session(session)?;
        }
        if !self.detector_admitted
            || self.mining_state != "disabled"
            || self.hardware_control_state != "disabled"
            || !self.cleanup_complete
            || !self.normal_package_restored
            || self.recovery_flash_used
            || !self.private_modes_valid
            || self.redaction_status != "passed"
        {
            return Err(
                "SDK config rollback safety, restoration, or redaction evidence is invalid",
            );
        }
        Ok(())
    }
}

fn validate_ready_session(session: &DeviceSessionEvidence) -> Result<(), &'static str> {
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
        return Err("SDK config rollback device session is incomplete");
    }
    Ok(())
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
    use crate::AutomationCommand;

    fn digest(character: char, length: usize) -> String {
        std::iter::repeat_n(character, length).collect()
    }

    fn session() -> DeviceSessionEvidence {
        DeviceSessionEvidence {
            schema_version: "esp-device-session-v1".to_owned(),
            terminal_category: "ready".to_owned(),
            platform_category: "macos".to_owned(),
            board_category: "205".to_owned(),
            same_physical_device: true,
            stable_enumeration: true,
            reenumerated: false,
            reader_armed: true,
            pre_restart_serial_delivery: true,
            post_restart_serial_delivery: true,
            serial_delivery: "correlated".to_owned(),
            request_outcome: "response_received".to_owned(),
            request_attempt_count: 1,
            service_loss_observed: true,
            trusted_origin_preserved: true,
            application_recovered: true,
            build_identity_matches: true,
            boot_session_changed: true,
            boot_ordinal_advanced_by_one: true,
            software_reset_observed: true,
            postcondition_matches: true,
            cleanup_complete: true,
            usb_disappearance_count: 0,
            enumeration_change_count: 0,
            serial_byte_count: 100,
            http_observation_count: 2,
            duration_millis: 1_000,
        }
    }

    fn evidence() -> SdkconfigRollbackEvidence {
        SdkconfigRollbackEvidence {
            schema_version: SDKCONFIG_ROLLBACK_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            source_commit: digest('a', 40),
            reference_commit: digest('b', 40),
            package_manifest_sha256: digest('c', 64),
            rollback_probe_image_sha256: digest('d', 64),
            rollback_probe_metadata_sha256: digest('e', 64),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::CaptureSdkconfigRollbackEvidence,
                request_sha256: digest('f', 64),
            },
            detector_admitted: true,
            rollback: SdkconfigRollbackObservationEvidence {
                sdkconfig_sha256: digest('1', 64),
                rollback_enabled: true,
                anti_rollback_disabled: true,
                rollback_probe_isolated: true,
                interrupted_upload_attempt_count: 1,
                interrupted_upload_prefix_bytes: 4_096,
                interruption_protocol_abort_observed: true,
                baseline_boot_session_unchanged: true,
                baseline_boot_ordinal_unchanged: true,
                baseline_build_unchanged: true,
                probe_pending_validation_observed: true,
                probe_running_partition_ota_0: true,
                rollback_running_partition_factory: true,
                final_normal_build_restored: true,
            },
            probe_boot_session: session(),
            rollback_session: session(),
            mining_state: "disabled".to_owned(),
            hardware_control_state: "disabled".to_owned(),
            cleanup_complete: true,
            normal_package_restored: true,
            recovery_flash_used: false,
            private_modes_valid: true,
            redaction_status: "passed".to_owned(),
        }
    }

    #[test]
    fn complete_interruption_and_rollback_evidence_validates() {
        // Arrange / Act / Assert
        assert_eq!(evidence().validate(), Ok(()));
    }

    #[test]
    fn recovery_flash_cannot_convert_a_failed_transaction_into_evidence() {
        // Arrange
        let mut candidate = evidence();
        candidate.recovery_flash_used = true;

        // Act / Assert
        assert!(candidate.validate().is_err());
    }
}
