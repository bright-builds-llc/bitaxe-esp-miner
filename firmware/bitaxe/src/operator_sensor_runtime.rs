//! Sole normal-runtime producer for bounded read-only operator sensor truth.

use std::{thread, time::Duration};

use anyhow::{Context, Result};
use bitaxe_api::{project_observation, TelemetryObservations};
use bitaxe_core::{
    runtime_orchestration::{PeriodicDeadline, OPERATOR_OBSERVATION_CADENCE_MS},
    StartupDebugText,
};
use bitaxe_safety::{
    core_voltage_acquisition::CoreVoltageProducerState,
    observation::{BootSessionId, MonotonicMillis, UnavailableReason},
    sensor_acquisition::{
        reduce_sensor_sweep, AcquisitionOutcome, ProducerSensorState, ProducerSequences,
        SensorSweepOutcomes,
    },
};
use esp_idf_svc::sys;

use crate::display_adapter::RuntimeDisplayOwner;
use crate::safety_adapter::{
    self, RuntimeI2cOwner, SafetyActuationOwnerInbox, SafetyActuationOwnerWait,
};

pub const SENSOR_SWEEP_CADENCE_MS: u64 = OPERATOR_OBSERVATION_CADENCE_MS;
pub const DISPLAY_REFRESH_CADENCE_MS: u64 = 1_000;
const SENSOR_STALE_AFTER_MS: u64 = 1_000;
const BOARD_POWER_TARGET_WATTS: f64 = 12.0;
const PRODUCER_THREAD_NAME: &str = "operator-sensors";
const PRODUCER_THREAD_STACK_BYTES: usize = 8 * 1024;

pub fn start(
    maybe_owner: Option<RuntimeI2cOwner<'static>>,
    maybe_core_voltage_adc: Option<safety_adapter::Ultra205CoreVoltageAdc>,
    maybe_display: Option<(RuntimeDisplayOwner, StartupDebugText)>,
) -> Result<()> {
    let (maybe_actuation_registration, maybe_actuation_inbox) = if maybe_owner.is_some() {
        let (registration, inbox) = safety_adapter::prepare_safety_actuation_owner();
        (Some(registration), Some(inbox))
    } else {
        (None, None)
    };
    thread::Builder::new()
        .name(PRODUCER_THREAD_NAME.to_owned())
        .stack_size(PRODUCER_THREAD_STACK_BYTES)
        .spawn(move || {
            run(
                maybe_owner,
                maybe_core_voltage_adc,
                maybe_display,
                maybe_actuation_inbox,
            )
        })
        .context("spawn operator sensor producer")?;
    if let Some(actuation_registration) = maybe_actuation_registration {
        safety_adapter::publish_safety_actuation_owner(actuation_registration)
            .context("publish operator sensor actuation owner")?;
    }
    log::info!(
        "operator_sensor_runtime=started cadence_ms={SENSOR_SWEEP_CADENCE_MS} display_refresh_ms={DISPLAY_REFRESH_CADENCE_MS}"
    );
    Ok(())
}

