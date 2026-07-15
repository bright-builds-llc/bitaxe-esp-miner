//! Adapter input boundary for AxeOS API responses.
//!
//! This module intentionally contains no ESP-IDF imports. Firmware adapters
//! collect platform facts, while pure `bitaxe-api` code maps the snapshot into
//! handwritten AxeOS wire DTOs.

use bitaxe_asic::bm1366::observation::AsicInitStatus;
use bitaxe_config::{
    ultra_205_catalog_entry, ultra_205_defaults, BoardCatalogEntry, Ultra205Defaults,
};
use bitaxe_safety::evidence::SafetyCriticalEvidence;
use bitaxe_safety::observation::{Observation, UnavailableReason};
use bitaxe_stratum::v1::state::MiningRuntimeState;

use crate::{
    BlockFoundNotificationState, ObservationReasonWire, ObservationStateWire, ObservationTruthWire,
    TelemetryObservations,
};

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
    /// wiring. Hardware-control telemetry is explicit unavailable status until
    /// live voltage, fan, thermal, and power evidence exists.
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
            safe_telemetry: SafeTelemetrySnapshot::unavailable("safety_telemetry_unavailable"),
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
    pub semantic_version: String,
    pub source_commit: String,
    pub reference_commit: String,
    pub app_elf_sha256: String,
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
}

