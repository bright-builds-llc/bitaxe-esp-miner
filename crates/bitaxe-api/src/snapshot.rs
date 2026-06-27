//! Adapter input boundary for AxeOS API responses.
//!
//! This module intentionally contains no ESP-IDF imports. Firmware adapters
//! collect platform facts, while pure `bitaxe-api` code maps the snapshot into
//! handwritten AxeOS wire DTOs.

use bitaxe_asic::bm1366::observation::AsicInitStatus;
use bitaxe_config::{
    ultra_205_catalog_entry, ultra_205_defaults, BoardCatalogEntry, Ultra205Defaults,
};
use bitaxe_stratum::v1::state::MiningRuntimeState;

use crate::BlockFoundNotificationState;

/// Complete pure input snapshot for the initial AxeOS API contract slice.
#[derive(Debug, Clone, PartialEq)]
pub struct ApiSnapshot {
    pub config: ConfigSnapshot,
    pub catalog: BoardCatalogEntry,
    pub mining: MiningRuntimeState,
    pub block_found: BlockFoundNotificationState,
    pub asic: AsicSnapshot,
    pub platform: PlatformSnapshot,
    pub safe_telemetry: SafeTelemetrySnapshot,
}

impl ApiSnapshot {
    /// Returns a safe Ultra 205 snapshot for contract tests and early firmware
    /// wiring. Hardware-control telemetry is deliberately zeroed until Phase 6
    /// owns live voltage, fan, thermal, and power evidence.
    #[must_use]
    pub fn safe_ultra_205() -> Self {
        Self {
            config: ConfigSnapshot::ultra_205(),
            catalog: ultra_205_catalog_entry(),
            mining: MiningRuntimeState::default(),
            block_found: BlockFoundNotificationState {
                block_found: 0,
                show_new_block: false,
            },
            asic: AsicSnapshot::chip_detect_only(),
            platform: PlatformSnapshot::safe_ultra_205(),
            safe_telemetry: SafeTelemetrySnapshot::unavailable_until_phase_6(),
        }
    }
}

/// Config facts that feed API DTOs without exposing the whole config crate as
/// the public wire contract.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConfigSnapshot {
    pub defaults: Ultra205Defaults,
    pub asic_frequency_mhz: f64,
    pub asic_voltage_mv: u16,
    pub auto_fan_speed: bool,
    pub manual_fan_speed: u16,
}

impl ConfigSnapshot {
    /// Returns the Ultra 205 defaults sourced from `config-205.cvs`.
    #[must_use]
    pub const fn ultra_205() -> Self {
        let defaults = ultra_205_defaults();

        Self {
            defaults,
            asic_frequency_mhz: defaults.asic_frequency_mhz() as f64,
            asic_voltage_mv: defaults.asic_voltage_mv(),
            auto_fan_speed: defaults.auto_fan_speed(),
            manual_fan_speed: defaults.manual_fan_speed(),
        }
    }
}

/// ASIC facts used by system and ASIC response DTOs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AsicSnapshot {
    pub init_status: AsicInitStatus,
    pub maybe_detected_chips: Option<u8>,
}

impl AsicSnapshot {
    /// Returns the initial safe status before Phase 6 hardware-control effects.
    #[must_use]
    pub const fn chip_detect_only() -> Self {
        Self {
            init_status: AsicInitStatus::ChipDetectOnly,
            maybe_detected_chips: Some(1),
        }
    }
}

/// Platform facts collected by firmware adapters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformSnapshot {
    pub version: String,
    pub axe_os_version: String,
    pub idf_version: String,
    pub reset_reason: String,
    pub running_partition: String,
    pub mac_addr: String,
    pub hostname: String,
    pub ssid: String,
    pub ipv4: String,
    pub ipv6: String,
    pub wifi_status: String,
    pub ap_enabled: bool,
    pub psram_available: bool,
    pub free_heap: u64,
    pub free_heap_internal: u64,
    pub free_heap_spiram: u64,
    pub min_free_heap: u64,
    pub max_alloc_heap: u64,
    pub uptime_seconds: u64,
}

