use schemars::schema_for;
use serde::Serialize;
use serde_json::Value;

use super::*;

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
    pub hashrate_monitor_evidence_schema: Value,
    pub scoreboard_evidence_schema: Value,
    pub ultra205_defaults_evidence_schema: Value,
    pub settings_patch_evidence_schema: Value,
    pub statistics_history_evidence_schema: Value,
    pub log_buffer_evidence_schema: Value,
    pub partition_layout_evidence_schema: Value,
    pub sdkconfig_rollback_evidence_schema: Value,
    pub network_reconnect_evidence_schema: Value,
    pub network_scan_evidence_schema: Value,
    pub asic_initialization_evidence_schema: Value,
    pub asic_power_initialization_evidence_schema: Value,
    pub core_voltage_control_evidence_schema: Value,
    pub display_behavior_evidence_schema: Value,
    pub screen_flow_evidence_schema: Value,
    pub ina260_evidence_schema: Value,
    pub input_uat_evidence_schema: Value,
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
    pub release_recovery_evidence_schema: Value,
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
        hashrate_monitor_evidence_schema: serde_json::to_value(schema_for!(
            HashrateMonitorEvidence
        ))
        .expect("hashrate monitor evidence schema must serialize"),
        scoreboard_evidence_schema: serde_json::to_value(schema_for!(ScoreboardEvidence))
            .expect("scoreboard evidence schema must serialize"),
        ultra205_defaults_evidence_schema: serde_json::to_value(schema_for!(
            Ultra205DefaultsEvidence
        ))
        .expect("Ultra 205 defaults evidence schema must serialize"),
        settings_patch_evidence_schema: serde_json::to_value(schema_for!(SettingsPatchEvidence))
            .expect("settings PATCH evidence schema must serialize"),
        statistics_history_evidence_schema: serde_json::to_value(schema_for!(
            StatisticsHistoryEvidence
        ))
        .expect("statistics history evidence schema must serialize"),
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
        display_behavior_evidence_schema: serde_json::to_value(schema_for!(
            DisplayBehaviorEvidence
        ))
        .expect("display-behavior evidence schema must serialize"),
        screen_flow_evidence_schema: serde_json::to_value(schema_for!(ScreenFlowEvidence))
            .expect("screen-flow evidence schema must serialize"),
        ina260_evidence_schema: serde_json::to_value(schema_for!(Ina260Evidence))
            .expect("INA260 evidence schema must serialize"),
        input_uat_evidence_schema: serde_json::to_value(schema_for!(InputUatEvidence))
            .expect("input UAT evidence schema must serialize"),
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
        release_recovery_evidence_schema: serde_json::to_value(schema_for!(
            ReleaseRecoveryEvidence
        ))
        .expect("release recovery evidence schema must serialize"),
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
            AutomationCommand::CaptureHashrateMonitorEvidence,
            AutomationCommand::CaptureScoreboardEvidence,
            AutomationCommand::CaptureUltra205DefaultsEvidence,
            AutomationCommand::CaptureSettingsPatchEvidence,
            AutomationCommand::CaptureStatisticsHistoryEvidence,
            AutomationCommand::CaptureLogBufferEvidence,
            AutomationCommand::CapturePartitionLayoutEvidence,
            AutomationCommand::CaptureSdkconfigRollbackEvidence,
            AutomationCommand::CaptureNetworkReconnectEvidence,
            AutomationCommand::CaptureNetworkScanEvidence,
            AutomationCommand::ProjectAsicInitializationEvidence,
            AutomationCommand::ProjectAsicPowerInitializationEvidence,
            AutomationCommand::ProjectCoreVoltageControlEvidence,
            AutomationCommand::ProjectDisplayBehaviorEvidence,
            AutomationCommand::ProjectScreenFlowEvidence,
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
            HASHRATE_MONITOR_EVIDENCE_SCHEMA,
            SCOREBOARD_EVIDENCE_SCHEMA,
            ULTRA205_DEFAULTS_EVIDENCE_SCHEMA,
            SETTINGS_PATCH_EVIDENCE_SCHEMA,
            STATISTICS_HISTORY_EVIDENCE_SCHEMA,
            LOG_BUFFER_EVIDENCE_SCHEMA,
            PARTITION_LAYOUT_EVIDENCE_SCHEMA,
            SDKCONFIG_ROLLBACK_EVIDENCE_SCHEMA,
            NETWORK_RECONNECT_EVIDENCE_SCHEMA,
            NETWORK_SCAN_EVIDENCE_SCHEMA,
            ASIC_INITIALIZATION_EVIDENCE_SCHEMA,
            ASIC_POWER_INITIALIZATION_EVIDENCE_SCHEMA,
            CORE_VOLTAGE_CONTROL_EVIDENCE_SCHEMA,
            DISPLAY_BEHAVIOR_EVIDENCE_SCHEMA,
            SCREEN_FLOW_EVIDENCE_SCHEMA,
            INA260_EVIDENCE_SCHEMA,
            INPUT_UAT_EVIDENCE_SCHEMA,
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
            RELEASE_RECOVERY_EVIDENCE_SCHEMA,
            UI_WORKFLOW_EVIDENCE_SCHEMA,
            MIGRATION_SCHEMA,
        ],
    }
}
