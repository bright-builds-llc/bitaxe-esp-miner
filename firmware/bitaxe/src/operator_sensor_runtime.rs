//! Sole normal-runtime producer for bounded read-only operator sensor truth.

use std::{thread, time::Duration};

use anyhow::{Context, Result};
use bitaxe_api::{project_observation, TelemetryObservations};
use bitaxe_safety::{
    observation::{BootSessionId, MonotonicMillis, UnavailableReason},
    sensor_acquisition::{
        reduce_sensor_sweep, ProducerSensorState, ProducerSequences, SensorSweepOutcomes,
    },
};
use esp_idf_svc::sys;

use crate::safety_adapter::{self, BitaxeI2cBus};

pub const SENSOR_SWEEP_CADENCE_MS: u64 = 500;
const SENSOR_STALE_AFTER_MS: u64 = 1_000;
const BOARD_POWER_TARGET_WATTS: f64 = 12.0;
const PRODUCER_THREAD_NAME: &str = "operator-sensors";
const PRODUCER_THREAD_STACK_BYTES: usize = 8 * 1024;

pub fn start(bus: BitaxeI2cBus<'static>) -> Result<()> {
    thread::Builder::new()
        .name(PRODUCER_THREAD_NAME.to_owned())
        .stack_size(PRODUCER_THREAD_STACK_BYTES)
        .spawn(move || run(bus))
        .context("spawn operator sensor producer")?;
    log::info!("operator_sensor_runtime=started cadence_ms={SENSOR_SWEEP_CADENCE_MS}");
    Ok(())
}

fn run(mut bus: BitaxeI2cBus<'static>) -> ! {
    let boot_session = new_boot_session_id();
    let mut state = ProducerSensorState::default();
    let mut sequences = ProducerSequences::default();
    let mut next_deadline_ms = crate::runtime_uptime::millis();

    loop {
        sleep_until(next_deadline_ms);

        let power = safety_adapter::read_power_acquisition(&mut bus);
        let temperature_celsius = safety_adapter::read_temperature_acquisition(&mut bus);
        let tachometer_rpm = safety_adapter::read_tachometer_acquisition(&mut bus);
        let acquired_at = MonotonicMillis::new(crate::runtime_uptime::millis());
        let outcomes = SensorSweepOutcomes {
            power,
            temperature_celsius,
            tachometer_rpm,
        };

        match reduce_sensor_sweep(
            state,
            sequences,
            outcomes,
            boot_session,
            acquired_at,
            BOARD_POWER_TARGET_WATTS,
        ) {
            Ok((next_state, next_sequences)) => {
                state = next_state.mark_stale_at(acquired_at, SENSOR_STALE_AFTER_MS);
                sequences = next_sequences;
            }
            Err(_) => {
                log::warn!("operator_sensor_runtime=fault category=sequence_overflow");
            }
        }

        safety_adapter::replace_observations_from_producer(project_observations(state));
        next_deadline_ms = next_future_deadline(next_deadline_ms);
    }
}

fn project_observations(state: ProducerSensorState) -> TelemetryObservations {
    let power = state.power().truth();
    let temperature = state.thermal().temperature_truth();
    let tachometer = state.thermal().tachometer_truth();

    TelemetryObservations {
        power_watts: project_observation(
            power,
            |reading| Some((*reading).power_watts()),
            UnavailableReason::PowerSampleUnavailable,
        ),
        bus_voltage_volts: project_observation(
            power,
            |reading| Some((*reading).bus_voltage_volts()),
            UnavailableReason::PowerSampleUnavailable,
        ),
        current_amps: project_observation(
            power,
            |reading| Some((*reading).current_amps()),
            UnavailableReason::PowerSampleUnavailable,
        ),
        chip_temp_celsius: project_observation(
            temperature,
            |reading| Some(reading.chip_temp_celsius),
            UnavailableReason::ThermalReadingUnavailable,
        ),
        vr_temp_celsius: bitaxe_safety::observation::Observation::unavailable(
            UnavailableReason::ThermalReadingUnavailable,
        ),
        fan_rpm: project_observation(
            tachometer,
            |reading| Some((*reading).rpm()),
            UnavailableReason::TachometerUnavailable,
        ),
    }
}

fn next_future_deadline(previous_deadline_ms: u64) -> u64 {
    let now_ms = crate::runtime_uptime::millis();
    let scheduled_ms = previous_deadline_ms.saturating_add(SENSOR_SWEEP_CADENCE_MS);
    if scheduled_ms > now_ms {
        return scheduled_ms;
    }

    let missed_slots = now_ms
        .saturating_sub(scheduled_ms)
        .saturating_div(SENSOR_SWEEP_CADENCE_MS)
        .saturating_add(1);
    log::warn!("operator_sensor_runtime=overrun category=deadline_missed slots={missed_slots}");
    scheduled_ms.saturating_add(missed_slots.saturating_mul(SENSOR_SWEEP_CADENCE_MS))
}

fn sleep_until(deadline_ms: u64) {
    let now_ms = crate::runtime_uptime::millis();
    if deadline_ms <= now_ms {
        return;
    }

    thread::sleep(Duration::from_millis(deadline_ms - now_ms));
}

fn new_boot_session_id() -> BootSessionId {
    // SAFETY: esp_random has no preconditions and returns one hardware RNG word per call.
    let high = u64::from(unsafe { sys::esp_random() });
    // SAFETY: esp_random has no preconditions and returns one hardware RNG word per call.
    let low = u64::from(unsafe { sys::esp_random() });
    BootSessionId::new((high << 32) | low)
}
