//! Rust-owned contracts for host automation and evidence orchestration.

use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_json::Value;

mod adc_observation_evidence;
mod adc_observation_input;
mod asic_frequency_transition_evidence;
mod asic_initialization_evidence;
mod asic_power_initialization_evidence;
mod asic_reset_evidence;
mod asic_result_parsing_evidence;
mod asic_serial_transport_evidence;
mod asic_work_send_evidence;
mod core_voltage_control_evidence;
mod emc2101_thermal_evidence;
mod emc2101_thermal_fault_evidence;
mod emc2101_thermal_input;
mod ina260_evidence;
mod log_buffer_evidence;
mod mining_criteria_evidence;
mod network_reconnect_evidence;
mod network_scan_evidence;
mod operator_snapshot_evidence;
mod partition_layout_evidence;
mod protocol_coordinator_evidence;
mod provisioning_network_evidence;
mod runtime_health_evidence;
mod sdkconfig_rollback_evidence;
mod settings_patch_evidence;
mod stratum_socket_evidence;
mod system_info_evidence;
mod ui_workflow_evidence;
mod ultra205_defaults_evidence;
pub use adc_observation_evidence::{
    AdcObservationEvidence, AdcObservationQuorum, AdcObservationSourceEvidence,
};
pub use adc_observation_input::{
    validate_adc_observation_inputs, AdcObservationSnapshotInput, AdcObservationWebSocketInput,
};
pub use asic_frequency_transition_evidence::{
    AsicFrequencyTransitionEvidence, AsicFrequencyTransitionObservationEvidence,
    AsicFrequencyTransitionSourceEvidence,
};
pub use asic_initialization_evidence::{
    AsicInitializationAttemptEvidence, AsicInitializationEvidence,
    AsicInitializationObservationEvidence,
};
pub use asic_power_initialization_evidence::{
    AsicPowerInitializationEvidence, AsicPowerInitializationObservationEvidence,
    AsicPowerInitializationSourceEvidence,
};
pub use asic_reset_evidence::{
    AsicResetEvidence, AsicResetObservationEvidence, AsicResetSourceEvidence,
};
pub use asic_result_parsing_evidence::{
    AsicResultParsingEvidence, AsicResultParsingObservationEvidence,
    AsicResultParsingSourceEvidence,
};
pub use asic_serial_transport_evidence::{
    AsicSerialTransportEvidence, AsicSerialTransportObservationEvidence,
    AsicSerialTransportSourceEvidence,
};
pub use asic_work_send_evidence::{
    AsicWorkSendEvidence, AsicWorkSendObservationEvidence, AsicWorkSendSourceEvidence,
};
pub use core_voltage_control_evidence::{
    CoreVoltageControlEvidence, CoreVoltageControlObservationEvidence,
    CoreVoltageControlSourceEvidence,
};
pub use emc2101_thermal_evidence::{
    Emc2101ThermalEvidence, Emc2101ThermalObservationEvidence, Emc2101ThermalSourceEvidence,
};
pub use emc2101_thermal_fault_evidence::{
    Emc2101ThermalFaultEvidence, Emc2101ThermalFaultRestorationEvidence,
    Emc2101ThermalFaultSourceEvidence, Emc2101ThermalFaultStimulusEvidence,
};
pub use emc2101_thermal_input::{
    validate_emc2101_thermal_inputs, Emc2101ThermalSnapshotInput, Emc2101ThermalWebSocketInput,
};
pub use ina260_evidence::{Ina260Evidence, Ina260ObservationEvidence, Ina260SourceEvidence};
pub use log_buffer_evidence::{LogBufferEvidence, LogBufferObservationEvidence};
pub use mining_criteria_evidence::MiningCriteriaEvidence;
pub use network_reconnect_evidence::{
    NetworkReconnectEvidence, NetworkReconnectObservationEvidence,
};
pub use network_scan_evidence::{NetworkScanEvidence, NetworkScanObservationEvidence};
pub use operator_snapshot_evidence::{
    DeviceSessionEvidence, OperatorSnapshotEpochEvidence, OperatorSnapshotEvidence,
};
pub use partition_layout_evidence::{PartitionLayoutEvidence, PartitionLayoutObservationEvidence};
pub use protocol_coordinator_evidence::{
    ProtocolCoordinatorEvidence, ProtocolCoordinatorObservationEvidence,
    ProtocolCoordinatorSourceEvidence,
};
pub use provisioning_network_evidence::{
    ProvisioningNetworkEvidence, ProvisioningNetworkObservationEvidence,
};
pub use runtime_health_evidence::{RuntimeHealthEvidence, RuntimeHealthObservationEvidence};
pub use sdkconfig_rollback_evidence::{
    SdkconfigRollbackEvidence, SdkconfigRollbackObservationEvidence,
};
pub use settings_patch_evidence::{SettingsPatchEvidence, SettingsPatchObservationEvidence};
pub use stratum_socket_evidence::{
    StratumSocketEvidence, StratumSocketObservationEvidence, StratumSocketSourceEvidence,
};
pub use system_info_evidence::{SystemInfoEvidence, SystemInfoObservationEvidence};
pub use ui_workflow_evidence::{
    UiWorkflowBrowserEvidence, UiWorkflowEvidence, UiWorkflowSourceEvidence,
};
pub use ultra205_defaults_evidence::{
    Ultra205DefaultsEvidence, Ultra205DefaultsObservationEvidence,
};
pub const CONTRACT_BUNDLE_SCHEMA: &str = "bitaxe-command-contract-v1";
pub const RESULT_SCHEMA: &str = "bitaxe-automation-result-v1";
pub const HARDWARE_ATTEMPT_SCHEMA: &str = "bitaxe-hardware-attempt-v1";
pub const CORRELATED_EVIDENCE_SCHEMA: &str = "bitaxe-correlated-runtime-evidence-v1";
pub const SUBSTANTIVE_EVIDENCE_SCHEMA: &str = "bitaxe-substantive-evidence-v1";
pub const VERSION_EVIDENCE_SCHEMA: &str = "bitaxe-version-evidence-v1";
pub const OPERATOR_SNAPSHOT_EVIDENCE_SCHEMA: &str = "bitaxe-operator-snapshot-evidence-v1";
pub const RUNTIME_HEALTH_EVIDENCE_SCHEMA: &str = "bitaxe-runtime-health-evidence-v1";
pub const SYSTEM_INFO_EVIDENCE_SCHEMA: &str = "bitaxe-system-info-evidence-v1";
pub const ADC_OBSERVATION_EVIDENCE_SCHEMA: &str = "bitaxe-adc-observation-evidence-v1";
pub const ULTRA205_DEFAULTS_EVIDENCE_SCHEMA: &str = "bitaxe-ultra205-defaults-evidence-v1";
pub const SETTINGS_PATCH_EVIDENCE_SCHEMA: &str = "bitaxe-settings-patch-evidence-v1";
pub const LOG_BUFFER_EVIDENCE_SCHEMA: &str = "bitaxe-log-buffer-evidence-v1";
pub const PARTITION_LAYOUT_EVIDENCE_SCHEMA: &str = "bitaxe-partition-layout-evidence-v1";
pub const SDKCONFIG_ROLLBACK_EVIDENCE_SCHEMA: &str = "bitaxe-sdkconfig-rollback-evidence-v1";
pub const NETWORK_RECONNECT_EVIDENCE_SCHEMA: &str = "bitaxe-network-reconnect-evidence-v1";
pub const NETWORK_SCAN_EVIDENCE_SCHEMA: &str = "bitaxe-network-scan-evidence-v1";
pub const ASIC_INITIALIZATION_EVIDENCE_SCHEMA: &str = "bitaxe-asic-initialization-evidence-v1";
pub const ASIC_POWER_INITIALIZATION_EVIDENCE_SCHEMA: &str =
    "bitaxe-asic-power-initialization-evidence-v1";
