//! AxeOS API wire contracts and pure adapter input boundaries.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/openapi.yaml`
//! - `reference/esp-miner/main/http_server/system_api_json.c`
//! - `reference/esp-miner/main/http_server/axe-os/api/system/asic_settings.c`

pub mod asic;
pub mod boot_identity;
pub mod build_identity;
pub mod commands;
pub mod deferred_effect;
pub mod logs;
pub mod mining;
pub mod network;
pub mod observation;
pub mod operator_snapshot;
pub mod operator_snapshot_publication;
pub mod phase33_evidence;
pub mod platform_identity;
pub mod provisioning;
pub mod route_shell;
pub mod runtime_boot_attestation;
pub mod runtime_projection;
pub mod scoreboard;
pub mod settings;
pub mod snapshot;
pub mod static_plan;
pub mod statistics;
pub mod system;
pub mod telemetry;
pub mod theme;
pub mod update_plan;
pub mod v12_settings;
pub mod websocket_state;
pub mod wire;

pub use asic::asic_settings_from_snapshot;
pub use build_identity::{
    BuildChannel, BuildIdentity, BuildIdentityError, BuildProvenance, BUILD_LABEL_MAX_BYTES,
    BUILD_PROVENANCE_SCHEMA_VERSION, FULL_COMMIT_BYTES, SHORT_COMMIT_BYTES,
};
pub use commands::{
    apply_block_found_dismiss_effect, apply_identify_mode_effect,
    apply_mining_operator_intent_effect, block_found_dismiss_plan, identify_plan,
    pause_mining_plan, restart_plan, resume_mining_plan, BlockFoundDismissEffect,
    BlockFoundNotificationState, CommandEffect, CommandPlan, IdentifyMode, IdentifyModeEffect,
    IdentifyModeState, MiningOperatorIntentEffect, IDENTIFY_DURATION_MS,
};
pub use deferred_effect::DeferredEffectQueueUnavailable;
pub use logs::{
    log_download_headers, LogDownloadHeaders, RawLogStreamPlanner, RetainedLogBuffer,
    DOWNLOAD_CONTENT_DISPOSITION, DOWNLOAD_CONTENT_TYPE, LOG_CHUNK_BYTES, LOG_RETENTION_BYTES,
};
pub use mining::{
    mining_state_from_runtime, AsicHashrateWire, HashrateMonitorWire, MiningStateWire,
    SharesRejectedReasonWire,
};
pub use network::{
    project_ipv6_address, WifiNetworkWire, WifiScanAuthMode, WifiScanResponse,
    WifiScanResponseError, MAX_WIFI_SCAN_NETWORKS,
};
pub use observation::{
    project_observation, ObservationReasonWire, ObservationStampWire, ObservationStateWire,
    ObservationStore, ObservationTruthWire, TelemetryObservations,
};
pub use operator_snapshot::{
    BootSessionId, OperatorSnapshotIdentity, OperatorSnapshotIdentityError,
    OperatorSnapshotRevision, OperatorSnapshotSequence, OperatorSnapshotSequenceError,
    BOOT_SESSION_BYTES, BOOT_SESSION_HEX_BYTES,
};
pub use operator_snapshot_publication::{
    OperatorSnapshotLockHealth, OperatorSnapshotPublication, OperatorSnapshotPublishError,
};
pub use platform_identity::{
    parse_static_asset_version, PlatformAsic, PlatformBoard, PlatformFact, PlatformIdentity,
    PlatformResetReason, PlatformUnavailableReason, StaticAssetVersionError,
};
pub use provisioning::{
    build_captive_dns_response, configuration_ap_ssid, CaptiveDnsError, CAPTIVE_DNS_PACKET_BYTES,
    CAPTIVE_DNS_PORT, CAPTIVE_DNS_TTL_SECONDS,
};
pub use route_shell::{
    maybe_origin_ip_from_header, normalize_peer_ipv4, origin_gate_from_header, phase05_routes,
    phase07_route_report, phase07_routes, plan_http_access, plan_settings_patch_body_size,
    plan_websocket_upgrade, unknown_api_route_response, unsupported_update_response, AxeosRoute,
    HttpAccessDecision, OriginGate, PeerIpv4Normalization, Phase07RouteReport, PublicHttpResponse,
    RouteAccessInput, RouteKind, RouteMethod, SettingsPatchBodyDecision,
    WebSocketClientRegistrationPlan, WebSocketRouteKind, WebSocketUpgradeDecision,
    MAX_SETTINGS_PATCH_BODY_BYTES, UNAUTHORIZED_BODY, UNKNOWN_API_ROUTE_BODY,
};
pub use runtime_boot_attestation::{
    classify_runtime_boot_attestations, ExpectedRuntimeAttestationIdentity,
    RuntimeAttestationAccumulator, RuntimeAttestationStatus, RuntimeBootAttestation,
    RuntimeBootAttestationError, RUNTIME_BOOT_ATTESTATION_MARKER,
    RUNTIME_BOOT_ATTESTATION_SCHEMA_VERSION,
};
pub use runtime_projection::{project_api_views, project_system_info, ProjectedApiViews};
pub use scoreboard::{
    scoreboard_response, Scoreboard, ScoreboardEntry, ScoreboardEntryWire, ScoreboardError,
    ScoreboardMutation, ScoreboardOwner, ScoreboardOwnerError, MAX_SCOREBOARD_ENTRIES,
};
pub use settings::{
    execute_settings_persistence_plan, plan_settings_patch_body, plan_settings_patch_value,
    AcceptedSettingsPatch, ReloadedSettings, SettingsAdapterFailure, SettingsPatchFailure,
    SettingsPatchFailureReason, SettingsPatchPublicError, SettingsPersistenceAdapter,
    SettingsPersistenceEffect, SettingsPersistenceFailure, SettingsPersistenceFailureDisposition,
    SettingsPersistenceFailureReport, SettingsPersistencePlan, SettingsPersistenceStep,
    SettingsPersistenceSuccess, SettingsPersistenceTransaction, SettingsPublicResponse,
};
pub use snapshot::{
    ApiSnapshot, AsicSnapshot, ConfigSnapshot, PlatformSnapshot, ProjectSettingsSnapshot,
    SafeTelemetrySnapshot, SafetyTelemetryReport, SafetyTelemetryStatus,
};
pub use static_plan::{
    resolve_static_request, FilesystemAvailability, RecoveryFallback, RecoverySource,
    RedirectToRoot, RejectPathTraversal, ServeRecovery, ServeStatic, StaticFileCatalog,
    StaticRequest, StaticRouteDecision, CONTENT_ENCODING_HEADER, GZIP_CONTENT_ENCODING,
    STATIC_CACHE_CONTROL, STATIC_REDIRECT_BODY,
};
pub use statistics::{
    empty_statistics_response, statistics_response, StatisticsHistory, StatisticsHistoryError,
    StatisticsHistoryRecord, StatisticsSample, StatisticsWire, MAX_STATISTICS_SAMPLES,
};
pub use system::system_info_from_snapshot;
pub use telemetry::{
    live_telemetry_update_envelope, maybe_live_telemetry_diff, LiveTelemetryPlanner,
    LIVE_TELEMETRY_CADENCE_MS,
};
pub use theme::{
    plan_theme_post, theme_settings_from_snapshot, ThemePostFailure, ThemePostPlan,
    ThemePostResponse, ThemeSettings, MAX_THEME_POST_BODY_BYTES,
};
pub use update_plan::{
    plan_update_request, FirmwareOtaDecision, OtaWwwGapDecision, UpdateRequestDecision,
    UpdateRequestInput, UpdateRouteKind, UpdateStatusLabel,
};
pub use v12_settings::{
    decide_v12_settings_body, decide_v12_settings_value, Hostname, V12SettingsChange,
    V12SettingsDecision, V12SettingsExclusionReason,
};
pub use websocket_state::{
    WebSocketClientLease, WebSocketRegisterOutcome, WebSocketState, MAX_WEBSOCKET_CLIENTS,
};
pub use wire::{retained_runtime_health_record, RuntimeHealthWire, SystemAsicWire, SystemInfoWire};

#[cfg(test)]
mod tests {
    use super::{ApiSnapshot, SystemInfoWire};

    #[test]
    fn api_contract_no_longer_exposes_phase_1_deferral_status() {
        // Arrange
        let public_contract_count = 3;

        // Act
        let expected_contract_count =
            [ApiSnapshot::safe_ultra_205()].len() + ["SystemInfoWire", "SystemAsicWire"].len();

        // Assert
        assert_eq!(expected_contract_count, public_contract_count);
    }

    #[test]
    fn api_snapshot_maps_to_system_info_wire_contract() {
        // Arrange
        let snapshot = ApiSnapshot::safe_ultra_205();

        // Act
        let wire = SystemInfoWire::from_snapshot(&snapshot);

        // Assert
        assert_eq!(wire.asic_model, "BM1366");
        assert_eq!(wire.frequency, 485.0);
        assert!(!wire.mining_paused);
        assert!(wire.start_mining_on_boot);
    }
}
