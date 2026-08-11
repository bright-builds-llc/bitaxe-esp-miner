//! Rust-owned contracts for host automation and evidence orchestration.

use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod log_buffer_evidence;
mod operator_snapshot_evidence;
mod runtime_health_evidence;
mod settings_patch_evidence;
mod system_info_evidence;

pub use log_buffer_evidence::{LogBufferEvidence, LogBufferObservationEvidence};
pub use operator_snapshot_evidence::{
    DeviceSessionEvidence, OperatorSnapshotEpochEvidence, OperatorSnapshotEvidence,
};
pub use runtime_health_evidence::{RuntimeHealthEvidence, RuntimeHealthObservationEvidence};
pub use settings_patch_evidence::{SettingsPatchEvidence, SettingsPatchObservationEvidence};
pub use system_info_evidence::{SystemInfoEvidence, SystemInfoObservationEvidence};

pub const CONTRACT_BUNDLE_SCHEMA: &str = "bitaxe-command-contract-v1";
pub const RESULT_SCHEMA: &str = "bitaxe-automation-result-v1";
pub const HARDWARE_ATTEMPT_SCHEMA: &str = "bitaxe-hardware-attempt-v1";
pub const CORRELATED_EVIDENCE_SCHEMA: &str = "bitaxe-correlated-runtime-evidence-v1";
pub const SUBSTANTIVE_EVIDENCE_SCHEMA: &str = "bitaxe-substantive-evidence-v1";
pub const VERSION_EVIDENCE_SCHEMA: &str = "bitaxe-version-evidence-v1";
pub const OPERATOR_SNAPSHOT_EVIDENCE_SCHEMA: &str = "bitaxe-operator-snapshot-evidence-v1";
pub const RUNTIME_HEALTH_EVIDENCE_SCHEMA: &str = "bitaxe-runtime-health-evidence-v1";
pub const SYSTEM_INFO_EVIDENCE_SCHEMA: &str = "bitaxe-system-info-evidence-v1";
pub const SETTINGS_PATCH_EVIDENCE_SCHEMA: &str = "bitaxe-settings-patch-evidence-v1";
pub const LOG_BUFFER_EVIDENCE_SCHEMA: &str = "bitaxe-log-buffer-evidence-v1";
pub const MIGRATION_SCHEMA: &str = "bitaxe-automation-migration-v1";

