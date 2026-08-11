//! Adapter input boundary for AxeOS API responses.
//!
//! This module intentionally contains no ESP-IDF imports. Firmware adapters
//! collect platform facts, while pure `bitaxe-api` code maps the snapshot into
//! handwritten AxeOS wire DTOs.

use bitaxe_asic::bm1366::observation::AsicInitStatus;
use bitaxe_config::{
    ultra_205_catalog_entry, ultra_205_defaults, BoardCatalogEntry, Ultra205Defaults,
};
use bitaxe_core::runtime_health::RuntimeHealthSnapshot;
use bitaxe_safety::evidence::SafetyCriticalEvidence;
use bitaxe_safety::observation::{Observation, UnavailableReason};
use bitaxe_stratum::v1::state::MiningRuntimeState;

use crate::{
    BlockFoundNotificationState, ObservationReasonWire, ObservationStateWire, ObservationTruthWire,
    OperatorSnapshotIdentity, PlatformIdentity, SystemInfoBlockSnapshot,
    SystemInfoSettingsSnapshot, TelemetryObservations,
};

/// Complete pure input snapshot for the initial AxeOS API contract slice.
#[derive(Debug, Clone, PartialEq)]
pub struct ApiSnapshot {
    /// Fixture construction uses a deterministic identity; firmware must replace it before projection.
    pub operator_snapshot_identity: OperatorSnapshotIdentity,
    /// Typed running-platform facts; fixture construction authenticates none of them.
    pub platform_identity: PlatformIdentity,
    /// Passive runtime-health facts captured with this operator snapshot.
    pub runtime_health: RuntimeHealthSnapshot,
    pub config: ConfigSnapshot,
    pub project_settings: ProjectSettingsSnapshot,
    /// Confirmed settings represented by the upstream system-info response.
    pub system_info_settings: SystemInfoSettingsSnapshot,
    pub catalog: BoardCatalogEntry,
    pub mining: MiningRuntimeState,
    pub block_found: BlockFoundNotificationState,
    /// Conditional block template facts; absent until a positive block height exists.
    pub maybe_block: Option<SystemInfoBlockSnapshot>,
    pub asic: AsicSnapshot,
    pub platform: PlatformSnapshot,
    pub safe_telemetry: SafeTelemetrySnapshot,
}

impl ApiSnapshot {
    /// Returns a safe Ultra 205 snapshot for contract tests and early firmware
    /// wiring. Hardware-control telemetry is explicit unavailable status until
    /// live voltage, fan, thermal, and power evidence exists.
    #[must_use]
    pub fn safe_ultra_205() -> Self {
        Self {
            operator_snapshot_identity: OperatorSnapshotIdentity::fixture_only(),
            platform_identity: PlatformIdentity::fixture_only(),
            runtime_health: RuntimeHealthSnapshot::fixture_unavailable(),
            config: ConfigSnapshot::ultra_205(),
            project_settings: ProjectSettingsSnapshot::default(),
            system_info_settings: SystemInfoSettingsSnapshot::safe_ultra_205(),
            catalog: ultra_205_catalog_entry(),
            mining: MiningRuntimeState::default(),
            block_found: BlockFoundNotificationState {
                block_found: 0,
                show_new_block: false,
            },
            maybe_block: None,
            asic: AsicSnapshot::chip_detect_only(),
            platform: PlatformSnapshot::safe_ultra_205(),
            safe_telemetry: SafeTelemetrySnapshot::unavailable("safety_telemetry_unavailable"),
        }
    }
}

/// Project-owned settings that intentionally do not extend the upstream schema.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectSettingsSnapshot {
    pub start_mining_on_boot: bool,
}

