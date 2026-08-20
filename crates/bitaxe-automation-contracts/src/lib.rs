//! Rust-owned contracts for host automation and evidence orchestration.

use schemars::JsonSchema;
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
mod bundle;
mod cfg07_evidence;
mod core_voltage_control_evidence;
mod display_behavior_evidence;
mod emc2101_thermal_evidence;
mod emc2101_thermal_fault_evidence;
mod emc2101_thermal_input;
mod hashrate_monitor_evidence;
mod ina260_evidence;
mod input_uat_evidence;
mod log_buffer_evidence;
mod mining_criteria_evidence;
mod network_reconnect_evidence;
mod network_scan_evidence;
mod operator_snapshot_evidence;
mod partition_layout_evidence;
mod protocol_coordinator_evidence;
mod provisioning_network_evidence;
mod release_recovery_evidence;
mod runtime_health_evidence;
mod safe10_evidence;
mod scoreboard_evidence;
mod screen_flow_evidence;
mod sdkconfig_rollback_evidence;
mod settings_patch_evidence;
mod statistics_history_evidence;
mod stratum_socket_evidence;
mod system_info_evidence;
mod typescript;
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
pub use bundle::{contract_bundle, ContractBundle};
pub use cfg07_evidence::{Cfg07CredentialEvidence, Cfg07Evidence, Cfg07SourceEvidence};
pub use core_voltage_control_evidence::{
    CoreVoltageControlEvidence, CoreVoltageControlObservationEvidence,
    CoreVoltageControlSourceEvidence,
};
pub use display_behavior_evidence::{
    DisplayBehaviorEvidence, DisplayBehaviorObservationEvidence, DisplayBehaviorSourceEvidence,
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
pub use hashrate_monitor_evidence::{
    HashrateMonitorEvidence, HashrateMonitorQuorum, HashrateMonitorSourceEvidence,
    HashrateTransportQuorum, HASHRATE_MONITOR_EVIDENCE_SCHEMA,
};
pub use ina260_evidence::{Ina260Evidence, Ina260ObservationEvidence, Ina260SourceEvidence};
pub use input_uat_evidence::{InputUatEvidence, InputUatObservationEvidence};
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
pub use release_recovery_evidence::{ReleaseRecoveryEvidence, RELEASE_RECOVERY_EVIDENCE_SCHEMA};
pub use runtime_health_evidence::{RuntimeHealthEvidence, RuntimeHealthObservationEvidence};
pub use safe10_evidence::{Safe10Evidence, Safe10PrerequisiteEvidence, Safe10SourceEvidence};
pub use scoreboard_evidence::{
    ScoreboardEvidence, ScoreboardEvidenceDocument, ScoreboardEvidenceV2,
    ScoreboardObservationEvidence, ScoreboardSourceEvidence, ScoreboardSourceEvidenceV2,
    SCOREBOARD_EVIDENCE_SCHEMA, SCOREBOARD_EVIDENCE_V2_SCHEMA,
};
pub use screen_flow_evidence::{
    ScreenFlowEvidence, ScreenFlowObservationEvidence, ScreenFlowSourceEvidence,
};
pub use sdkconfig_rollback_evidence::{
    SdkconfigRollbackEvidence, SdkconfigRollbackObservationEvidence,
};
pub use settings_patch_evidence::{SettingsPatchEvidence, SettingsPatchObservationEvidence};
pub use statistics_history_evidence::*;
pub use stratum_socket_evidence::{
    StratumSocketEvidence, StratumSocketObservationEvidence, StratumSocketSourceEvidence,
};
pub use system_info_evidence::{SystemInfoEvidence, SystemInfoObservationEvidence};
pub use typescript::{
    input_uat_typescript_contracts, typescript_contracts, ui_workflow_typescript_contracts,
};
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
pub const SAFE10_EVIDENCE_SCHEMA: &str = "bitaxe-safe10-evidence-v1";
pub const CFG07_EVIDENCE_SCHEMA: &str = "bitaxe-cfg07-evidence-v1";
pub const ADC_OBSERVATION_EVIDENCE_SCHEMA: &str = "bitaxe-adc-observation-evidence-v1";
pub const ULTRA205_DEFAULTS_EVIDENCE_SCHEMA: &str = "bitaxe-ultra205-defaults-evidence-v1";
pub const SETTINGS_PATCH_EVIDENCE_SCHEMA: &str = "bitaxe-settings-patch-evidence-v1";
pub const STATISTICS_HISTORY_EVIDENCE_SCHEMA: &str = "bitaxe-statistics-history-evidence-v1";
pub const LOG_BUFFER_EVIDENCE_SCHEMA: &str = "bitaxe-log-buffer-evidence-v1";
pub const PARTITION_LAYOUT_EVIDENCE_SCHEMA: &str = "bitaxe-partition-layout-evidence-v1";
pub const SDKCONFIG_ROLLBACK_EVIDENCE_SCHEMA: &str = "bitaxe-sdkconfig-rollback-evidence-v1";
pub const NETWORK_RECONNECT_EVIDENCE_SCHEMA: &str = "bitaxe-network-reconnect-evidence-v1";
pub const NETWORK_SCAN_EVIDENCE_SCHEMA: &str = "bitaxe-network-scan-evidence-v1";
pub const ASIC_INITIALIZATION_EVIDENCE_SCHEMA: &str = "bitaxe-asic-initialization-evidence-v1";
pub const ASIC_POWER_INITIALIZATION_EVIDENCE_SCHEMA: &str =
    "bitaxe-asic-power-initialization-evidence-v1";
pub const CORE_VOLTAGE_CONTROL_EVIDENCE_SCHEMA: &str = "bitaxe-core-voltage-control-evidence-v1";
pub const DISPLAY_BEHAVIOR_EVIDENCE_SCHEMA: &str = "bitaxe-display-behavior-evidence-v1";
pub const SCREEN_FLOW_EVIDENCE_SCHEMA: &str = "bitaxe-screen-flow-evidence-v1";
pub const INA260_EVIDENCE_SCHEMA: &str = "bitaxe-ina260-evidence-v2";
pub const INPUT_UAT_EVIDENCE_SCHEMA: &str = "bitaxe-input-uat-evidence-v1";
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
    CaptureHashrateMonitorEvidence,
    CaptureScoreboardEvidence,
    CaptureUltra205DefaultsEvidence,
    CaptureSettingsPatchEvidence,
    CaptureStatisticsHistoryEvidence,
    CaptureLogBufferEvidence,
    CapturePartitionLayoutEvidence,
    CaptureSdkconfigRollbackEvidence,
    CaptureNetworkReconnectEvidence,
    CaptureNetworkScanEvidence,
    ProjectAsicInitializationEvidence,
    ProjectAsicPowerInitializationEvidence,
    ProjectCoreVoltageControlEvidence,
    ProjectDisplayBehaviorEvidence,
    ProjectScreenFlowEvidence,
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
    ProjectSafe10Evidence,
    ProjectCfg07Evidence,
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

#[cfg(test)]
mod lib_tests;