#[must_use]
pub fn typescript_contracts() -> &'static str {
    include_str!("../typescript-contracts.ts")
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AutomationCommand {
    Doctor,
    BootstrapEsp,
    BuildFirmware,
    PackageFirmware,
    VerifyReference,
    VerifyRedaction,
    VerifyProductionSession,
    ObserveSerial,
    VerifyFlashDurability,
    VerifyFirmwareOta,
    VerifyWebAssetsOta,
    VerifyRecovery,
    VerifyHttpApi,
    VerifyHardwareSurface,
    VerifyMining,
    CaptureOperatorEvidence,
    VerifySettingsDurability,
    CaptureCorrelatedRuntimeEvidence,
    CaptureVersionEvidence,
    CaptureOperatorSnapshotEvidence,
    CaptureRuntimeHealthEvidence,
    CaptureSystemInfoEvidence,
    CaptureSettingsPatchEvidence,
    CaptureLogBufferEvidence,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomationStatus {
    Succeeded,
    Failed,
    Blocked,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum AutomationCategory {
    Complete,
    InvalidInvocation,
    ContractMismatch,
    WorkspaceInvalid,
    DependencyUnavailable,
    PolicyBlocked,
    AuthorizationBlocked,
    ProcessFailed,
    Timeout,
    EvidenceInvalid,
    HardwareBlocked,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct AutomationResult {
    pub schema_version: String,
    pub command: AutomationCommand,
    pub status: AutomationStatus,
    pub category: AutomationCategory,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub public: Option<Value>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct WorkflowIdentity {
    pub schema_version: String,
    pub command: AutomationCommand,
    pub request_sha256: String,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct VersionEvidence {
    pub schema_version: String,
    pub board: u16,
    pub source_commit: String,
    pub reference_commit: String,
    pub package_manifest_sha256: String,
    pub workflow: WorkflowIdentity,
    pub boot_observed: bool,
    pub same_origin_api_observed: bool,
    pub mining_state: String,
    pub hardware_control_state: String,
    pub redaction_status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version_projection: Option<VersionProjectionEvidence>,
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct VersionProjectionEvidence {
    pub api_build_label_matches_manifest: bool,
    pub api_static_asset_version_matches_manifest: bool,
    pub api_extended_provenance_matches_manifest: bool,
    pub api_esp_idf_version_matches_manifest: bool,
    pub websocket_same_boot_revision_observed: bool,
    pub websocket_version_projection_matches_api: bool,
}

impl VersionEvidence {
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != VERSION_EVIDENCE_SCHEMA || self.board != 205 {
            return Err("version evidence schema or board is invalid");
        }
        if self.workflow.schema_version != "bitaxe-workflow-identity-v1"
            || self.workflow.command != AutomationCommand::CaptureVersionEvidence
        {
            return Err("version evidence workflow identity is invalid");
        }
        for digest in [
            self.package_manifest_sha256.as_str(),
            self.workflow.request_sha256.as_str(),
        ] {
            if digest.len() != 64
                || !digest
                    .bytes()
                    .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
            {
                return Err("version evidence digest is invalid");
            }
        }
        if !self.boot_observed || !self.same_origin_api_observed {
            return Err("required passive observations are missing");
        }
        if self.mining_state != "disabled"
            || self.hardware_control_state != "disabled"
            || self.redaction_status != "passed"
        {
            return Err("version evidence safe-state or redaction marker is invalid");
        }
        if let Some(projection) = &self.version_projection {
            if !projection.api_build_label_matches_manifest
                || !projection.api_static_asset_version_matches_manifest
                || !projection.api_extended_provenance_matches_manifest
                || !projection.api_esp_idf_version_matches_manifest
                || !projection.websocket_same_boot_revision_observed
                || !projection.websocket_version_projection_matches_api
            {
                return Err("version evidence projection comparison is invalid");
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
pub struct HardwareAttempt {
    pub schema_version: String,
    pub board: u16,
    pub source_commit: String,
    pub reference_commit: String,
    pub workflow: WorkflowIdentity,
    pub status: AutomationStatus,
    pub category: AutomationCategory,
}

#[derive(Clone, Debug, Serialize)]
pub struct ContractBundle {
    pub schema_version: &'static str,
    pub result_schema: Value,
    pub workflow_identity_schema: Value,
    pub hardware_attempt_schema: Value,
    pub version_evidence_schema: Value,
    pub operator_snapshot_evidence_schema: Value,
    pub runtime_health_evidence_schema: Value,
    pub system_info_evidence_schema: Value,
    pub settings_patch_evidence_schema: Value,
    pub log_buffer_evidence_schema: Value,
    pub commands: Vec<AutomationCommand>,
    pub evidence_schemas: Vec<&'static str>,
}

#[must_use]
pub fn contract_bundle() -> ContractBundle {
    ContractBundle {
        schema_version: CONTRACT_BUNDLE_SCHEMA,
        result_schema: serde_json::to_value(schema_for!(AutomationResult))
            .expect("automation result schema must serialize"),
        workflow_identity_schema: serde_json::to_value(schema_for!(WorkflowIdentity))
            .expect("workflow identity schema must serialize"),
        hardware_attempt_schema: serde_json::to_value(schema_for!(HardwareAttempt))
            .expect("hardware attempt schema must serialize"),
        version_evidence_schema: serde_json::to_value(schema_for!(VersionEvidence))
            .expect("version evidence schema must serialize"),
        operator_snapshot_evidence_schema: serde_json::to_value(schema_for!(
            OperatorSnapshotEvidence
        ))
        .expect("operator snapshot evidence schema must serialize"),
        runtime_health_evidence_schema: serde_json::to_value(schema_for!(RuntimeHealthEvidence))
            .expect("runtime health evidence schema must serialize"),
        system_info_evidence_schema: serde_json::to_value(schema_for!(SystemInfoEvidence))
            .expect("system info evidence schema must serialize"),
        settings_patch_evidence_schema: serde_json::to_value(schema_for!(SettingsPatchEvidence))
            .expect("settings PATCH evidence schema must serialize"),
        log_buffer_evidence_schema: serde_json::to_value(schema_for!(LogBufferEvidence))
            .expect("log buffer evidence schema must serialize"),
        commands: vec![
            AutomationCommand::Doctor,
            AutomationCommand::BootstrapEsp,
            AutomationCommand::BuildFirmware,
            AutomationCommand::PackageFirmware,
            AutomationCommand::VerifyReference,
            AutomationCommand::VerifyRedaction,
            AutomationCommand::VerifyProductionSession,
            AutomationCommand::ObserveSerial,
            AutomationCommand::VerifyFlashDurability,
            AutomationCommand::VerifyFirmwareOta,
            AutomationCommand::VerifyWebAssetsOta,
            AutomationCommand::VerifyRecovery,
            AutomationCommand::VerifyHttpApi,
            AutomationCommand::VerifyHardwareSurface,
            AutomationCommand::VerifyMining,
            AutomationCommand::CaptureOperatorEvidence,
            AutomationCommand::VerifySettingsDurability,
            AutomationCommand::CaptureCorrelatedRuntimeEvidence,
            AutomationCommand::CaptureVersionEvidence,
            AutomationCommand::CaptureOperatorSnapshotEvidence,
            AutomationCommand::CaptureRuntimeHealthEvidence,
            AutomationCommand::CaptureSystemInfoEvidence,
            AutomationCommand::CaptureSettingsPatchEvidence,
            AutomationCommand::CaptureLogBufferEvidence,
        ],
        evidence_schemas: vec![
            HARDWARE_ATTEMPT_SCHEMA,
            CORRELATED_EVIDENCE_SCHEMA,
            SUBSTANTIVE_EVIDENCE_SCHEMA,
            VERSION_EVIDENCE_SCHEMA,
            OPERATOR_SNAPSHOT_EVIDENCE_SCHEMA,
            RUNTIME_HEALTH_EVIDENCE_SCHEMA,
            SYSTEM_INFO_EVIDENCE_SCHEMA,
            SETTINGS_PATCH_EVIDENCE_SCHEMA,
            LOG_BUFFER_EVIDENCE_SCHEMA,
            MIGRATION_SCHEMA,
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_bundle_has_semantic_schema_names() {
        // Arrange
        let bundle = contract_bundle();

        // Act
        let encoded = serde_json::to_value(bundle).expect("bundle should serialize");

        // Assert
        assert_eq!(encoded["schema_version"], CONTRACT_BUNDLE_SCHEMA);
        assert_eq!(encoded["commands"][0], "doctor");
        assert!(encoded["evidence_schemas"]
            .as_array()
            .expect("evidence schemas should be an array")
            .iter()
            .all(|schema| !schema.as_str().unwrap_or_default().contains("phase")));
    }

    #[test]
    fn version_evidence_accepts_legacy_base_projection() {
        // Arrange
        let evidence = valid_version_evidence(None);

        // Act
        let result = evidence.validate();

        // Assert
        assert_eq!(result, Ok(()));
    }

    #[test]
    fn version_evidence_rejects_a_failed_live_projection_comparison() {
        // Arrange
        let mut projection = valid_version_projection();
        projection.websocket_version_projection_matches_api = false;
        let evidence = valid_version_evidence(Some(projection));

        // Act
        let result = evidence.validate();

        // Assert
        assert_eq!(
            result,
            Err("version evidence projection comparison is invalid")
        );
    }

    #[test]
    fn operator_snapshot_evidence_requires_two_complete_epochs_and_ready_restart() {
        // Arrange
        let valid = valid_operator_snapshot_evidence();
        let mut invalid = valid.clone();
        invalid.restart_session.request_attempt_count = 2;

        // Act
        let accepted = valid.validate();
        let rejected = invalid.validate();

        // Assert
        assert_eq!(accepted, Ok(()));
        assert_eq!(
            rejected,
            Err("operator snapshot restart transaction is incomplete")
        );
    }

    fn valid_version_evidence(
        version_projection: Option<VersionProjectionEvidence>,
    ) -> VersionEvidence {
        VersionEvidence {
            schema_version: VERSION_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            source_commit: "a".repeat(40),
            reference_commit: "b".repeat(40),
            package_manifest_sha256: "c".repeat(64),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::CaptureVersionEvidence,
                request_sha256: "d".repeat(64),
            },
            boot_observed: true,
            same_origin_api_observed: true,
            mining_state: "disabled".to_owned(),
            hardware_control_state: "disabled".to_owned(),
            redaction_status: "passed".to_owned(),
            version_projection,
        }
    }

    fn valid_version_projection() -> VersionProjectionEvidence {
        VersionProjectionEvidence {
            api_build_label_matches_manifest: true,
            api_static_asset_version_matches_manifest: true,
            api_extended_provenance_matches_manifest: true,
            api_esp_idf_version_matches_manifest: true,
            websocket_same_boot_revision_observed: true,
            websocket_version_projection_matches_api: true,
        }
    }

    fn valid_operator_snapshot_evidence() -> OperatorSnapshotEvidence {
        let epoch = |session: char, projection: char| OperatorSnapshotEpochEvidence {
            boot_session_sha256: session.to_string().repeat(64),
            http_snapshot_observed: true,
            websocket_snapshot_observed: true,
            same_boot_session: true,
            http_revision: 7,
            websocket_revision: 8,
            websocket_revision_not_earlier: true,
            retained_log_marker_matches_http: true,
            retained_log_marker_matches_websocket: true,
            substantive_fields_present: true,
            stable_fields_match: true,
            safe_operator_state_confirmed: true,
            substantive_projection_sha256: projection.to_string().repeat(64),
        };
        OperatorSnapshotEvidence {
            schema_version: OPERATOR_SNAPSHOT_EVIDENCE_SCHEMA.to_owned(),
            board: 205,
            source_commit: "a".repeat(40),
            reference_commit: "b".repeat(40),
            package_manifest_sha256: "c".repeat(64),
            workflow: WorkflowIdentity {
                schema_version: "bitaxe-workflow-identity-v1".to_owned(),
                command: AutomationCommand::CaptureOperatorSnapshotEvidence,
                request_sha256: "d".repeat(64),
            },
            baseline_epoch: epoch('1', 'e'),
            post_restart_epoch: epoch('2', 'f'),
            distinct_boot_sessions: true,
            restart_session: DeviceSessionEvidence {
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
                serial_byte_count: 128,
                http_observation_count: 3,
                duration_millis: 1_000,
            },
            mining_state: "disabled".to_owned(),
            hardware_control_state: "disabled".to_owned(),
            cleanup_complete: true,
            redaction_status: "passed".to_owned(),
        }
    }
}