fn run(
    mut maybe_owner: Option<RuntimeI2cOwner<'static>>,
    mut maybe_core_voltage_adc: Option<safety_adapter::Ultra205CoreVoltageAdc>,
    maybe_display: Option<(RuntimeDisplayOwner, StartupDebugText)>,
    maybe_actuation_inbox: Option<SafetyActuationOwnerInbox>,
) -> ! {
    let boot_session = new_boot_session_id();
    let mut state = ProducerSensorState::default();
    let mut core_voltage_state = CoreVoltageProducerState::default();
    let mut sequences = ProducerSequences::default();
    let started_at_ms = crate::runtime_uptime::millis();
    let mut sensor_schedule = PeriodicDeadline::new(started_at_ms, SENSOR_SWEEP_CADENCE_MS)
        .expect("operator observation cadence is nonzero");
    let mut maybe_display = maybe_display.map(|(owner, text)| RuntimeDisplay {
        owner,
        last_alternating_line: text.frame_at(0).lines()[2].to_owned(),
        text,
    });
    let mut maybe_display_schedule = maybe_display.as_ref().map(|_| {
        let first_deadline_ms = started_at_ms
            .checked_add(DISPLAY_REFRESH_CADENCE_MS)
            .expect("runtime display deadline is representable");
        PeriodicDeadline::new(first_deadline_ms, DISPLAY_REFRESH_CADENCE_MS)
            .expect("runtime display cadence is nonzero")
    });

    loop {
        let now_ms = crate::runtime_uptime::millis();

        if sensor_schedule.is_due(now_ms) {
            let (power, asic_temperature_celsius, tachometer_rpm) =
                if let Some(owner) = maybe_owner.as_mut() {
                    (
                        safety_adapter::read_power_acquisition(owner),
                        safety_adapter::read_asic_temperature_acquisition(owner),
                        safety_adapter::read_tachometer_acquisition(owner),
                    )
                } else {
                    (
                        AcquisitionOutcome::Unavailable(UnavailableReason::ProducerUnavailable),
                        AcquisitionOutcome::Unavailable(UnavailableReason::ProducerUnavailable),
                        AcquisitionOutcome::Unavailable(UnavailableReason::ProducerUnavailable),
                    )
                };
            let vr_temperature_celsius =
                AcquisitionOutcome::Unavailable(UnavailableReason::UnsupportedOnBoard);
            let core_voltage_millivolts = maybe_core_voltage_adc.as_mut().map_or(
                AcquisitionOutcome::Unavailable(UnavailableReason::CoreVoltageUnavailable),
                safety_adapter::read_core_voltage_acquisition,
            );
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

            match core_voltage_state.record(core_voltage_millivolts, boot_session, acquired_at) {
                Ok(next_state) => {
                    core_voltage_state =
                        next_state.mark_stale_at(acquired_at, SENSOR_STALE_AFTER_MS);
                }
                Err(_) => {
                    log::warn!("core_voltage_adc=fault category=sequence_overflow");
                }
            }

            safety_adapter::replace_observations_from_producer(project_observations(
                state,
                core_voltage_state,
            ));
            let advance = match sensor_schedule.advance_past(crate::runtime_uptime::millis()) {
                Ok(advance) => advance,
                Err(_) => {
                    log::error!(
                        "operator_sensor_runtime=fault category=deadline_overflow action=halt"
                    );
                    park_forever();
                }
            };
            if advance.missed_slots() > 0 {
                log::warn!(
                    "operator_sensor_runtime=overrun category=deadline_missed slots={}",
                    advance.missed_slots()
                );
            }
        }

        if maybe_display_schedule.is_some_and(|schedule| schedule.is_due(now_ms)) {
            if let Some(owner) = maybe_owner.as_mut() {
                service_display(owner, &mut maybe_display, now_ms);
            }
            if maybe_display.is_none() {
                maybe_display_schedule = None;
            }
            if let Some(schedule) = maybe_display_schedule.as_mut() {
                match schedule.advance_past(crate::runtime_uptime::millis()) {
                    Ok(advance) if advance.missed_slots() > 0 => log::warn!(
                        "display_runtime=overrun category=deadline_missed slots={}",
                        advance.missed_slots()
                    ),
                    Ok(_) => {}
                    Err(_) => {
                        log::warn!(
                            "display_status=runtime_refresh_disabled reason=deadline_overflow"
                        );
                        maybe_display = None;
                        maybe_display_schedule = None;
                    }
                }
            }
        }

        let next_display_deadline_ms =
            maybe_display_schedule.map_or(u64::MAX, PeriodicDeadline::next_deadline_ms);
        let next_owner_deadline_ms = sensor_schedule
            .next_deadline_ms()
            .min(next_display_deadline_ms);
        let wait = duration_until(next_owner_deadline_ms);
        match (maybe_owner.as_mut(), maybe_actuation_inbox.as_ref()) {
            (Some(owner), Some(actuation_inbox)) => {
                if safety_adapter::service_next_safety_actuation_request(
                    owner,
                    actuation_inbox,
                    wait,
                ) == SafetyActuationOwnerWait::Disconnected
                {
                    sleep_until(next_owner_deadline_ms);
                }
            }
            _ => sleep_until(next_owner_deadline_ms),
        }
    }
}

struct RuntimeDisplay {
    owner: RuntimeDisplayOwner,
    text: StartupDebugText,
    last_alternating_line: String,
}

fn service_display(
    owner: &mut RuntimeI2cOwner<'_>,
    maybe_display: &mut Option<RuntimeDisplay>,
    uptime_ms: u64,
) {
    let Some(display) = maybe_display.as_mut() else {
        return;
    };
    if let Err(error) = display.owner.service_power(owner, uptime_ms, false) {
        disable_runtime_display(maybe_display, "power_command_failed", &error);
        return;
    }

    let frame = display.text.frame_at(uptime_ms);
    let alternating_line = frame.lines()[2];
    if display.last_alternating_line == alternating_line {
        return;
    }

    if let Err(error) = display.owner.render_runtime_debug_text(owner, &frame) {
        disable_runtime_display(maybe_display, "render_failed", &error);
        return;
    }
    display.last_alternating_line = alternating_line.to_owned();
}

fn disable_runtime_display(
    maybe_display: &mut Option<RuntimeDisplay>,
    reason: &str,
    error: &anyhow::Error,
) {
    log::warn!("display_status=runtime_refresh_disabled reason={reason} error={error:#}");
    crate::display_adapter::publish_runtime_display_input_boundary(
        crate::display_adapter::RuntimeDisplayMode::Unavailable,
    );
    *maybe_display = None;
}

fn project_observations(
    state: ProducerSensorState,
    core_voltage_state: CoreVoltageProducerState,
) -> TelemetryObservations {
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
        core_voltage_actual_mv: project_observation(
            core_voltage_state.observation(),
            |millivolts| Some(f64::from(*millivolts)),
            UnavailableReason::CoreVoltageUnavailable,
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

fn sleep_until(deadline_ms: u64) {
    thread::sleep(duration_until(deadline_ms));
}

fn park_forever() -> ! {
    loop {
        thread::park();
    }
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
