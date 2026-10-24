//! The 601's shared observations: each safe-boot sweep's readings reduced and published, so
//! the API and every mining-safety check see the board (ADR-0041).
//!
//! Reference: `reference/esp-miner/main/power/power.c` and `main/thermal/thermal.c`.

use crate::operator_sensor_runtime::{new_boot_session_id, project_observations};
use crate::safety_adapter::gamma601::ThermalAcquisitions;
use bitaxe_safety::core_voltage_acquisition::CoreVoltageProducerState;
use bitaxe_safety::gamma601_safe_boot::SafeBootReport;
use bitaxe_safety::gamma601_sensors::{self, RegulatorReadings};
use bitaxe_safety::observation::{BootSessionId, MonotonicMillis};
use bitaxe_safety::sensor_acquisition::{
    reduce_tps546_sensor_sweep, ProducerSensorState, ProducerSequences, SensorSweepOutcomes,
};
use bitaxe_safety::tps546::StatusWord;

/// A sample older than this is stale, as on the 205; the sweep runs at twice this rate.
const STALE_AFTER_MS: u64 = 1_000;

/// The producer's own truth between sweeps.
pub(super) struct Producer {
    boot_session: BootSessionId,
    state: ProducerSensorState,
    sequences: ProducerSequences,
    core_voltage: CoreVoltageProducerState,
}

impl Producer {
    pub(super) fn new() -> Self {
        Self {
            boot_session: new_boot_session_id(),
            state: ProducerSensorState::default(),
            sequences: ProducerSequences::default(),
            core_voltage: CoreVoltageProducerState::default(),
        }
    }

    /// Reduces the sweep just read into the observations and publishes them.
    // Kept a distinct frame so the build's stack audit can bound it (`firmware-stack-budget.ts`).
    #[inline(never)]
    pub(super) fn publish(&mut self, report: &SafeBootReport, thermal: ThermalAcquisitions) {
        let entry = crate::board_identity::catalog_entry();
        let readings = RegulatorReadings {
            maybe_vin_volts: report.maybe_vin_volts,
            maybe_iout_amps: report.maybe_iout_amps,
            maybe_vout_volts: report.maybe_vout_volts,
            maybe_temperature_celsius: report.maybe_vr_temperature_celsius,
        };
        let outcomes = SensorSweepOutcomes {
            power: gamma601_sensors::power_outcome(
                readings,
                report.off_confirmed(),
                f64::from(entry.family_facts().power_offset_watts),
            ),
            asic_temperature_celsius: thermal.chip_temperature,
            vr_temperature_celsius: gamma601_sensors::vr_temperature_outcome(readings),
            tachometer_rpm: thermal.tachometer,
        };
        let acquired_at = MonotonicMillis::new(crate::runtime_uptime::millis());
        match reduce_tps546_sensor_sweep(
            self.state,
            self.sequences,
            outcomes,
            self.boot_session,
            acquired_at,
            f64::from(entry.power_consumption_target()),
            // The safe boot keeps the output off, so `OFF` is not a fault here.
            gamma601_sensors::regulator_fault(report.maybe_status.map(StatusWord), false),
        ) {
            Ok((state, sequences)) => {
                self.state = state.mark_stale_at(acquired_at, STALE_AFTER_MS);
                self.sequences = sequences;
            }
            Err(_) => log::warn!("gamma601_observations=fault category=sequence_overflow"),
        }
        match self.core_voltage.record(
            gamma601_sensors::core_voltage_outcome(readings),
            self.boot_session,
            acquired_at,
        ) {
            Ok(core_voltage) => {
                self.core_voltage = core_voltage.mark_stale_at(acquired_at, STALE_AFTER_MS);
            }
            Err(_) => log::warn!("gamma601_observations=fault category=sequence_overflow"),
        }
        crate::safety_adapter::replace_observations_from_producer(project_observations(
            self.state,
            self.core_voltage,
        ));
    }
}
