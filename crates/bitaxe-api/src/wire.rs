//! Handwritten AxeOS wire DTOs for the initial system and ASIC contracts.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/system_api_json.c`
//! - `reference/esp-miner/main/http_server/openapi.yaml`
//! - `reference/esp-miner/main/http_server/axe-os/api/system/asic_settings.c`

use core::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use bitaxe_core::runtime_health::RuntimeHealthSnapshot;

use crate::legacy_units::{milliamps_from_amps, millivolts_from_volts};
use crate::mining::{mining_state_from_runtime, HashrateMonitorWire, SharesRejectedReasonWire};
use crate::{
    ApiSnapshot, BootSessionId, ObservationTruthWire, OperatorSnapshotRevision, PlatformIdentity,
    SYSTEM_INFO_STATISTICS_LIMIT,
};

/// Error type for host-side fixture compatibility helpers.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum WireCompatibilityError {
    #[error("missing required AxeOS wire field {field}")]
    MissingRequiredField { field: &'static str },
}

/// Verifies that a structured JSON value contains required AxeOS fields.
pub fn require_wire_keys(
    value: &Value,
    keys: &[&'static str],
) -> Result<(), WireCompatibilityError> {
    for field in keys {
        if value.get(field).is_none() {
            return Err(WireCompatibilityError::MissingRequiredField { field });
        }
    }

    Ok(())
}

/// Initial `/api/system/info` wire DTO slice.
#[derive(Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemInfoWire {
    #[serde(rename = "bootSession")]
    pub boot_session: BootSessionId,
    #[serde(rename = "bootOrdinal")]
    pub boot_ordinal: u64,
    #[serde(rename = "resetReasonCategory")]
    pub reset_reason_category: String,
    #[serde(rename = "operatorSnapshotRevision")]
    pub operator_snapshot_revision: OperatorSnapshotRevision,
    #[serde(rename = "platformIdentity")]
    pub platform_identity: PlatformIdentity,
    #[serde(rename = "runtimeHealth")]
    pub runtime_health: RuntimeHealthWire,
    #[serde(rename = "ASICModel")]
    pub asic_model: String,
    #[serde(rename = "boardVersion")]
    pub board_version: String,
    #[serde(rename = "hashRate")]
    pub hash_rate: f64,
    #[serde(rename = "hashRate_1m")]
    pub hash_rate_1m: f64,
    #[serde(rename = "hashRate_10m")]
    pub hash_rate_10m: f64,
    #[serde(rename = "hashRate_1h")]
    pub hash_rate_1h: f64,
    #[serde(rename = "hashrateMonitor")]
    pub hashrate_monitor: HashrateMonitorWire,
    #[serde(rename = "fanspeed")]
    pub fan_speed: u16,
    #[serde(rename = "fanrpm")]
    pub fan_rpm: u16,
    #[serde(rename = "fan2rpm")]
    pub fan2_rpm: u16,
    #[serde(rename = "fanRpmStatus")]
    pub fan_rpm_status: ObservationTruthWire,
    #[serde(rename = "miningPaused")]
    pub mining_paused: bool,
    #[serde(rename = "miningActivity")]
    pub mining_activity: String,
    #[serde(rename = "startMiningOnBoot")]
    pub start_mining_on_boot: bool,
    #[serde(rename = "apEnabled")]
    pub ap_enabled: u8,
    #[serde(rename = "autofanspeed")]
    pub auto_fan_speed: u8,
    pub display: String,
    pub rotation: u16,
    #[serde(rename = "invertscreen")]
    pub invert_screen: u8,
    #[serde(rename = "displayTimeout")]
    pub display_timeout: i32,
    #[serde(rename = "manualFanSpeed")]
    pub manual_fan_speed: u16,
    #[serde(rename = "minFanSpeed")]
    pub min_fan_speed: u16,
    #[serde(rename = "temptarget")]
    pub temp_target: u16,
    #[serde(rename = "statsFrequency")]
    pub stats_frequency: u16,
    #[serde(rename = "statsLimit")]
    pub stats_limit: u16,
    #[serde(rename = "overclockEnabled")]
    pub overclock_enabled: u8,
    #[serde(rename = "overheat_mode")]
    pub overheat_mode: u8,
    #[serde(rename = "showNewBlock")]
    pub show_new_block: bool,
    #[serde(rename = "blockFound")]
    pub block_found: u64,
    #[serde(rename = "blockHeight", skip_serializing_if = "Option::is_none")]
    pub maybe_block_height: Option<u64>,
    #[serde(rename = "scriptsig", skip_serializing_if = "Option::is_none")]
    pub maybe_script_sig: Option<String>,
    #[serde(rename = "networkDifficulty", skip_serializing_if = "Option::is_none")]
    pub maybe_network_difficulty: Option<f64>,
    #[serde(
        rename = "coinbaseValueTotalSatoshis",
        skip_serializing_if = "Option::is_none"
    )]
    pub maybe_coinbase_value_total_satoshis: Option<u64>,
    #[serde(
        rename = "coinbaseValueUserSatoshis",
        skip_serializing_if = "Option::is_none"
    )]
    pub maybe_coinbase_value_user_satoshis: Option<u64>,
    #[serde(rename = "blockSignals", skip_serializing_if = "Option::is_none")]
    pub maybe_block_signals: Option<Vec<String>>,
    #[serde(rename = "coinbaseOutputs", skip_serializing_if = "Option::is_none")]
    pub maybe_coinbase_outputs: Option<Vec<CoinbaseOutputWire>>,
    #[serde(rename = "frequency")]
    pub frequency: f64,
    #[serde(rename = "actualFrequency")]
    pub actual_frequency: f64,
    #[serde(rename = "coreVoltage")]
    pub core_voltage: u16,
    #[serde(rename = "coreVoltageActual")]
    pub core_voltage_actual: f64,
    #[serde(rename = "coreVoltageActualStatus")]
    pub core_voltage_actual_status: ObservationTruthWire,
    #[serde(rename = "power")]
    pub power: f64,
    #[serde(rename = "powerStatus")]
    pub power_status: ObservationTruthWire,
    #[serde(rename = "voltage")]
    pub voltage_millivolts: f64,
    #[serde(rename = "voltageStatus")]
    pub voltage_status: ObservationTruthWire,
    #[serde(rename = "current")]
    pub current_milliamps: f64,
    #[serde(rename = "currentStatus")]
    pub current_status: ObservationTruthWire,
    #[serde(rename = "temp")]
    pub temp: f64,
    #[serde(rename = "chipTempStatus")]
    pub chip_temp_status: ObservationTruthWire,
    #[serde(rename = "temp2")]
    pub temp2: f64,
    #[serde(rename = "vrTemp")]
    pub vr_temp: f64,
    #[serde(rename = "vrTempStatus")]
    pub vr_temp_status: ObservationTruthWire,
    #[serde(rename = "expectedHashrate")]
    pub expected_hashrate: f64,
    #[serde(rename = "sharesAccepted")]
    pub shares_accepted: u64,
    #[serde(rename = "sharesRejected")]
    pub shares_rejected: u64,
    #[serde(rename = "sharesRejectedReasons")]
    pub shares_rejected_reasons: Vec<SharesRejectedReasonWire>,
    #[serde(rename = "bestDiff")]
    pub best_diff: f64,
    #[serde(rename = "bestSessionDiff")]
    pub best_session_diff: f64,
    #[serde(rename = "poolDifficulty")]
    pub pool_difficulty: f64,
    #[serde(rename = "poolConnectionInfo")]
    pub pool_connection_info: String,
    #[serde(rename = "responseTime")]
    pub response_time: f64,
    #[serde(rename = "responseShareBatch")]
    pub response_share_batch: u64,
    #[serde(rename = "processTime")]
    pub process_time: f64,
    #[serde(rename = "errorPercentage")]
    pub error_percentage: f64,
    #[serde(rename = "isUsingFallbackStratum")]
    pub is_using_fallback_stratum: u8,
    #[serde(rename = "stratumURL")]
    pub stratum_url: String,
    #[serde(rename = "stratumPort")]
    pub stratum_port: u16,
    #[serde(rename = "stratumUser")]
    pub stratum_user: String,
    #[serde(rename = "stratumSuggestedDifficulty")]
    pub stratum_suggested_difficulty: u16,
    #[serde(rename = "stratumExtranonceSubscribe")]
    pub stratum_extranonce_subscribe: bool,
    #[serde(rename = "stratumTLS")]
    pub stratum_tls: u16,
    #[serde(rename = "stratumCert")]
    pub stratum_certificate: String,
    #[serde(rename = "stratumDecodeCoinbase")]
    pub stratum_decode_coinbase: bool,
    #[serde(rename = "stratumProtocol")]
    pub stratum_protocol: String,
    #[serde(rename = "stratumV2AuthorityPubkey")]
    pub stratum_v2_authority_public_key: String,
    #[serde(rename = "stratumV2ChannelType")]
    pub stratum_v2_channel_type: String,
    #[serde(rename = "fallbackStratumURL")]
    pub fallback_stratum_url: String,
    #[serde(rename = "fallbackStratumPort")]
    pub fallback_stratum_port: u16,
    #[serde(rename = "fallbackStratumUser")]
    pub fallback_stratum_user: String,
    #[serde(rename = "fallbackStratumSuggestedDifficulty")]
    pub fallback_stratum_suggested_difficulty: u16,
    #[serde(rename = "fallbackStratumExtranonceSubscribe")]
    pub fallback_stratum_extranonce_subscribe: bool,
    #[serde(rename = "fallbackStratumTLS")]
    pub fallback_stratum_tls: u16,
    #[serde(rename = "fallbackStratumCert")]
    pub fallback_stratum_certificate: String,
    #[serde(rename = "fallbackStratumDecodeCoinbase")]
    pub fallback_stratum_decode_coinbase: bool,
    #[serde(rename = "fallbackStratumProtocol")]
    pub fallback_stratum_protocol: String,
    #[serde(rename = "fallbackStratumV2AuthorityPubkey")]
    pub fallback_stratum_v2_authority_public_key: String,
    #[serde(rename = "fallbackStratumV2ChannelType")]
    pub fallback_stratum_v2_channel_type: String,
    #[serde(rename = "maxPower")]
    pub max_power: u16,
    #[serde(rename = "nominalVoltage")]
    pub nominal_voltage: u16,
    #[serde(rename = "smallCoreCount")]
    pub small_core_count: u16,
    #[serde(rename = "isPSRAMAvailable")]
    pub is_psram_available: u8,
    #[serde(rename = "wifiRSSI")]
    pub wifi_rssi: i16,
    #[serde(rename = "wifiStatus")]
    pub wifi_status: String,
    #[serde(rename = "cpuUsage")]
    pub cpu_usage: f64,
    #[serde(rename = "power_fault", skip_serializing_if = "Option::is_none")]
    pub maybe_power_fault: Option<String>,
    #[serde(rename = "hardware_fault", skip_serializing_if = "Option::is_none")]
    pub maybe_hardware_fault: Option<String>,
    #[serde(rename = "version")]
    pub version: String,
    #[serde(rename = "semanticVersion")]
    pub semantic_version: String,
    #[serde(rename = "sourceCommit")]
    pub source_commit: String,
    #[serde(rename = "referenceCommit")]
    pub reference_commit: String,
    #[serde(rename = "appElfSha256")]
    pub app_elf_sha256: String,
    #[serde(rename = "buildTimestampUtc")]
    pub build_timestamp_utc: String,
    #[serde(rename = "buildChannel")]
    pub build_channel: String,
    #[serde(rename = "sourceDirty")]
    pub source_dirty: bool,
    #[serde(rename = "releaseTag")]
    pub maybe_release_tag: Option<String>,
    #[serde(rename = "axeOSVersion")]
    pub axe_os_version: String,
    #[serde(rename = "idfVersion")]
    pub idf_version: String,
    #[serde(rename = "resetReason")]
    pub reset_reason: String,
    #[serde(rename = "runningPartition")]
    pub running_partition: String,
    #[serde(rename = "macAddr")]
    pub mac_addr: String,
    #[serde(rename = "hostname")]
    pub hostname: String,
    #[serde(rename = "ssid")]
    pub ssid: String,
    #[serde(rename = "ipv4")]
    pub ipv4: String,
    #[serde(rename = "ipv6")]
    pub ipv6: String,
    #[serde(rename = "uptimeSeconds")]
    pub uptime_seconds: u64,
    #[serde(rename = "freeHeap")]
    pub free_heap: u64,
    #[serde(rename = "freeHeapInternal")]
    pub free_heap_internal: u64,
    #[serde(rename = "freeHeapSpiram")]
    pub free_heap_spiram: u64,
    #[serde(rename = "minFreeHeap")]
    pub min_free_heap: u64,
    #[serde(rename = "maxAllocHeap")]
    pub max_alloc_heap: u64,
}