impl PlatformSnapshot {
    /// Returns synthetic-safe platform values that avoid secrets and live
    /// hardware claims while keeping the upstream field contract populated.
    #[must_use]
    pub fn safe_ultra_205() -> Self {
        Self {
            version: "000000000000-dev".to_owned(),
            semantic_version: "0.0.0-safe".to_owned(),
            source_commit: "0".repeat(40),
            reference_commit: "0".repeat(40),
            app_elf_sha256: "0".repeat(64),
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
        let all_fresh = observations.power_watts.is_fresh()
            && observations.bus_voltage_volts.is_fresh()
            && observations.current_amps.is_fresh()
            && observations.chip_temp_celsius.is_fresh()
            && observations.vr_temp_celsius.is_fresh()
            && observations.fan_rpm.is_fresh();

        Self {
            status: if all_fresh {
                SafetyTelemetryStatus::Fresh
            } else {
                SafetyTelemetryStatus::Unavailable {
                    reason: "observation_truth_not_all_fresh",
                }
            },
            evidence: SafetyCriticalEvidence::Missing,
            power_watts: fresh_f64(&observations.power_watts),
            voltage_volts: fresh_f64(&observations.bus_voltage_volts),
            current_amps: fresh_f64(&observations.current_amps),
            chip_temp_celsius: fresh_f64(&observations.chip_temp_celsius),
            vr_temp_celsius: fresh_f64(&observations.vr_temp_celsius),
            fan_rpm: fresh_u16(&observations.fan_rpm),
            power_status: (&observations.power_watts).into(),
            voltage_status: (&observations.bus_voltage_volts).into(),
            current_status: (&observations.current_amps).into(),
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
mod tests {
    use bitaxe_safety::evidence::SafetyCriticalEvidence;
    use bitaxe_safety::observation::{
        BootSessionId, FaultReason, MonotonicMillis, Observation, ObservationSequence, StaleReason,
        UnavailableReason,
    };

    use crate::{
        ApiSnapshot, ConfigSnapshot, ObservationStateWire, SafeTelemetrySnapshot,
        SafetyTelemetryReport, SafetyTelemetryStatus, TelemetryObservations,
    };

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

    #[test]
    fn safety_telemetry_model_safe_ultra_205_is_explicit_unavailable() {
        // Arrange
        let snapshot = ApiSnapshot::safe_ultra_205();

        // Act
        let telemetry = snapshot.safe_telemetry;

        // Assert
        assert_eq!(
            telemetry.status,
            SafetyTelemetryStatus::Unavailable {
                reason: "safety_telemetry_unavailable"
            }
        );
        assert_eq!(telemetry.evidence, SafetyCriticalEvidence::Missing);
        assert_eq!(telemetry.power_watts, 0.0);
        assert_eq!(telemetry.fan_rpm, 0);
    }

    #[test]
    fn safety_telemetry_model_fresh_legacy_report_cannot_publish_unstamped_values() {
        // Arrange
        let report = fresh_report(SafetyCriticalEvidence::hardware_smoke(
            "phase-06-api-telemetry-smoke",
        ));

        // Act
        let snapshot = SafeTelemetrySnapshot::from_report(report);

        // Assert
        assert_eq!(
            snapshot.status,
            SafetyTelemetryStatus::Unavailable {
                reason: "legacy_telemetry_unstamped"
            }
        );
        assert_eq!(snapshot.evidence, report.evidence);
        assert_eq!(snapshot.power_watts, 0.0);
        assert_eq!(snapshot.voltage_volts, 0.0);
        assert_eq!(snapshot.current_amps, 0.0);
        assert_eq!(snapshot.chip_temp_celsius, 0.0);
        assert_eq!(snapshot.fan_rpm, 0);
        assert_eq!(
            snapshot.power_status.state,
            ObservationStateWire::Unavailable
        );
    }

    #[test]
    fn safety_telemetry_projection_compatibility_zero_does_not_authenticate_truth() {
        // Arrange
        let unavailable = TelemetryObservations::default();
        let fresh_zero = TelemetryObservations {
            power_watts: fresh_f64_observation(0.0, 1),
            ..TelemetryObservations::default()
        };

        // Act
        let unavailable_projection = SafeTelemetrySnapshot::from_observations(&unavailable);
        let fresh_projection = SafeTelemetrySnapshot::from_observations(&fresh_zero);

        // Assert
        assert_eq!(unavailable_projection.power_watts, 0.0);
        assert_eq!(fresh_projection.power_watts, 0.0);
        assert_eq!(
            unavailable_projection.power_status.state,
            ObservationStateWire::Unavailable
        );
        assert_eq!(
            fresh_projection.power_status.state,
            ObservationStateWire::Fresh
        );
    }

    #[test]
    fn safety_telemetry_projection_preserves_mixed_independent_states() {
        // Arrange
        let fresh_power = fresh_f64_observation(10.0, 1);
        let stale_voltage = fresh_f64_observation(5.0, 2)
            .mark_stale(StaleReason::PowerSampleStale)
            .expect("fresh voltage can become stale");
        let unavailable_current =
            Observation::unavailable(UnavailableReason::PowerSampleUnavailable);
        let fault_temperature =
            Observation::<f64>::unavailable(UnavailableReason::ThermalReadingUnavailable)
                .record_fault(FaultReason::ThermalReadingInvalid);
        let fresh_vr = fresh_f64_observation(42.0, 3);
        let fresh_fan = fresh_u16_observation(3_200, 4);
        let observations = TelemetryObservations {
            power_watts: fresh_power,
            bus_voltage_volts: stale_voltage,
            current_amps: unavailable_current,
            chip_temp_celsius: fault_temperature,
            vr_temp_celsius: fresh_vr,
            fan_rpm: fresh_fan,
        };

        // Act
        let projection = SafeTelemetrySnapshot::from_observations(&observations);

        // Assert
        assert_eq!(projection.power_status.state, ObservationStateWire::Fresh);
        assert_eq!(projection.voltage_status.state, ObservationStateWire::Stale);
        assert_eq!(
            projection.current_status.state,
            ObservationStateWire::Unavailable
        );
        assert_eq!(
            projection.chip_temp_status.state,
            ObservationStateWire::Fault
        );
        assert_eq!(projection.vr_temp_status.state, ObservationStateWire::Fresh);
        assert_eq!(projection.fan_rpm_status.state, ObservationStateWire::Fresh);
        assert_eq!(projection.power_watts, 10.0);
        assert_eq!(projection.voltage_volts, 0.0);
        assert_eq!(projection.vr_temp_celsius, 42.0);
        assert_eq!(projection.fan_rpm, 3_200);
    }

    #[test]
    fn safety_telemetry_model_d17_stale_fault_unavailable_zero_numeric_projection() {
        // Arrange
        let stale = SafetyTelemetryReport {
            status: SafetyTelemetryStatus::Stale {
                reason: "power_sample_stale",
            },
            ..fresh_report(SafetyCriticalEvidence::hardware_smoke(
                "phase-06-api-telemetry-smoke",
            ))
        };
        let fault = SafetyTelemetryReport {
            status: SafetyTelemetryStatus::Fault {
                reason: "thermal_reading_invalid",
            },
            ..fresh_report(SafetyCriticalEvidence::hardware_smoke(
                "phase-06-api-telemetry-smoke",
            ))
        };
        let unavailable = SafetyTelemetryReport {
            status: SafetyTelemetryStatus::Unavailable {
                reason: "safety_telemetry_unavailable",
            },
            ..fresh_report(SafetyCriticalEvidence::Missing)
        };

        // Act
        let projections = [
            SafeTelemetrySnapshot::from_report(stale),
            SafeTelemetrySnapshot::from_report(fault),
            SafeTelemetrySnapshot::from_report(unavailable),
        ];

        // Assert
        for projection in projections {
            assert_eq!(projection.power_watts, 0.0);
            assert_eq!(projection.voltage_volts, 0.0);
            assert_eq!(projection.current_amps, 0.0);
            assert_eq!(projection.fan_rpm, 0);
            assert_ne!(projection.status, SafetyTelemetryStatus::Fresh);
        }
    }

    #[test]
    fn safety_telemetry_model_d18_fresh_unit_evidence_does_not_claim_hardware_values() {
        // Arrange
        let report = fresh_report(SafetyCriticalEvidence::implemented_not_verified("unit"));

        // Act
        let snapshot = SafeTelemetrySnapshot::from_report(report);

        // Assert
        assert_eq!(
            snapshot.status,
            SafetyTelemetryStatus::Unavailable {
                reason: "legacy_telemetry_unstamped"
            }
        );
        assert_eq!(snapshot.evidence, report.evidence);
        assert_eq!(snapshot.power_watts, 0.0);
        assert_eq!(snapshot.chip_temp_celsius, 0.0);
    }

    fn fresh_report(evidence: SafetyCriticalEvidence) -> SafetyTelemetryReport {
        SafetyTelemetryReport {
            status: SafetyTelemetryStatus::Fresh,
            evidence,
            power_watts: 11.5,
            voltage_volts: 5.1,
            current_amps: 2.25,
            chip_temp_celsius: 56.0,
            chip_temp2_celsius: 57.0,
            vr_temp_celsius: 45.0,
            core_voltage_actual_mv: 1_198.0,
            actual_frequency_mhz: 485.0,
            expected_hashrate_ghs: 525.0,
            fan_speed_percent: 70,
            fan_rpm: 3_200,
            fan2_rpm: 0,
            wifi_rssi_dbm: -50,
        }
    }

    fn fresh_f64_observation(value: f64, prior_sequence: u64) -> Observation<f64> {
        Observation::record_success(
            value,
            BootSessionId::new(7),
            ObservationSequence::new(prior_sequence),
            MonotonicMillis::new(250),
        )
        .expect("fixture sequence should advance")
        .0
    }

    fn fresh_u16_observation(value: u16, prior_sequence: u64) -> Observation<u16> {
        Observation::record_success(
            value,
            BootSessionId::new(7),
            ObservationSequence::new(prior_sequence),
            MonotonicMillis::new(250),
        )
        .expect("fixture sequence should advance")
        .0
    }
}