pub const CORE_VOLTAGE_CONTROL_EVIDENCE_SCHEMA: &str = "bitaxe-core-voltage-control-evidence-v1";
pub const INA260_EVIDENCE_SCHEMA: &str = "bitaxe-ina260-evidence-v1";
pub const EMC2101_THERMAL_EVIDENCE_SCHEMA: &str = "bitaxe-emc2101-thermal-evidence-v1";
pub const EMC2101_THERMAL_FAULT_EVIDENCE_SCHEMA: &str = "bitaxe-emc2101-thermal-fault-evidence-v1";
pub const ASIC_RESET_EVIDENCE_SCHEMA: &str = "bitaxe-asic-reset-evidence-v1";
pub const ASIC_FREQUENCY_TRANSITION_EVIDENCE_SCHEMA: &str =
    "bitaxe-asic-frequency-transition-evidence-v1";
pub const ASIC_RESULT_PARSING_EVIDENCE_SCHEMA: &str = "bitaxe-asic-result-parsing-evidence-v1";
pub const ASIC_SERIAL_TRANSPORT_EVIDENCE_SCHEMA: &str = "bitaxe-asic-serial-transport-evidence-v1";
pub const ASIC_WORK_SEND_EVIDENCE_SCHEMA: &str = "bitaxe-asic-work-send-evidence-v1";
pub const STRATUM_SOCKET_EVIDENCE_SCHEMA: &str = "bitaxe-stratum-socket-evidence-v1";
pub const PROTOCOL_COORDINATOR_EVIDENCE_SCHEMA: &str = "bitaxe-protocol-coordinator-evidence-v1";
pub const MINING_CRITERIA_EVIDENCE_SCHEMA: &str = "bitaxe-mining-criteria-evidence-v1";
pub const PROVISIONING_NETWORK_EVIDENCE_SCHEMA: &str = "bitaxe-provisioning-network-evidence-v1";
pub const UI_WORKFLOW_EVIDENCE_SCHEMA: &str = "bitaxe-ui-workflow-evidence-v1";
pub const MIGRATION_SCHEMA: &str = "bitaxe-automation-migration-v1";
#[must_use]
pub fn typescript_contracts() -> &'static str {
    include_str!("../typescript-contracts.ts")
}