impl fmt::Debug for SystemInfoWire {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("SystemInfoWire([redacted-response])")
    }
}

impl SystemInfoWire {
    /// Maps typed runtime facts into the initial AxeOS system info DTO.
    #[must_use]
    pub fn from_snapshot(snapshot: &ApiSnapshot) -> Self {
        let config = snapshot.config;
        let safe_telemetry = snapshot.safe_telemetry.operator_projection();
        let mining_state = mining_state_from_runtime(&snapshot.mining);
        let platform = &snapshot.platform;
        let settings = &snapshot.system_info_settings;
        let primary_pool = &settings.primary_pool;
        let fallback_pool = &settings.fallback_pool;
        let maybe_block = snapshot.maybe_block.as_ref();

        Self {
            boot_session: snapshot.operator_snapshot_identity.boot_session(),
            boot_ordinal: platform.boot_ordinal,
            reset_reason_category: platform.reset_reason_category.clone(),
            operator_snapshot_revision: snapshot.operator_snapshot_identity.revision(),
            platform_identity: snapshot.platform_identity.clone(),
            runtime_health: RuntimeHealthWire::from(&snapshot.runtime_health),
            asic_model: snapshot.catalog.asic().model().to_owned(),
            board_version: snapshot.catalog.board_version().to_owned(),
            hash_rate: mining_state.hash_rate,
            hash_rate_1m: mining_state.hash_rate_1m,
            hash_rate_10m: mining_state.hash_rate_10m,
            hash_rate_1h: mining_state.hash_rate_1h,
            hashrate_monitor: mining_state.hashrate_monitor,
            fan_speed: safe_telemetry.fan_speed_percent,
            fan_rpm: safe_telemetry.fan_rpm,
            fan2_rpm: safe_telemetry.fan2_rpm,
            fan_rpm_status: safe_telemetry.fan_rpm_status,
            mining_paused: mining_state.mining_paused,
            mining_activity: mining_state.mining_activity,
            start_mining_on_boot: snapshot.project_settings.start_mining_on_boot,
            ap_enabled: numeric_bool(platform.ap_enabled),
            auto_fan_speed: numeric_bool(config.auto_fan_speed),
            display: settings.display.clone(),
            rotation: settings.rotation,
            invert_screen: numeric_bool(settings.invert_screen),
            display_timeout: settings.display_timeout,
            manual_fan_speed: settings.manual_fan_speed,
            min_fan_speed: settings.min_fan_speed,
            temp_target: settings.temp_target,
            stats_frequency: settings.stats_frequency,
            stats_limit: SYSTEM_INFO_STATISTICS_LIMIT,
            overclock_enabled: numeric_bool(settings.overclock_enabled),
            overheat_mode: numeric_bool(settings.overheat_mode),
            show_new_block: snapshot.block_found.show_new_block,
            block_found: snapshot.block_found.block_found,
            maybe_block_height: maybe_block.map(|block| block.height),
            maybe_script_sig: maybe_block.map(|block| block.script_sig.clone()),
            maybe_network_difficulty: maybe_block.map(|block| block.network_difficulty),
            maybe_coinbase_value_total_satoshis: maybe_block
                .map(|block| block.coinbase_value_total_satoshis),
            maybe_coinbase_value_user_satoshis: maybe_block
                .map(|block| block.coinbase_value_user_satoshis),
            maybe_block_signals: maybe_block.map(|block| block.signals.clone()),
            maybe_coinbase_outputs: maybe_block.map(|block| {
                block
                    .coinbase_outputs
                    .iter()
                    .map(CoinbaseOutputWire::from)
                    .collect()
            }),
            frequency: config.asic_frequency_mhz,
            actual_frequency: safe_telemetry.actual_frequency_mhz,
            core_voltage: config.asic_voltage_mv,
            core_voltage_actual: safe_telemetry.core_voltage_actual_mv,
            core_voltage_actual_status: safe_telemetry.core_voltage_status,
            power: safe_telemetry.power_watts,
            power_status: safe_telemetry.power_status,
            voltage_millivolts: millivolts_from_volts(safe_telemetry.voltage_volts),
            voltage_status: safe_telemetry.voltage_status,
            current_milliamps: milliamps_from_amps(safe_telemetry.current_amps),
            current_status: safe_telemetry.current_status,
            temp: safe_telemetry.chip_temp_celsius,
            chip_temp_status: safe_telemetry.chip_temp_status,
            temp2: safe_telemetry.chip_temp2_celsius,
            vr_temp: safe_telemetry.vr_temp_celsius,
            vr_temp_status: safe_telemetry.vr_temp_status,
            expected_hashrate: safe_telemetry.expected_hashrate_ghs,
            shares_accepted: mining_state.shares_accepted,
            shares_rejected: mining_state.shares_rejected,
            shares_rejected_reasons: mining_state.shares_rejected_reasons,
            best_diff: mining_state.best_diff,
            best_session_diff: mining_state.best_session_diff,
            pool_difficulty: mining_state.pool_difficulty,
            pool_connection_info: mining_state.pool_connection_info,
            response_time: mining_state.response_time,
            response_share_batch: mining_state.response_share_batch,
            process_time: mining_state.process_time,
            error_percentage: mining_state.error_percentage,
            is_using_fallback_stratum: mining_state.is_using_fallback_stratum,
            stratum_url: primary_pool.url.clone(),
            stratum_port: primary_pool.port,
            stratum_user: primary_pool.user.clone(),
            stratum_suggested_difficulty: primary_pool.suggested_difficulty,
            stratum_extranonce_subscribe: primary_pool.extranonce_subscribe,
            stratum_tls: primary_pool.tls,
            stratum_certificate: primary_pool.certificate.clone(),
            stratum_decode_coinbase: primary_pool.decode_coinbase,
            stratum_protocol: primary_pool.protocol.clone(),
            stratum_v2_authority_public_key: primary_pool.v2_authority_public_key.clone(),
            stratum_v2_channel_type: primary_pool.v2_channel_type.clone(),
            fallback_stratum_url: fallback_pool.url.clone(),
            fallback_stratum_port: fallback_pool.port,
            fallback_stratum_user: fallback_pool.user.clone(),
            fallback_stratum_suggested_difficulty: fallback_pool.suggested_difficulty,
            fallback_stratum_extranonce_subscribe: fallback_pool.extranonce_subscribe,
            fallback_stratum_tls: fallback_pool.tls,
            fallback_stratum_certificate: fallback_pool.certificate.clone(),
            fallback_stratum_decode_coinbase: fallback_pool.decode_coinbase,
            fallback_stratum_protocol: fallback_pool.protocol.clone(),
            fallback_stratum_v2_authority_public_key: fallback_pool.v2_authority_public_key.clone(),
            fallback_stratum_v2_channel_type: fallback_pool.v2_channel_type.clone(),
            max_power: config.max_power_watts,
            nominal_voltage: config.nominal_voltage_volts,
            small_core_count: snapshot.catalog.asic().small_core_count(),
            is_psram_available: numeric_bool(platform.psram_available),
            wifi_rssi: safe_telemetry.wifi_rssi_dbm,
            wifi_status: platform.wifi_status.clone(),
            cpu_usage: platform.cpu_usage_percent,
            maybe_power_fault: platform.maybe_power_fault.clone(),
            maybe_hardware_fault: platform.maybe_hardware_fault.clone(),
            version: platform.version.clone(),
            semantic_version: platform.semantic_version.clone(),
            source_commit: platform.source_commit.clone(),
            reference_commit: platform.reference_commit.clone(),
            app_elf_sha256: platform.app_elf_sha256.clone(),
            build_timestamp_utc: platform.build_timestamp_utc.clone(),
            build_channel: platform.build_channel.clone(),
            source_dirty: platform.source_dirty,
            maybe_release_tag: platform.maybe_release_tag.clone(),
            axe_os_version: platform.axe_os_version.clone(),
            idf_version: platform.idf_version.clone(),
            reset_reason: platform.reset_reason.clone(),
            running_partition: platform.running_partition.clone(),
            mac_addr: platform.mac_addr.clone(),
            hostname: platform.hostname.clone(),
            ssid: platform.ssid.clone(),
            ipv4: platform.ipv4.clone(),
            ipv6: platform.ipv6.clone(),
            uptime_seconds: platform.uptime_seconds,
            free_heap: platform.free_heap,
            free_heap_internal: platform.free_heap_internal,
            free_heap_spiram: platform.free_heap_spiram,
            min_free_heap: platform.min_free_heap,
            max_alloc_heap: platform.max_alloc_heap,
        }
    }
}