impl PlatformSnapshot {
    /// Returns synthetic-safe platform values that avoid secrets and live
    /// hardware claims while keeping the upstream field contract populated.
    #[must_use]
    pub fn safe_ultra_205() -> Self {
        Self {
            version: "bitaxe-rust-safe".to_owned(),
            axe_os_version: "safe-fixture".to_owned(),
            idf_version: "v5.5.4".to_owned(),
            reset_reason: "Reset due to power-on event".to_owned(),
            running_partition: "factory".to_owned(),
            mac_addr: "00:00:00:00:00:00".to_owned(),
            hostname: ultra_205_defaults().hostname().to_owned(),
            ssid: String::new(),
            ipv4: "0.0.0.0".to_owned(),
            ipv6: String::new(),
            wifi_status: "disconnected".to_owned(),
            ap_enabled: false,
            psram_available: true,
            free_heap: 0,
            free_heap_internal: 0,
            free_heap_spiram: 0,
            min_free_heap: 0,
            max_alloc_heap: 0,
            uptime_seconds: 0,
        }
    }
}

/// Explicit safe values for Phase 6-owned hardware-control telemetry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SafeTelemetrySnapshot {
    pub power_watts: f64,
    pub voltage_volts: f64,
    pub current_amps: f64,
    pub chip_temp_celsius: f64,
    pub chip_temp2_celsius: f64,
    pub vr_temp_celsius: f64,
    pub core_voltage_actual_mv: f64,
    pub actual_frequency_mhz: f64,
    pub expected_hashrate_ghs: f64,
    pub fan_speed_percent: u16,
    pub fan_rpm: u16,
    pub fan2_rpm: u16,
    pub wifi_rssi_dbm: i16,
}

impl SafeTelemetrySnapshot {
    /// Returns safe zeroed values until Phase 6 records hardware evidence.
    #[must_use]
    pub const fn unavailable_until_phase_6() -> Self {
        Self {
            power_watts: 0.0,
            voltage_volts: 0.0,
            current_amps: 0.0,
            chip_temp_celsius: 0.0,
            chip_temp2_celsius: 0.0,
            vr_temp_celsius: 0.0,
            core_voltage_actual_mv: 0.0,
            actual_frequency_mhz: 0.0,
            expected_hashrate_ghs: 0.0,
            fan_speed_percent: 0,
            fan_rpm: 0,
            fan2_rpm: 0,
            wifi_rssi_dbm: -90,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{ApiSnapshot, ConfigSnapshot};

    #[test]
    fn api_snapshot_contains_typed_input_fields_without_platform_sdk_dependencies() {
        // Arrange
        let snapshot = ApiSnapshot::safe_ultra_205();

        // Act
        let config = snapshot.config;
        let catalog = snapshot.catalog;
        let mining = snapshot.mining;
        let block_found = snapshot.block_found;
        let asic = snapshot.asic;
        let platform = snapshot.platform;

        // Assert
        assert_eq!(config.defaults.asic_model(), "BM1366");
        assert_eq!(catalog.board_version(), "205");
        assert_eq!(mining.counters.accepted, 0);
        assert_eq!(block_found.block_found, 0);
        assert!(!block_found.show_new_block);
        assert_eq!(asic.maybe_detected_chips, Some(1));
        assert_eq!(platform.hostname, "bitaxe");
    }

    #[test]
    fn config_snapshot_uses_ultra_205_defaults() {
        // Arrange
        let config = ConfigSnapshot::ultra_205();

        // Act
        let defaults = config.defaults;

        // Assert
        assert_eq!(defaults.board_version(), "205");
        assert_eq!(defaults.asic_frequency_mhz(), 485);
        assert_eq!(defaults.asic_voltage_mv(), 1200);
    }
}
