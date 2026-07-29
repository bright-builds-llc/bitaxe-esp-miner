//! Sole normal-runtime producer for bounded read-only operator sensor truth.

use std::{thread, time::Duration};

use anyhow::{Context, Result};
use bitaxe_api::{project_observation, TelemetryObservations};
use bitaxe_core::StartupDebugText;
use bitaxe_safety::{
    observation::{BootSessionId, MonotonicMillis, UnavailableReason},
    sensor_acquisition::{
        reduce_sensor_sweep, ProducerSensorState, ProducerSequences, SensorSweepOutcomes,
    },
};
use esp_idf_svc::sys;

use crate::safety_adapter::{
    self, RuntimeI2cOwner, SafetyActuationOwnerInbox, SafetyActuationOwnerWait,
};

pub const SENSOR_SWEEP_CADENCE_MS: u64 = 500;
pub const DISPLAY_REFRESH_CADENCE_MS: u64 = 1_000;
const SENSOR_STALE_AFTER_MS: u64 = 1_000;
const BOARD_POWER_TARGET_WATTS: f64 = 12.0;
const PRODUCER_THREAD_NAME: &str = "operator-sensors";
const PRODUCER_THREAD_STACK_BYTES: usize = 8 * 1024;

pub fn start(
    owner: RuntimeI2cOwner<'static>,
    maybe_display_text: Option<StartupDebugText>,
) -> Result<()> {
    let (actuation_registration, actuation_inbox) =
        safety_adapter::prepare_safety_actuation_owner();
    thread::Builder::new()
        .name(PRODUCER_THREAD_NAME.to_owned())
        .stack_size(PRODUCER_THREAD_STACK_BYTES)
        .spawn(move || run(owner, maybe_display_text, actuation_inbox))
        .context("spawn operator sensor producer")?;
    safety_adapter::publish_safety_actuation_owner(actuation_registration)
        .context("publish operator sensor actuation owner")?;
    log::info!(
        "operator_sensor_runtime=started cadence_ms={SENSOR_SWEEP_CADENCE_MS} display_refresh_ms={DISPLAY_REFRESH_CADENCE_MS}"
    );
    Ok(())
}

fn run(
    mut owner: RuntimeI2cOwner<'static>,
    mut maybe_display_text: Option<StartupDebugText>,
    actuation_inbox: SafetyActuationOwnerInbox,
) -> ! {
    let boot_session = new_boot_session_id();
    let mut state = ProducerSensorState::default();
    let mut sequences = ProducerSequences::default();
    let mut next_deadline_ms = crate::runtime_uptime::millis();
    let mut next_display_deadline_ms = next_deadline_ms.saturating_add(DISPLAY_REFRESH_CADENCE_MS);
    let mut maybe_last_display_line = maybe_display_text
        .as_ref()
        .map(|text| text.frame_at(0).lines()[2].to_owned());

    loop {
        let now_ms = crate::runtime_uptime::millis();

        if now_ms >= next_deadline_ms {
            let power = safety_adapter::read_power_acquisition(&mut owner);
            let asic_temperature_celsius =
                safety_adapter::read_asic_temperature_acquisition(&mut owner);
            let vr_temperature_celsius =
                safety_adapter::read_vr_temperature_acquisition(&mut owner);
            let tachometer_rpm = safety_adapter::read_tachometer_acquisition(&mut owner);
            let acquired_at = MonotonicMillis::new(crate::runtime_uptime::millis());
            let outcomes = SensorSweepOutcomes {
                power,
                asic_temperature_celsius,
                vr_temperature_celsius,
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

        if now_ms >= next_display_deadline_ms {
            refresh_display(
                &mut owner,
                &mut maybe_display_text,
                &mut maybe_last_display_line,
                now_ms,
            );
            next_display_deadline_ms = now_ms.saturating_add(DISPLAY_REFRESH_CADENCE_MS);
        }

        let next_owner_deadline_ms = next_deadline_ms.min(next_display_deadline_ms);
        let wait = duration_until(next_owner_deadline_ms);
        if safety_adapter::service_next_safety_actuation_request(&mut owner, &actuation_inbox, wait)
            == SafetyActuationOwnerWait::Disconnected
        {
            sleep_until(next_owner_deadline_ms);
        }
    }
}

fn refresh_display(
    owner: &mut RuntimeI2cOwner<'_>,
    maybe_display_text: &mut Option<StartupDebugText>,
    maybe_last_display_line: &mut Option<String>,
    uptime_ms: u64,
) {
    let Some(display_text) = maybe_display_text.as_ref() else {
        return;
    };
    let frame = display_text.frame_at(uptime_ms);
    let alternating_line = frame.lines()[2];
    if maybe_last_display_line.as_deref() == Some(alternating_line) {
        return;
    }

    if let Err(error) = crate::display_adapter::render_runtime_debug_text(owner, &frame) {
        log::warn!("display_status=runtime_refresh_disabled reason=render_failed error={error:#}");
        crate::display_adapter::publish_runtime_display_input_boundary(
            crate::display_adapter::RuntimeDisplayMode::Unavailable,
        );
        *maybe_display_text = None;
        *maybe_last_display_line = None;
        return;
    }
    *maybe_last_display_line = Some(alternating_line.to_owned());
}

fn project_observations(state: ProducerSensorState) -> TelemetryObservations {
    let power = state.power().truth();
    let temperature = state.thermal().temperature_truth();
    let vr_temperature = state.vr_temperature();
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
        vr_temp_celsius: project_observation(
            vr_temperature,
            |reading| Some(*reading),
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
    thread::sleep(duration_until(deadline_ms));
}

fn duration_until(deadline_ms: u64) -> Duration {
    Duration::from_millis(deadline_ms.saturating_sub(crate::runtime_uptime::millis()))
}

fn new_boot_session_id() -> BootSessionId {
    // SAFETY: esp_random has no preconditions and returns one hardware RNG word per call.
    let high = u64::from(unsafe { sys::esp_random() });
    // SAFETY: esp_random has no preconditions and returns one hardware RNG word per call.
    let low = u64::from(unsafe { sys::esp_random() });
    BootSessionId::new((high << 32) | low)
}