/// One conditional coinbase output in the upstream response shape.
#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CoinbaseOutputWire {
    pub value: u64,
    pub address: String,
}

impl fmt::Debug for CoinbaseOutputWire {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CoinbaseOutputWire")
            .field("value", &self.value)
            .field("address", &"[redacted]")
            .finish()
    }
}

impl From<&crate::SystemInfoCoinbaseOutput> for CoinbaseOutputWire {
    fn from(output: &crate::SystemInfoCoinbaseOutput) -> Self {
        Self {
            value: output.value_satoshis,
            address: output.address.clone(),
        }
    }
}

/// Additive passive runtime-health projection shared by system-info and live telemetry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeHealthWire {
    #[serde(rename = "selfTestState")]
    pub self_test_state: String,
    #[serde(rename = "supervisorAvailability")]
    pub supervisor_availability: String,
    #[serde(rename = "checkpointCategory")]
    pub maybe_checkpoint_category: Option<String>,
    #[serde(rename = "checkpointSequence")]
    pub maybe_checkpoint_sequence: Option<u64>,
    #[serde(rename = "checkpointAgeMillis")]
    pub maybe_checkpoint_age_millis: Option<u64>,
    #[serde(rename = "checkpointHealth")]
    pub checkpoint_health: String,
    #[serde(rename = "taskWatchdogParticipation")]
    pub task_watchdog_participation: String,
    #[serde(rename = "taskWatchdogReason")]
    pub maybe_task_watchdog_reason: Option<String>,
    #[serde(rename = "taskWatchdogFeedSequence")]
    pub maybe_task_watchdog_feed_sequence: Option<u64>,
    #[serde(rename = "taskWatchdogFeedAgeMillis")]
    pub maybe_task_watchdog_feed_age_millis: Option<u64>,
    #[serde(rename = "taskWatchdogOwnerPhase", default = "unavailable_owner_phase")]
    pub task_watchdog_owner_phase: String,
}
impl From<&RuntimeHealthSnapshot> for RuntimeHealthWire {
    fn from(snapshot: &RuntimeHealthSnapshot) -> Self {
        Self {
            self_test_state: snapshot.passive_self_test_state().as_str().to_owned(),
            supervisor_availability: snapshot.supervisor_availability().as_str().to_owned(),
            maybe_checkpoint_category: snapshot.maybe_checkpoint_category().map(str::to_owned),
            maybe_checkpoint_sequence: snapshot.maybe_checkpoint_sequence(),
            maybe_checkpoint_age_millis: snapshot.maybe_checkpoint_age_millis(),
            checkpoint_health: snapshot.checkpoint_health().as_str().to_owned(),
            task_watchdog_participation: snapshot.task_watchdog_participation().as_str().to_owned(),
            maybe_task_watchdog_reason: snapshot.maybe_task_watchdog_reason().map(str::to_owned),
            maybe_task_watchdog_feed_sequence: snapshot.maybe_task_watchdog_feed_sequence(),
            maybe_task_watchdog_feed_age_millis: snapshot.maybe_task_watchdog_feed_age_millis(),
            task_watchdog_owner_phase: snapshot.task_watchdog_owner_phase().as_str().to_owned(),
        }
    }
}
fn unavailable_owner_phase() -> String {
    "unavailable".to_owned()
}
/// Renders the redacted retained runtime-health record for one coherent capture.
#[must_use]
pub fn retained_runtime_health_record(
    boot_session: BootSessionId,
    operator_snapshot_revision: OperatorSnapshotRevision,
    snapshot: &RuntimeHealthSnapshot,
) -> String {
    let checkpoint_category = snapshot
        .maybe_checkpoint_category()
        .unwrap_or("unavailable");
    let checkpoint_sequence = optional_u64(snapshot.maybe_checkpoint_sequence());
    let checkpoint_age_millis = optional_u64(snapshot.maybe_checkpoint_age_millis());
    let task_watchdog_reason = snapshot
        .maybe_task_watchdog_reason()
        .unwrap_or("unavailable");
    let task_watchdog_feed_sequence = optional_u64(snapshot.maybe_task_watchdog_feed_sequence());
    let task_watchdog_feed_age_millis =
        optional_u64(snapshot.maybe_task_watchdog_feed_age_millis());

    format!(
        "runtime_health boot_session={boot_session} operator_snapshot_revision={} self_test={} supervisor={} checkpoint_category={checkpoint_category} checkpoint_sequence={checkpoint_sequence} checkpoint_age_millis={checkpoint_age_millis} checkpoint_health={} task_watchdog_participation={} task_watchdog_reason={task_watchdog_reason} task_watchdog_feed_sequence={task_watchdog_feed_sequence} task_watchdog_feed_age_millis={task_watchdog_feed_age_millis} task_watchdog_owner_phase={} redacted=true",
        operator_snapshot_revision.get(),
        snapshot.passive_self_test_state().as_str(),
        snapshot.supervisor_availability().as_str(),
        snapshot.checkpoint_health().as_str(),
        snapshot.task_watchdog_participation().as_str(),
        snapshot.task_watchdog_owner_phase().as_str(),
    )
}
fn optional_u64(maybe_value: Option<u64>) -> String {
    maybe_value.map_or_else(|| "unavailable".to_owned(), |value| value.to_string())
}
/// Initial `/api/system/asic` wire DTO.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SystemAsicWire {
    #[serde(rename = "ASICModel")]
    pub asic_model: String,
    #[serde(rename = "deviceModel")]
    pub device_model: String,
    #[serde(rename = "swarmColor")]
    pub swarm_color: String,
    #[serde(rename = "asicCount")]
    pub asic_count: u8,
    #[serde(rename = "hashDomains")]
    pub hash_domains: u8,
    #[serde(rename = "defaultFrequency")]
    pub default_frequency: u16,
    #[serde(rename = "frequencyOptions")]
    pub frequency_options: Vec<u16>,
    #[serde(rename = "defaultVoltage")]
    pub default_voltage: u16,
    #[serde(rename = "voltageOptions")]
    pub voltage_options: Vec<u16>,
}