#[must_use]
pub fn ui_workflow_typescript_contracts() -> &'static str {
    include_str!("../ui-workflow-typescript-contracts.ts")
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AutomationCommand {
    Doctor,
    BootstrapEsp,
    BuildFirmware,
    PackageFirmware,
    PackageRollbackProbe,
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
    ApiCommandEffectsCampaign,
    CaptureCorrelatedRuntimeEvidence,
    CaptureVersionEvidence,
    CaptureOperatorSnapshotEvidence,
    CaptureRuntimeHealthEvidence,
    CaptureSystemInfoEvidence,
    CaptureAdcObservationEvidence,
    CaptureUltra205DefaultsEvidence,
    CaptureSettingsPatchEvidence,
    CaptureLogBufferEvidence,
    CapturePartitionLayoutEvidence,
    CaptureSdkconfigRollbackEvidence,
    CaptureNetworkReconnectEvidence,
    CaptureNetworkScanEvidence,
    ProjectAsicInitializationEvidence,
    ProjectAsicPowerInitializationEvidence,
    ProjectCoreVoltageControlEvidence,
    ProjectIna260Evidence,
    CaptureEmc2101ThermalEvidence,
    CaptureEmc2101ThermalFaultEvidence,
    ProjectAsicResetEvidence,
    ProjectAsicFrequencyTransitionEvidence,
    ProjectAsicWorkSendEvidence,
    ProjectAsicResultParsingEvidence,
    ProjectAsicSerialTransportEvidence,
    ProjectStratumSocketEvidence,
    ProjectProtocolCoordinatorEvidence,
    ProjectMiningCriteriaEvidence,
    CaptureProvisioningNetworkEvidence,
    ProjectUiWorkflowEvidence,
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
    PackageInvalid,
    InterruptionNotObserved,
    ProbeBootFailed,
    RollbackNotObserved,
    RecoveryFailed,
    ReconnectNotObserved,
    ReconnectTimingInvalid,
    ServiceRecoveryFailed,
    BrowserBlocked,
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
    pub adc_observation_evidence_schema: Value,
    pub ultra205_defaults_evidence_schema: Value,
    pub settings_patch_evidence_schema: Value,
    pub log_buffer_evidence_schema: Value,
    pub partition_layout_evidence_schema: Value,
    pub sdkconfig_rollback_evidence_schema: Value,
    pub network_reconnect_evidence_schema: Value,
    pub network_scan_evidence_schema: Value,
    pub asic_initialization_evidence_schema: Value,
    pub asic_power_initialization_evidence_schema: Value,
    pub core_voltage_control_evidence_schema: Value,
    pub ina260_evidence_schema: Value,
    pub emc2101_thermal_evidence_schema: Value,
    pub emc2101_thermal_fault_evidence_schema: Value,
    pub asic_reset_evidence_schema: Value,
    pub asic_frequency_transition_evidence_schema: Value,
    pub asic_work_send_evidence_schema: Value,
    pub asic_result_parsing_evidence_schema: Value,
    pub asic_serial_transport_evidence_schema: Value,
    pub stratum_socket_evidence_schema: Value,
    pub protocol_coordinator_evidence_schema: Value,
    pub mining_criteria_evidence_schema: Value,
    pub provisioning_network_evidence_schema: Value,
    pub ui_workflow_evidence_schema: Value,
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
        adc_observation_evidence_schema: serde_json::to_value(schema_for!(AdcObservationEvidence))
            .expect("ADC observation evidence schema must serialize"),
        ultra205_defaults_evidence_schema: serde_json::to_value(schema_for!(
            Ultra205DefaultsEvidence
        ))
        .expect("Ultra 205 defaults evidence schema must serialize"),
        settings_patch_evidence_schema: serde_json::to_value(schema_for!(SettingsPatchEvidence))
            .expect("settings PATCH evidence schema must serialize"),
        log_buffer_evidence_schema: serde_json::to_value(schema_for!(LogBufferEvidence))
            .expect("log buffer evidence schema must serialize"),
        partition_layout_evidence_schema: serde_json::to_value(schema_for!(
            PartitionLayoutEvidence
        ))
        .expect("partition layout evidence schema must serialize"),
        sdkconfig_rollback_evidence_schema: serde_json::to_value(schema_for!(
            SdkconfigRollbackEvidence
        ))
        .expect("SDK config rollback evidence schema must serialize"),
        network_reconnect_evidence_schema: serde_json::to_value(schema_for!(
            NetworkReconnectEvidence
        ))
        .expect("network reconnect evidence schema must serialize"),
        network_scan_evidence_schema: serde_json::to_value(schema_for!(NetworkScanEvidence))
            .expect("network scan evidence schema must serialize"),
        asic_initialization_evidence_schema: serde_json::to_value(schema_for!(
            AsicInitializationEvidence
        ))
        .expect("ASIC initialization evidence schema must serialize"),
        asic_power_initialization_evidence_schema: serde_json::to_value(schema_for!(
            AsicPowerInitializationEvidence
        ))
        .expect("ASIC power initialization evidence schema must serialize"),
        core_voltage_control_evidence_schema: serde_json::to_value(schema_for!(
            CoreVoltageControlEvidence
        ))
        .expect("core-voltage-control evidence schema must serialize"),
        ina260_evidence_schema: serde_json::to_value(schema_for!(Ina260Evidence))
            .expect("INA260 evidence schema must serialize"),
        emc2101_thermal_evidence_schema: serde_json::to_value(schema_for!(Emc2101ThermalEvidence))
            .expect("EMC2101 thermal evidence schema must serialize"),
        emc2101_thermal_fault_evidence_schema: serde_json::to_value(schema_for!(
            Emc2101ThermalFaultEvidence
        ))
        .expect("EMC2101 thermal fault evidence schema must serialize"),
        asic_reset_evidence_schema: serde_json::to_value(schema_for!(AsicResetEvidence))
            .expect("ASIC reset evidence schema must serialize"),
        asic_frequency_transition_evidence_schema: serde_json::to_value(schema_for!(
            AsicFrequencyTransitionEvidence
        ))
        .expect("ASIC frequency-transition evidence schema must serialize"),
        asic_work_send_evidence_schema: serde_json::to_value(schema_for!(AsicWorkSendEvidence))
            .expect("ASIC work-send evidence schema must serialize"),
        asic_result_parsing_evidence_schema: serde_json::to_value(schema_for!(
            AsicResultParsingEvidence
        ))
        .expect("ASIC result-parsing evidence schema must serialize"),
        asic_serial_transport_evidence_schema: serde_json::to_value(schema_for!(
            AsicSerialTransportEvidence
        ))
        .expect("ASIC serial-transport evidence schema must serialize"),
        stratum_socket_evidence_schema: serde_json::to_value(schema_for!(StratumSocketEvidence))
            .expect("Stratum socket evidence schema must serialize"),
        protocol_coordinator_evidence_schema: serde_json::to_value(schema_for!(
            ProtocolCoordinatorEvidence
        ))
        .expect("protocol coordinator evidence schema must serialize"),
        mining_criteria_evidence_schema: serde_json::to_value(schema_for!(MiningCriteriaEvidence))
            .expect("mining criteria evidence schema must serialize"),
        provisioning_network_evidence_schema: serde_json::to_value(schema_for!(
            ProvisioningNetworkEvidence
        ))
        .expect("provisioning network evidence schema must serialize"),
        ui_workflow_evidence_schema: serde_json::to_value(schema_for!(UiWorkflowEvidence))
            .expect("UI workflow evidence schema must serialize"),
        commands: vec![
            AutomationCommand::Doctor,
            AutomationCommand::BootstrapEsp,
            AutomationCommand::BuildFirmware,
            AutomationCommand::PackageFirmware,
            AutomationCommand::PackageRollbackProbe,
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
            AutomationCommand::ApiCommandEffectsCampaign,
            AutomationCommand::CaptureCorrelatedRuntimeEvidence,
            AutomationCommand::CaptureVersionEvidence,
            AutomationCommand::CaptureOperatorSnapshotEvidence,
            AutomationCommand::CaptureRuntimeHealthEvidence,
            AutomationCommand::CaptureSystemInfoEvidence,
            AutomationCommand::CaptureAdcObservationEvidence,
            AutomationCommand::CaptureUltra205DefaultsEvidence,
            AutomationCommand::CaptureSettingsPatchEvidence,
            AutomationCommand::CaptureLogBufferEvidence,
            AutomationCommand::CapturePartitionLayoutEvidence,
            AutomationCommand::CaptureSdkconfigRollbackEvidence,
            AutomationCommand::CaptureNetworkReconnectEvidence,
            AutomationCommand::CaptureNetworkScanEvidence,
            AutomationCommand::ProjectAsicInitializationEvidence,
            AutomationCommand::ProjectAsicPowerInitializationEvidence,
            AutomationCommand::ProjectCoreVoltageControlEvidence,
            AutomationCommand::ProjectIna260Evidence,
            AutomationCommand::CaptureEmc2101ThermalEvidence,
            AutomationCommand::CaptureEmc2101ThermalFaultEvidence,
            AutomationCommand::ProjectAsicResetEvidence,
            AutomationCommand::ProjectAsicFrequencyTransitionEvidence,
            AutomationCommand::ProjectAsicWorkSendEvidence,
            AutomationCommand::ProjectAsicResultParsingEvidence,
            AutomationCommand::ProjectAsicSerialTransportEvidence,
            AutomationCommand::ProjectStratumSocketEvidence,
            AutomationCommand::ProjectProtocolCoordinatorEvidence,
            AutomationCommand::ProjectMiningCriteriaEvidence,
            AutomationCommand::CaptureProvisioningNetworkEvidence,
            AutomationCommand::ProjectUiWorkflowEvidence,
        ],
        evidence_schemas: vec![
            HARDWARE_ATTEMPT_SCHEMA,
            CORRELATED_EVIDENCE_SCHEMA,
            SUBSTANTIVE_EVIDENCE_SCHEMA,
            VERSION_EVIDENCE_SCHEMA,
            OPERATOR_SNAPSHOT_EVIDENCE_SCHEMA,
            RUNTIME_HEALTH_EVIDENCE_SCHEMA,
            SYSTEM_INFO_EVIDENCE_SCHEMA,
            ADC_OBSERVATION_EVIDENCE_SCHEMA,
            ULTRA205_DEFAULTS_EVIDENCE_SCHEMA,
            SETTINGS_PATCH_EVIDENCE_SCHEMA,
            LOG_BUFFER_EVIDENCE_SCHEMA,
            PARTITION_LAYOUT_EVIDENCE_SCHEMA,
            SDKCONFIG_ROLLBACK_EVIDENCE_SCHEMA,
            NETWORK_RECONNECT_EVIDENCE_SCHEMA,
            NETWORK_SCAN_EVIDENCE_SCHEMA,
            ASIC_INITIALIZATION_EVIDENCE_SCHEMA,
            ASIC_POWER_INITIALIZATION_EVIDENCE_SCHEMA,
            CORE_VOLTAGE_CONTROL_EVIDENCE_SCHEMA,
            INA260_EVIDENCE_SCHEMA,
            EMC2101_THERMAL_EVIDENCE_SCHEMA,
            EMC2101_THERMAL_FAULT_EVIDENCE_SCHEMA,
            ASIC_RESET_EVIDENCE_SCHEMA,
            ASIC_FREQUENCY_TRANSITION_EVIDENCE_SCHEMA,
            ASIC_WORK_SEND_EVIDENCE_SCHEMA,
            ASIC_RESULT_PARSING_EVIDENCE_SCHEMA,
            ASIC_SERIAL_TRANSPORT_EVIDENCE_SCHEMA,
            STRATUM_SOCKET_EVIDENCE_SCHEMA,
            PROTOCOL_COORDINATOR_EVIDENCE_SCHEMA,
            MINING_CRITERIA_EVIDENCE_SCHEMA,
            PROVISIONING_NETWORK_EVIDENCE_SCHEMA,
            UI_WORKFLOW_EVIDENCE_SCHEMA,
            MIGRATION_SCHEMA,
        ],
    }
}

#[cfg(test)]
mod lib_tests;
