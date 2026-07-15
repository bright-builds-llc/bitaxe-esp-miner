//! Handwritten AxeOS wire DTOs for the initial system and ASIC contracts.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/http_server/system_api_json.c`
//! - `reference/esp-miner/main/http_server/openapi.yaml`
//! - `reference/esp-miner/main/http_server/axe-os/api/system/asic_settings.c`

use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::mining::{mining_state_from_runtime, SharesRejectedReasonWire};
use crate::{ApiSnapshot, BootSessionId, ObservationTruthWire, OperatorSnapshotRevision};

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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SystemInfoWire {
    #[serde(rename = "bootSession")]
    pub boot_session: BootSessionId,
    #[serde(rename = "operatorSnapshotRevision")]
    pub operator_snapshot_revision: OperatorSnapshotRevision,
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
    #[serde(rename = "apEnabled")]
    pub ap_enabled: u8,
    #[serde(rename = "autofanspeed")]
    pub auto_fan_speed: u8,
    #[serde(rename = "showNewBlock")]
    pub show_new_block: bool,
    #[serde(rename = "blockFound")]
    pub block_found: u64,
    #[serde(rename = "frequency")]
    pub frequency: f64,
    #[serde(rename = "actualFrequency")]
    pub actual_frequency: f64,
    #[serde(rename = "coreVoltage")]
    pub core_voltage: u16,
    #[serde(rename = "coreVoltageActual")]
    pub core_voltage_actual: f64,
    #[serde(rename = "power")]
    pub power: f64,
    #[serde(rename = "powerStatus")]
    pub power_status: ObservationTruthWire,
    #[serde(rename = "voltage")]
    pub voltage: f64,
    #[serde(rename = "voltageStatus")]
    pub voltage_status: ObservationTruthWire,
    #[serde(rename = "current")]
    pub current: f64,
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

impl SystemInfoWire {
    /// Maps typed runtime facts into the initial AxeOS system info DTO.
    #[must_use]
    pub fn from_snapshot(snapshot: &ApiSnapshot) -> Self {
        let config = snapshot.config;
        let safe_telemetry = snapshot.safe_telemetry.operator_projection();
        let mining_state = mining_state_from_runtime(&snapshot.mining);
        let platform = &snapshot.platform;

        Self {
            boot_session: snapshot.operator_snapshot_identity.boot_session(),
            operator_snapshot_revision: snapshot.operator_snapshot_identity.revision(),
            asic_model: snapshot.catalog.asic().model().to_owned(),
            board_version: snapshot.catalog.board_version().to_owned(),
            hash_rate: mining_state.hash_rate,
            hash_rate_1m: mining_state.hash_rate_1m,
            hash_rate_10m: mining_state.hash_rate_10m,
            hash_rate_1h: mining_state.hash_rate_1h,
            fan_speed: safe_telemetry.fan_speed_percent,
            fan_rpm: safe_telemetry.fan_rpm,
            fan2_rpm: safe_telemetry.fan2_rpm,
            fan_rpm_status: safe_telemetry.fan_rpm_status,
            mining_paused: mining_state.mining_paused,
            ap_enabled: numeric_bool(platform.ap_enabled),
            auto_fan_speed: numeric_bool(config.auto_fan_speed),
            show_new_block: snapshot.block_found.show_new_block,
            block_found: snapshot.block_found.block_found,
            frequency: config.asic_frequency_mhz,
            actual_frequency: safe_telemetry.actual_frequency_mhz,
            core_voltage: config.asic_voltage_mv,
            core_voltage_actual: safe_telemetry.core_voltage_actual_mv,
            power: safe_telemetry.power_watts,
            power_status: safe_telemetry.power_status,
            voltage: safe_telemetry.voltage_volts,
            voltage_status: safe_telemetry.voltage_status,
            current: safe_telemetry.current_amps,
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
            error_percentage: 0.0,
            is_using_fallback_stratum: mining_state.is_using_fallback_stratum,
            max_power: snapshot.catalog.power_consumption_target(),
            nominal_voltage: 0,
            small_core_count: snapshot.catalog.asic().small_core_count(),
            is_psram_available: numeric_bool(platform.psram_available),
            wifi_rssi: safe_telemetry.wifi_rssi_dbm,
            wifi_status: platform.wifi_status.clone(),
            version: platform.version.clone(),
            semantic_version: platform.semantic_version.clone(),
            source_commit: platform.source_commit.clone(),
            reference_commit: platform.reference_commit.clone(),
            app_elf_sha256: platform.app_elf_sha256.clone(),
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
mod tests {
    use bitaxe_safety::observation::{
        BootSessionId, FaultReason, MonotonicMillis, Observation, ObservationSequence, StaleReason,
        UnavailableReason,
    };
    use serde_json::{json, Value};

    use super::require_wire_keys;
    use crate::{
        ApiSnapshot, SafeTelemetrySnapshot, SystemAsicWire, SystemInfoWire, TelemetryObservations,
    };

    #[test]
    fn system_info_wire_serializes_upstream_field_names_and_encodings() {
        // Arrange
        let snapshot = ApiSnapshot::safe_ultra_205();
        let wire = SystemInfoWire::from_snapshot(&snapshot);

        // Act
        let value = serde_json::to_value(wire).expect("system info should serialize");

        // Assert
        assert!(value.get("ASICModel").is_some());
        assert!(value.get("hashRate_1m").is_some());
        assert!(value.get("fanspeed").is_some());
        assert!(value.get("fanrpm").is_some());
        assert_eq!(value.get("miningPaused"), Some(&Value::Bool(true)));
        assert_eq!(value.get("apEnabled"), Some(&json!(0)));
        assert_eq!(value.get("autofanspeed"), Some(&json!(1)));
        assert_eq!(value.get("showNewBlock"), Some(&Value::Bool(false)));
        assert_eq!(value.get("version"), Some(&json!("000000000000-dev")));
        assert_eq!(value.get("semanticVersion"), Some(&json!("0.0.0-safe")));
        assert_eq!(value.get("sourceCommit"), Some(&json!("0".repeat(40))));
        assert_eq!(value.get("referenceCommit"), Some(&json!("0".repeat(40))));
        assert_eq!(value.get("appElfSha256"), Some(&json!("0".repeat(64))));
        assert_eq!(value.get("buildChannel"), Some(&json!("dev")));
        assert_eq!(value.get("sourceDirty"), Some(&Value::Bool(false)));
        assert_eq!(value.get("releaseTag"), Some(&Value::Null));
        assert_eq!(value.get("bootSession"), Some(&json!("0".repeat(32))));
        assert_eq!(value.get("operatorSnapshotRevision"), Some(&json!(1)));
        assert!(require_wire_keys(
            &value,
            &[
                "ASICModel",
                "hashRate_1m",
                "fanspeed",
                "fanrpm",
                "miningPaused",
                "apEnabled",
            ],
        )
        .is_ok());
    }

    #[test]
    fn safety_telemetry_system_info_exposes_exact_six_truth_fields() {
        // Arrange
        let snapshot = ApiSnapshot::safe_ultra_205();

        // Act
        let value = serde_json::to_value(SystemInfoWire::from_snapshot(&snapshot))
            .expect("system info should serialize");
        let status_fields = [
            "chipTempStatus",
            "currentStatus",
            "fanRpmStatus",
            "powerStatus",
            "voltageStatus",
            "vrTempStatus",
        ];

        // Assert
        for field in status_fields {
            assert_eq!(value[field]["state"], "unavailable");
        }
        for unsupported_field in [
            "fanSpeedStatus",
            "fan2RpmStatus",
            "chipTemp2Status",
            "coreVoltageStatus",
        ] {
            assert!(value.get(unsupported_field).is_none());
        }
    }

    #[test]
    fn system_info_wire_uses_block_found_notification_snapshot() {
        // Arrange
        let mut snapshot = ApiSnapshot::safe_ultra_205();
        snapshot.block_found.block_found = 840_000;
        snapshot.block_found.show_new_block = true;

        // Act
        let wire = SystemInfoWire::from_snapshot(&snapshot);

        // Assert
        assert_eq!(wire.block_found, 840_000);
        assert!(wire.show_new_block);
    }

    #[test]
    fn system_info_wire_uses_runtime_config_snapshot() {
        // Arrange
        let mut snapshot = ApiSnapshot::safe_ultra_205();
        snapshot.config.asic_frequency_mhz = 500.0;
        snapshot.config.asic_voltage_mv = 1_250;
        snapshot.config.auto_fan_speed = false;

        // Act
        let wire = SystemInfoWire::from_snapshot(&snapshot);

        // Assert
        assert_eq!(wire.frequency, 500.0);
        assert_eq!(wire.core_voltage, 1_250);
        assert_eq!(wire.auto_fan_speed, 0);
    }

    #[test]
    fn safety_telemetry_projection_system_info_reads_safe_telemetry_values() {
        // Arrange
        let mut snapshot = ApiSnapshot::safe_ultra_205();
        snapshot.safe_telemetry =
            SafeTelemetrySnapshot::from_observations(&fresh_telemetry_observations());

        // Act
        let wire = SystemInfoWire::from_snapshot(&snapshot);

        // Assert
        assert_eq!(wire.power, 11.5);
        assert_eq!(wire.voltage, 5.1);
        assert_eq!(wire.current, 2.25);
        assert_eq!(wire.fan_rpm, 3_200);
        assert_eq!(wire.temp, 56.0);
        assert_eq!(wire.vr_temp, 45.0);
        assert_eq!(wire.power_status.state, crate::ObservationStateWire::Fresh);
        assert!(wire.power_status.stamp.is_some());
    }

    #[test]
    fn system_info_wire_rejects_nonfresh_truth_numeric_claims_even_with_fresh_aggregate() {
        // Arrange
        let mut snapshot = ApiSnapshot::safe_ultra_205();
        snapshot.safe_telemetry =
            SafeTelemetrySnapshot::from_observations(&fresh_telemetry_observations());
        let stale_voltage = fresh_f64(5.1, 2)
            .mark_stale(StaleReason::PowerSampleStale)
            .expect("fresh voltage can become stale");
        let unavailable_power =
            Observation::<f64>::unavailable(UnavailableReason::PowerSampleUnavailable);
        let fault_current = fresh_f64(2.25, 3).record_fault(FaultReason::ReadFailed);
        snapshot.safe_telemetry.power_status = (&unavailable_power).into();
        snapshot.safe_telemetry.voltage_status = (&stale_voltage).into();
        snapshot.safe_telemetry.current_status = (&fault_current).into();
        assert_eq!(
            snapshot.safe_telemetry.status,
            crate::SafetyTelemetryStatus::Fresh
        );

        // Act
        let value = serde_json::to_value(SystemInfoWire::from_snapshot(&snapshot))
            .expect("system info should serialize");

        // Assert
        assert_eq!(value["power"], 0.0);
        assert_eq!(value["voltage"], 0.0);
        assert_eq!(value["current"], 0.0);
        assert_eq!(value["powerStatus"]["state"], "unavailable");
        assert_eq!(value["voltageStatus"]["state"], "stale");
        assert_eq!(value["currentStatus"]["state"], "fault");
        assert_eq!(value["temp"], 56.0);
        assert_eq!(value["chipTempStatus"]["state"], "fresh");
    }

    #[test]
    fn system_asic_wire_serializes_upstream_asic_contract_names() {
        // Arrange
        let snapshot = ApiSnapshot::safe_ultra_205();
        let wire = SystemAsicWire::from_snapshot(&snapshot);

        // Act
        let value = serde_json::to_value(wire).expect("system asic should serialize");

        // Assert
        assert_eq!(value.get("ASICModel"), Some(&json!("BM1366")));
        assert_eq!(value.get("deviceModel"), Some(&json!("Ultra")));
        assert_eq!(value.get("swarmColor"), Some(&json!("purple")));
        assert_eq!(value.get("asicCount"), Some(&json!(1)));
    }

    fn fresh_telemetry_observations() -> TelemetryObservations {
        TelemetryObservations {
            power_watts: fresh_f64(11.5, 1),
            bus_voltage_volts: fresh_f64(5.1, 2),
            current_amps: fresh_f64(2.25, 3),
            chip_temp_celsius: fresh_f64(56.0, 4),
            vr_temp_celsius: fresh_f64(45.0, 5),
            fan_rpm: fresh_u16(3_200, 6),
        }
    }

    fn fresh_f64(value: f64, prior_sequence: u64) -> Observation<f64> {
        Observation::record_success(
            value,
            BootSessionId::new(7),
            ObservationSequence::new(prior_sequence),
            MonotonicMillis::new(250),
        )
        .expect("fixture sequence should advance")
        .0
    }

    fn fresh_u16(value: u16, prior_sequence: u64) -> Observation<u16> {
        Observation::record_success(
            value,
            BootSessionId::new(7),
            ObservationSequence::new(prior_sequence),
            MonotonicMillis::new(250),
        )
        .expect("fixture sequence should advance")
        .0
    }

    #[test]
    fn wire_system_info_fixture_from_reference_safe_ultra_205_defaults_round_trips() {
        // Arrange
        let fixture = include_str!("../fixtures/api/system-info-ultra205-safe.json");
        let original: Value =
            serde_json::from_str(fixture).expect("system info fixture should be valid JSON");

        // Act
        let parsed: SystemInfoWire =
            serde_json::from_str(fixture).expect("system info fixture should parse");
        let round_trip =
            serde_json::to_value(parsed).expect("system info fixture should serialize");

        // Assert
        assert_eq!(round_trip, original);
    }

    #[test]
    fn wire_system_info_fixture_preserves_mixed_numeric_and_boolean_encodings() {
        // Arrange
        let fixture = include_str!("../fixtures/api/system-info-ultra205-safe.json");

        // Act
        let value: Value =
            serde_json::from_str(fixture).expect("system info fixture should be valid JSON");

        // Assert
        assert!(value["apEnabled"].is_number());
        assert!(value["autofanspeed"].is_number());
        assert_eq!(value["miningPaused"], Value::Bool(true));
        assert_eq!(value["showNewBlock"], Value::Bool(false));
    }

    #[test]
    fn wire_system_info_fixture_keeps_phase_6_hardware_telemetry_safe() {
        // Arrange
        let fixture = include_str!("../fixtures/api/system-info-ultra205-safe.json");

        // Act
        let value: Value =
            serde_json::from_str(fixture).expect("system info fixture should be valid JSON");

        // Assert
        assert_eq!(value["power"], json!(0.0));
        assert_eq!(value["voltage"], json!(0.0));
        assert_eq!(value["current"], json!(0.0));
        assert_eq!(value["temp"], json!(0.0));
        assert_eq!(value["fanspeed"], json!(0));
        assert_eq!(value["fanrpm"], json!(0));
        assert_eq!(value["actualFrequency"], json!(0.0));
        assert_eq!(value["expectedHashrate"], json!(0.0));
    }

    #[test]
    fn wire_system_asic_fixture_from_reference_safe_ultra_205_defaults_round_trips() {
        // Arrange
        let fixture = include_str!("../fixtures/api/asic-settings-ultra205.json");
        let original: Value =
            serde_json::from_str(fixture).expect("ASIC settings fixture should be valid JSON");

        // Act
        let parsed: SystemAsicWire =
            serde_json::from_str(fixture).expect("ASIC settings fixture should parse");
        let round_trip =
            serde_json::to_value(parsed).expect("ASIC settings fixture should serialize");

        // Assert
        assert_eq!(round_trip, original);
    }
}