impl SystemAsicWire {
    /// Maps typed Ultra 205 catalog facts into the AxeOS ASIC DTO.
    #[must_use]
    pub fn from_snapshot(snapshot: &ApiSnapshot) -> Self {
        let asic = snapshot.catalog.asic();

        Self {
            asic_model: asic.model().to_owned(),
            device_model: snapshot.catalog.family().to_owned(),
            swarm_color: swarm_color_for_family(snapshot.catalog.family()).to_owned(),
            asic_count: snapshot.catalog.asic_count(),
            hash_domains: asic.hash_domains(),
            default_frequency: asic.default_frequency_mhz(),
            frequency_options: asic.frequency_options().to_vec(),
            default_voltage: asic.default_voltage_mv(),
            voltage_options: asic.voltage_options().to_vec(),
        }
    }
}

fn numeric_bool(value: bool) -> u8 {
    u8::from(value)
}

fn swarm_color_for_family(family: &str) -> &'static str {
    match family {
        "Ultra" => "purple",
        "Max" => "red",
        "Hex" => "orange",
        "Supra" => "blue",
        "Gamma" | "GammaDuo" => "green",
        "SupraHex" => "darkblue",
        "GammaTurbo" => "cyan",
        _ => "",
    }
}

#[cfg(test)]
mod tests;
