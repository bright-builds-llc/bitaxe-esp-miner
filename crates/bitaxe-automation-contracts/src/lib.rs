//! Rust-owned contracts for host automation and evidence orchestration.

use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub const CONTRACT_BUNDLE_SCHEMA: &str = "bitaxe-command-contract-v1";
pub const RESULT_SCHEMA: &str = "bitaxe-automation-result-v1";
pub const HARDWARE_ATTEMPT_SCHEMA: &str = "bitaxe-hardware-attempt-v1";
pub const CORRELATED_EVIDENCE_SCHEMA: &str = "bitaxe-correlated-runtime-evidence-v1";
pub const SUBSTANTIVE_EVIDENCE_SCHEMA: &str = "bitaxe-substantive-evidence-v1";
pub const VERSION_EVIDENCE_SCHEMA: &str = "bitaxe-version-evidence-v1";
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
        ],
        evidence_schemas: vec![
            HARDWARE_ATTEMPT_SCHEMA,
            CORRELATED_EVIDENCE_SCHEMA,
            SUBSTANTIVE_EVIDENCE_SCHEMA,
            VERSION_EVIDENCE_SCHEMA,
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
}
