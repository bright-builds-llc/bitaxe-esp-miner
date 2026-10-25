//! The 601's shared observations: each safe-boot sweep's readings reduced and published, so
//! the API and every mining-safety check see the board (ADR-0041).
//!
//! Reference: `reference/esp-miner/main/power/power.c` and `main/thermal/thermal.c`.

use super::actuation::ActuationState;
use crate::operator_sensor_runtime::{new_boot_session_id, project_observations};
use crate::safety_adapter::gamma601::ThermalAcquisitions;
use bitaxe_safety::core_voltage_acquisition::CoreVoltageProducerState;
use bitaxe_safety::gamma601_diode::{
    chip_temperature_outcome, chip_unloaded, next_stage, DiodeStage, UninitializedWindow,
};
use bitaxe_safety::gamma601_safe_boot::SafeBootReport;
use bitaxe_safety::gamma601_sensors::{self, RegulatorReadings};
use bitaxe_safety::observation::{BootSessionId, MonotonicMillis};
use bitaxe_safety::sensor_acquisition::{
    reduce_tps546_sensor_sweep, ProducerSensorState, ProducerSequences, SensorSweepOutcomes,
};
use bitaxe_safety::tps546::StatusWord;

/// A sample older than this is stale, as on the 205; the sweep runs at twice this rate.
const STALE_AFTER_MS: u64 = 1_000;
/// The power-fault diagnostic writes at most one line this often.
const POWER_FAULT_LOG_INTERVAL_MS: u64 = 10_000;

/// The producer's own truth between sweeps.
pub(super) struct Producer {
    boot_session: BootSessionId,
    state: ProducerSensorState,
    sequences: ProducerSequences,
    core_voltage: CoreVoltageProducerState,
    /// Whether the BM1370's diode reading counts (ADR-0041 item 11). Boot starts powered but
    /// uninitialized, the cautious stage, until the safe boot confirms the output off.
    diode_stage: DiodeStage,
    uninitialized: UninitializedWindow,
    /// The power fact's last fault cause, so each change is logged once, and when the last
    /// line was written, so a flapping cause cannot flood the retained log.
    maybe_power_fault: Option<&'static str>,
    maybe_power_fault_logged_ms: Option<u64>,
}

impl Producer {
    pub(super) fn new() -> Self {
        Self {
            boot_session: new_boot_session_id(),
            state: ProducerSensorState::default(),
            sequences: ProducerSequences::default(),
            core_voltage: CoreVoltageProducerState::default(),
            diode_stage: DiodeStage::PoweredUninitialized,
            uninitialized: UninitializedWindow::default(),
            maybe_power_fault: None,
            maybe_power_fault_logged_ms: None,
        }
    }

    /// Reduces the sweep just read into the observations and publishes them.
    // Kept a distinct frame so the build's stack audit can bound it (`firmware-stack-budget.ts`).
    #[inline(never)]
    /// `state` says whether the regulator is deliberately on (so its `OFF` bit is a fault) and
    /// for which generation, which decides whether the chip's diode reading counts.
    pub(super) fn publish(
        &mut self,
        report: &SafeBootReport,
        thermal: ThermalAcquisitions,
        state: &ActuationState,
    ) {
        let entry = crate::board_identity::catalog_entry();
        let init_matches_generation = state.powered_generation().is_some_and(|generation| {
            crate::asic_adapter::production::bm1370_initialized_for(generation)
        });
        self.diode_stage = next_stage(
            self.diode_stage,
            report.off_confirmed(),
            state.regulator_on,
            init_matches_generation,
        );
        let readings = RegulatorReadings {
            maybe_vin_volts: report.maybe_vin_volts,
            maybe_iout_amps: report.maybe_iout_amps,
            maybe_vout_volts: report.maybe_vout_volts,
            maybe_temperature_celsius: report.maybe_vr_temperature_celsius,
        };
        let outcomes = SensorSweepOutcomes {
            power: gamma601_sensors::power_outcome(
                readings,
                chip_unloaded(self.diode_stage, report.off_confirmed(), state.regulator_on),
                f64::from(entry.family_facts().power_offset_watts),
            ),
            asic_temperature_celsius: chip_temperature_outcome(
                self.diode_stage,
                state.regulator_on,
                thermal.chip_temperature,
            ),
            vr_temperature_celsius: gamma601_sensors::vr_temperature_outcome(readings),
            tachometer_rpm: thermal.tachometer,
        };
        let acquired_at = MonotonicMillis::new(crate::runtime_uptime::millis());
        // The init window is a run's: it counts only while the regulator is commanded on, so a
        // boot whose off is slow to confirm is not called an overdue init.
        let window_stage = if state.regulator_on {
            self.diode_stage
        } else {
            DiodeStage::Unpowered
        };
        let maybe_overdue = self.uninitialized.observe(window_stage, acquired_at.get());
        let maybe_fault = gamma601_sensors::regulator_fault(
            report.maybe_status.map(StatusWord),
            state.output_verified_on(),
        )
        .or(maybe_overdue);
        self.note_power_fault(maybe_fault, outcomes.power, report);
        match reduce_tps546_sensor_sweep(
            self.state,
            self.sequences,
            outcomes,
            self.boot_session,
            acquired_at,
            f64::from(entry.power_consumption_target()),
            maybe_fault,
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

    /// Logs each change of the power fact's fault cause once, with the raw regulator readings,
    /// so a revocation on the power fact names its cause in the retained log.
    fn note_power_fault(
        &mut self,
        maybe_fault: Option<bitaxe_safety::observation::FaultReason>,
        power: bitaxe_safety::sensor_acquisition::AcquisitionOutcome<
            bitaxe_safety::power::Ina260RawSample,
        >,
        report: &SafeBootReport,
    ) {
        use bitaxe_safety::sensor_acquisition::AcquisitionOutcome;
        let maybe_cause = maybe_fault.map(|fault| fault.as_str()).or(match power {
            AcquisitionOutcome::InvalidSample => Some("power_reading_invalid"),
            AcquisitionOutcome::ReadFailed => Some("power_read_failed"),
            _ => None,
        });
        if maybe_cause == self.maybe_power_fault {
            return;
        }
        self.maybe_power_fault = maybe_cause;
        let now_ms = crate::runtime_uptime::millis();
        if self
            .maybe_power_fault_logged_ms
            .is_some_and(|logged_ms| now_ms.saturating_sub(logged_ms) < POWER_FAULT_LOG_INTERVAL_MS)
        {
            return;
        }
        self.maybe_power_fault_logged_ms = Some(now_ms);
        let milli = |maybe: Option<f64>| {
            maybe.map_or_else(
                || "unread".to_owned(),
                |value| format!("{:.0}", value * 1000.0),
            )
        };
        let line = format!(
            "gamma601_observations=power_fault cause={} iout_ma={} vout_mv={} status={} stage={:?}",
            maybe_cause.unwrap_or("none"),
            milli(report.maybe_iout_amps),
            milli(report.maybe_vout_volts),
            report
                .maybe_status
                .map_or_else(|| "unread".to_owned(), |status| format!("0x{status:04x}")),
            self.diode_stage,
        );
        log::warn!("{line}");
        crate::log_buffer::append_runtime_log_line(&line);
    }
}