impl Default for ProjectSettingsSnapshot {
    fn default() -> Self {
        Self {
            start_mining_on_boot: true,
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
    pub max_power_watts: u16,
    pub nominal_voltage_volts: u16,
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
            max_power_watts: 25,
            nominal_voltage_volts: 5,
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
#[derive(Debug, Clone, PartialEq)]
pub struct PlatformSnapshot {
    pub boot_ordinal: u64,
    pub reset_reason_category: String,
    pub version: String,
    pub semantic_version: String,
    pub source_commit: String,
    pub reference_commit: String,
    pub app_elf_sha256: String,
    pub build_timestamp_utc: String,
    pub build_channel: String,
    pub source_dirty: bool,
    pub maybe_release_tag: Option<String>,
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
    pub cpu_usage_percent: f64,
    pub maybe_power_fault: Option<String>,
    pub maybe_hardware_fault: Option<String>,
}

impl PlatformSnapshot {
    /// Returns synthetic-safe platform values that avoid secrets and live
    /// hardware claims while keeping the upstream field contract populated.
    #[must_use]
    pub fn safe_ultra_205() -> Self {
        Self {
            boot_ordinal: 0,
            reset_reason_category: "unavailable".to_owned(),
            version: "000000000000-dev".to_owned(),
            semantic_version: "0.0.0-safe".to_owned(),
            source_commit: "0".repeat(40),
            reference_commit: "0".repeat(40),
            app_elf_sha256: "0".repeat(64),
            build_timestamp_utc: "Unavailable".to_owned(),
            build_channel: "dev".to_owned(),
            source_dirty: false,
            maybe_release_tag: None,
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
            cpu_usage_percent: 0.0,
            maybe_power_fault: None,
            maybe_hardware_fault: None,
        }
    }
}

/// Explicit status for Phase 6-owned hardware-control telemetry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SafetyTelemetryStatus {
    Fresh,
    Stale { reason: &'static str },
    Fault { reason: &'static str },
    Unavailable { reason: &'static str },
}

/// Adapter-owned safety telemetry before API numeric projection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SafetyTelemetryReport {
    pub status: SafetyTelemetryStatus,
    pub evidence: SafetyCriticalEvidence,
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

/// Explicit safe values for Phase 6-owned hardware-control telemetry.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SafeTelemetrySnapshot {
    pub status: SafetyTelemetryStatus,
    pub evidence: SafetyCriticalEvidence,
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
    pub power_status: ObservationTruthWire,
    pub voltage_status: ObservationTruthWire,
    pub current_status: ObservationTruthWire,
    pub core_voltage_status: ObservationTruthWire,
    pub chip_temp_status: ObservationTruthWire,
    pub vr_temp_status: ObservationTruthWire,
    pub fan_rpm_status: ObservationTruthWire,
}

impl SafeTelemetrySnapshot {
    /// Returns safe zero-compatible values with a visible unavailable reason.
    #[must_use]
    pub const fn unavailable(reason: &'static str) -> Self {
        Self {
            status: SafetyTelemetryStatus::Unavailable { reason },
            evidence: SafetyCriticalEvidence::Missing,
            ..Self::zero_compatible()
        }
    }

    /// Preserves legacy report status without treating unstamped values as
    /// operator observation truth.
    #[must_use]
    pub const fn from_report(report: SafetyTelemetryReport) -> Self {
        let mut snapshot = Self::zero_compatible();
        snapshot.status = if matches!(report.status, SafetyTelemetryStatus::Fresh) {
            SafetyTelemetryStatus::Unavailable {
                reason: "legacy_telemetry_unstamped",
            }
        } else {
            report.status
        };
        snapshot.evidence = report.evidence;
        snapshot
    }

    /// Returns the operator projection with compatibility numerics suppressed
    /// whenever their corresponding fact lacks fresh stamped truth.
    #[must_use]
    pub(crate) fn operator_projection(mut self) -> Self {
        if !is_fresh_stamped(self.power_status) {
            self.power_watts = 0.0;
        }
        if !is_fresh_stamped(self.voltage_status) {
            self.voltage_volts = 0.0;
        }
        if !is_fresh_stamped(self.current_status) {
            self.current_amps = 0.0;
        }
        if !is_fresh_stamped(self.core_voltage_status) {
            self.core_voltage_actual_mv = 0.0;
        }
        if !is_fresh_stamped(self.chip_temp_status) {
            self.chip_temp_celsius = 0.0;
        }
        if !is_fresh_stamped(self.vr_temp_status) {
            self.vr_temp_celsius = 0.0;
        }
        if !is_fresh_stamped(self.fan_rpm_status) {
            self.fan_rpm = 0;
        }

        self
    }

    /// Projects stored observation truth separately from numeric compatibility values.
    #[must_use]
    pub fn from_observations(observations: &TelemetryObservations) -> Self {
        let supported_facts_fresh = observations.power_watts.is_fresh()
            && observations.bus_voltage_volts.is_fresh()
            && observations.current_amps.is_fresh()
            && observations.core_voltage_actual_mv.is_fresh()
            && observations.chip_temp_celsius.is_fresh()
            && observations.fan_rpm.is_fresh();

        Self {
            status: if supported_facts_fresh {
                SafetyTelemetryStatus::Fresh
            } else {
                SafetyTelemetryStatus::Unavailable {
                    reason: "supported_observation_truth_not_all_fresh",
                }
            },
            evidence: SafetyCriticalEvidence::Missing,
            power_watts: fresh_f64(&observations.power_watts),
            voltage_volts: fresh_f64(&observations.bus_voltage_volts),
            current_amps: fresh_f64(&observations.current_amps),
            core_voltage_actual_mv: fresh_f64(&observations.core_voltage_actual_mv),
            chip_temp_celsius: fresh_f64(&observations.chip_temp_celsius),
            vr_temp_celsius: fresh_f64(&observations.vr_temp_celsius),
            fan_rpm: fresh_u16(&observations.fan_rpm),
            power_status: (&observations.power_watts).into(),
            voltage_status: (&observations.bus_voltage_volts).into(),
            current_status: (&observations.current_amps).into(),
            core_voltage_status: (&observations.core_voltage_actual_mv).into(),
            chip_temp_status: (&observations.chip_temp_celsius).into(),
            vr_temp_status: (&observations.vr_temp_celsius).into(),
            fan_rpm_status: (&observations.fan_rpm).into(),
            ..Self::zero_compatible()
        }
    }

    const fn zero_compatible() -> Self {
        Self {
            status: SafetyTelemetryStatus::Unavailable {
                reason: "safety_telemetry_unavailable",
            },
            evidence: SafetyCriticalEvidence::Missing,
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
            power_status: legacy_unavailable_truth(),
            voltage_status: legacy_unavailable_truth(),
            current_status: legacy_unavailable_truth(),
            core_voltage_status: legacy_unavailable_truth(),
            chip_temp_status: legacy_unavailable_truth(),
            vr_temp_status: legacy_unavailable_truth(),
            fan_rpm_status: legacy_unavailable_truth(),
        }
    }
}

fn is_fresh_stamped(truth: ObservationTruthWire) -> bool {
    matches!(truth.state, ObservationStateWire::Fresh) && truth.stamp.is_some()
}

const fn legacy_unavailable_truth() -> ObservationTruthWire {
    ObservationTruthWire {
        state: ObservationStateWire::Unavailable,
        stamp: None,
        reason: Some(ObservationReasonWire::Unavailable(
            UnavailableReason::ProducerUnavailable,
        )),
    }
}

fn fresh_f64(observation: &Observation<f64>) -> f64 {
    if !observation.is_fresh() {
        return 0.0;
    }

    observation
        .maybe_last_good()
        .map_or(0.0, |sample| *sample.value())
}

fn fresh_u16(observation: &Observation<u16>) -> u16 {
    if !observation.is_fresh() {
        return 0;
    }

    observation
        .maybe_last_good()
        .map_or(0, |sample| *sample.value())
}

#[cfg(test)]
mod tests;
