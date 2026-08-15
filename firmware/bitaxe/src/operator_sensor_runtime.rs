//! Sole normal-runtime producer for bounded read-only operator sensor truth.

use std::{thread, time::Duration};

use anyhow::{Context, Result};
use bitaxe_api::{
    project_observation, DisplayFrameKind, DisplayRenderOutcome, TelemetryObservations,
};
use bitaxe_core::{
    runtime_orchestration::{PeriodicDeadline, OPERATOR_OBSERVATION_CADENCE_MS},
    screen::{ScreenFlow, ScreenFrame, SCREEN_UPDATE_MS},
};
use bitaxe_safety::{
    core_voltage_acquisition::CoreVoltageProducerState,
    observation::{BootSessionId, MonotonicMillis, UnavailableReason},
    sensor_acquisition::{
        reduce_sensor_sweep, AcquisitionOutcome, ProducerSensorState, ProducerSequences,
        SensorSweepOutcomes,
    },
    thermal::ThermalReading,
    thermal_fault_stimulus::{ThermalFaultStimulus, THERMAL_FAULT_STIMULUS_SAMPLE_COUNT},
};
use esp_idf_svc::sys;

use crate::display_adapter::RuntimeDisplayOwner;
use crate::operator_sensor_diagnostics::{OperatorSensorOutcome, OperatorSensorStage};
use crate::safety_adapter::{
    self, RuntimeI2cOwner, SafetyActuationOwnerInbox, SafetyActuationOwnerWait,
};

pub const SENSOR_SWEEP_CADENCE_MS: u64 = OPERATOR_OBSERVATION_CADENCE_MS;
pub const DISPLAY_REFRESH_CADENCE_MS: u64 = SCREEN_UPDATE_MS;
const SENSOR_STALE_AFTER_MS: u64 = 1_000;
const SENSOR_PUBLISH_HEADROOM_MS: u64 = 100;
const BOARD_POWER_TARGET_WATTS: f64 = 12.0;
const PRODUCER_THREAD_NAME: &str = "operator-sensors";
const PRODUCER_THREAD_STACK_BYTES: usize = 8 * 1024;

pub fn start(
    maybe_owner: Option<RuntimeI2cOwner<'static>>,
    maybe_core_voltage_adc: Option<safety_adapter::Ultra205CoreVoltageAdc>,
    maybe_display: Option<RuntimeDisplayOwner>,
    maybe_thermal_fault_admission: Option<crate::settings_adapter::ThermalFaultStimulusAdmission>,
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
                admitted_thermal_fault_stimulus(maybe_thermal_fault_admission),
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
    maybe_display: Option<RuntimeDisplayOwner>,
    maybe_actuation_inbox: Option<SafetyActuationOwnerInbox>,
    mut maybe_thermal_fault_stimulus: Option<ThermalFaultStimulus>,
) -> ! {
    let boot_session = new_boot_session_id();
    if !crate::operator_sensor_diagnostics::initialize(boot_session.get()) {
        log::warn!("operator_sensor_diagnostic=unavailable reason=owner_already_initialized");
    }
    let mut state = ProducerSensorState::default();
    let mut core_voltage_state = CoreVoltageProducerState::default();
    let mut sequences = ProducerSequences::default();
    let started_at_ms = crate::runtime_uptime::millis();
    let mut sensor_publish_deadline_ms = started_at_ms
        .checked_add(SENSOR_STALE_AFTER_MS - SENSOR_PUBLISH_HEADROOM_MS)
        .expect("initial sensor publication deadline is representable");
    let mut sensor_schedule = PeriodicDeadline::new(started_at_ms, SENSOR_SWEEP_CADENCE_MS)
        .expect("operator observation cadence is nonzero");
    let mut maybe_display = maybe_display.map(|owner| {
        let snapshot = crate::runtime_snapshot::collect_screen_snapshot(started_at_ms);
        RuntimeDisplay {
            owner,
            flow: ScreenFlow::new(started_at_ms, &snapshot),
            maybe_last_frame: None,
        }
    });
    crate::runtime_snapshot::record_display_availability(maybe_display.is_some(), started_at_ms);
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
            let (power, actual_asic_temperature_celsius, tachometer_rpm) =
                if let Some(owner) = maybe_owner.as_mut() {
                    let power = timed_i2c_acquisition(
                        OperatorSensorStage::Power,
                        sensor_publish_deadline_ms,
                        |budget| safety_adapter::read_power_acquisition(owner, budget),
                    );
                    let actual_asic_temperature_celsius = timed_i2c_acquisition(
                        OperatorSensorStage::AsicTemperature,
                        sensor_publish_deadline_ms,
                        |budget| safety_adapter::read_asic_temperature_acquisition(owner, budget),
                    );
                    let tachometer_rpm = timed_i2c_acquisition(
                        OperatorSensorStage::Tachometer,
                        sensor_publish_deadline_ms,
                        |budget| safety_adapter::read_tachometer_acquisition(owner, budget),
                    );
                    (power, actual_asic_temperature_celsius, tachometer_rpm)
                } else {
                    (
                        AcquisitionOutcome::Unavailable(UnavailableReason::ProducerUnavailable),
                        AcquisitionOutcome::Unavailable(UnavailableReason::ProducerUnavailable),
                        AcquisitionOutcome::Unavailable(UnavailableReason::ProducerUnavailable),
                    )
                };
            let asic_temperature_celsius = apply_thermal_fault_stimulus(
                &mut maybe_thermal_fault_stimulus,
                state.thermal().temperature_truth(),
                actual_asic_temperature_celsius,
            );
            let vr_temperature_celsius =
                AcquisitionOutcome::Unavailable(UnavailableReason::UnsupportedOnBoard);
            let core_voltage_millivolts = if let Some(adc) = maybe_core_voltage_adc.as_mut() {
                let started_at_ms = crate::runtime_uptime::millis();
                let outcome = safety_adapter::read_core_voltage_acquisition(adc);
                record_sensor_stage(
                    OperatorSensorStage::CoreVoltage,
                    started_at_ms,
                    crate::runtime_uptime::millis(),
                    acquisition_outcome(&outcome),
                );
                outcome
            } else {
                AcquisitionOutcome::Unavailable(UnavailableReason::CoreVoltageUnavailable)
            };
            let acquired_at = MonotonicMillis::new(crate::runtime_uptime::millis());
            sensor_publish_deadline_ms = acquired_at
                .get()
                .checked_add(SENSOR_STALE_AFTER_MS - SENSOR_PUBLISH_HEADROOM_MS)
                .expect("sensor publication deadline is representable");
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
                let display_started_at_ms = crate::runtime_uptime::millis();
                let display_outcome = service_display(
                    owner,
                    &mut maybe_display,
                    now_ms,
                    sensor_publish_deadline_ms,
                );
                record_sensor_stage(
                    OperatorSensorStage::Display,
                    display_started_at_ms,
                    crate::runtime_uptime::millis(),
                    budget_outcome(display_outcome),
                );
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
                        crate::runtime_snapshot::record_display_availability(false, now_ms);
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
                    sensor_publish_deadline_ms,
                ) == SafetyActuationOwnerWait::Disconnected
                {
                    sleep_until(next_owner_deadline_ms);
                }
            }
            _ => sleep_until(next_owner_deadline_ms),
        }
    }
}

fn timed_i2c_acquisition<T>(
    stage: OperatorSensorStage,
    sensor_publish_deadline_ms: u64,
    acquire: impl FnOnce(&mut safety_adapter::RuntimeI2cBudget) -> AcquisitionOutcome<T>,
) -> AcquisitionOutcome<T> {
    let started_at_ms = crate::runtime_uptime::millis();
    let mut budget = safety_adapter::RuntimeI2cBudget::new(sensor_publish_deadline_ms);
    let outcome = acquire(&mut budget);
    let diagnostic_outcome = match budget.outcome() {
        safety_adapter::RuntimeI2cBudgetOutcome::BudgetExhausted => {
            OperatorSensorOutcome::BudgetExhausted
        }
        safety_adapter::RuntimeI2cBudgetOutcome::DriverFailed => {
            OperatorSensorOutcome::DriverFailed
        }
        safety_adapter::RuntimeI2cBudgetOutcome::Recovered
            if matches!(outcome, AcquisitionOutcome::Success(_)) =>
        {
            OperatorSensorOutcome::Recovered
        }
        safety_adapter::RuntimeI2cBudgetOutcome::Ready
        | safety_adapter::RuntimeI2cBudgetOutcome::Recovered => acquisition_outcome(&outcome),
    };
    record_sensor_stage(
        stage,
        started_at_ms,
        crate::runtime_uptime::millis(),
        diagnostic_outcome,
    );
    outcome
}

fn acquisition_outcome<T>(outcome: &AcquisitionOutcome<T>) -> OperatorSensorOutcome {
    match outcome {
        AcquisitionOutcome::Success(_) => OperatorSensorOutcome::Ready,
        AcquisitionOutcome::ReadFailed => OperatorSensorOutcome::DriverFailed,
        AcquisitionOutcome::InvalidSample => OperatorSensorOutcome::SampleInvalid,
        AcquisitionOutcome::Unavailable(_) => OperatorSensorOutcome::Unavailable,
    }
}

fn budget_outcome(outcome: safety_adapter::RuntimeI2cBudgetOutcome) -> OperatorSensorOutcome {
    match outcome {
        safety_adapter::RuntimeI2cBudgetOutcome::Ready => OperatorSensorOutcome::Ready,
        safety_adapter::RuntimeI2cBudgetOutcome::Recovered => OperatorSensorOutcome::Recovered,
        safety_adapter::RuntimeI2cBudgetOutcome::DriverFailed => {
            OperatorSensorOutcome::DriverFailed
        }
        safety_adapter::RuntimeI2cBudgetOutcome::BudgetExhausted => {
            OperatorSensorOutcome::BudgetExhausted
        }
    }
}

fn record_sensor_stage(
    stage: OperatorSensorStage,
    started_at_ms: u64,
    completed_at_ms: u64,
    outcome: OperatorSensorOutcome,
) {
    let Some(diagnostic) = crate::operator_sensor_diagnostics::record_stage(
        stage,
        started_at_ms,
        completed_at_ms,
        outcome,
    ) else {
        return;
    };
    crate::info_retained(&diagnostic.marker());
}

fn admitted_thermal_fault_stimulus(
    maybe_admission: Option<crate::settings_adapter::ThermalFaultStimulusAdmission>,
) -> Option<ThermalFaultStimulus> {
    let admission = maybe_admission?;
    if admission.sample_count() != THERMAL_FAULT_STIMULUS_SAMPLE_COUNT
        || !admission.has_nonzero_lease()
    {
        log::warn!("thermal_fault_stimulus=unavailable reason=admission_contract");
        return None;
    }
    Some(ThermalFaultStimulus::default())
}

fn apply_thermal_fault_stimulus(
    maybe_stimulus: &mut Option<ThermalFaultStimulus>,
    prior: &bitaxe_safety::observation::Observation<ThermalReading>,
    actual: AcquisitionOutcome<f64>,
) -> AcquisitionOutcome<f64> {
    let Some(stimulus) = maybe_stimulus.as_mut() else {
        return actual;
    };
    match stimulus.step(prior, actual) {
        Ok(step) => {
            if let Some(marker) = step.maybe_marker {
                crate::info_retained(&format!(
                    "thermal_fault_stimulus state={} redacted=true",
                    marker.label()
                ));
            }
            if stimulus.is_complete() {
                *maybe_stimulus = None;
            }
            step.outcome
        }
        Err(error) => {
            log::warn!(
                "thermal_fault_stimulus=aborted reason={} redacted=true",
                error.label()
            );
            *maybe_stimulus = None;
            actual
        }
    }
}

struct RuntimeDisplay {
    owner: RuntimeDisplayOwner,
    flow: ScreenFlow,
    maybe_last_frame: Option<ScreenFrame>,
}

fn service_display(
    owner: &mut RuntimeI2cOwner<'_>,
    maybe_display: &mut Option<RuntimeDisplay>,
    uptime_ms: u64,
    sensor_publish_deadline_ms: u64,
) -> safety_adapter::RuntimeI2cBudgetOutcome {
    let mut i2c_budget = safety_adapter::RuntimeI2cBudget::new(sensor_publish_deadline_ms);
    let Some(display) = maybe_display.as_mut() else {
        return i2c_budget.outcome();
    };
    let snapshot = crate::runtime_snapshot::collect_screen_snapshot(uptime_ms);
    let pending_advances = crate::input_adapter::take_pending_screen_advances();
    let decision =
        match next_screen_decision(&mut display.flow, uptime_ms, &snapshot, pending_advances) {
            Ok(decision) => decision,
            Err(error) => {
                let error = anyhow::Error::new(error);
                disable_runtime_display(maybe_display, "screen_flow_failed", &error);
                return i2c_budget.outcome();
            }
        };
    if pending_advances > 0 {
        if let Err(error) = display.owner.record_input_activity(uptime_ms) {
            disable_runtime_display(maybe_display, "input_activity_failed", &error);
            return i2c_budget.outcome();
        }
    }
    if let Err(error) =
        display
            .owner
            .service_power(owner, &mut i2c_budget, uptime_ms, decision.priority_visible)
    {
        if i2c_budget.outcome() != safety_adapter::RuntimeI2cBudgetOutcome::BudgetExhausted {
            disable_runtime_display(maybe_display, "power_command_failed", &error);
        }
        return i2c_budget.outcome();
    }
    if display.maybe_last_frame.as_ref() == Some(&decision.frame) {
        return i2c_budget.outcome();
    }

    if let Err(error) = display
        .owner
        .render_runtime_screen(owner, &mut i2c_budget, &decision.frame)
    {
        crate::runtime_snapshot::record_display_render(
            if snapshot.identify_active {
                DisplayFrameKind::Identify
            } else {
                DisplayFrameKind::NonIdentify
            },
            DisplayRenderOutcome::Failed,
            uptime_ms,
        );
        if i2c_budget.outcome() != safety_adapter::RuntimeI2cBudgetOutcome::BudgetExhausted {
            disable_runtime_display(maybe_display, "render_failed", &error);
        }
        return i2c_budget.outcome();
    }
    // A frame decision is not display evidence. Publish the receipt only after
    // the SSD1306 owner confirms that the framebuffer flush completed.
    crate::runtime_snapshot::record_display_render(
        if snapshot.identify_active {
            DisplayFrameKind::Identify
        } else {
            DisplayFrameKind::NonIdentify
        },
        DisplayRenderOutcome::Rendered,
        uptime_ms,
    );
    display.maybe_last_frame = Some(decision.frame);
    i2c_budget.outcome()
}

fn next_screen_decision(
    flow: &mut ScreenFlow,
    uptime_ms: u64,
    snapshot: &bitaxe_core::screen::ScreenSnapshot,
    pending_advances: u8,
) -> Result<bitaxe_core::screen::ScreenDecision, bitaxe_core::screen::ScreenFlowError> {
    if pending_advances == 0 {
        return flow.update(uptime_ms, snapshot);
    }
    let mut decision = flow.advance_by_input(uptime_ms, snapshot)?;
    for _ in 1..pending_advances {
        decision = flow.advance_by_input(uptime_ms, snapshot)?;
    }
    Ok(decision)
}

fn disable_runtime_display(
    maybe_display: &mut Option<RuntimeDisplay>,
    reason: &str,
    error: &anyhow::Error,
) {
    log::warn!("display_status=runtime_refresh_disabled reason={reason} error={error:#}");
    crate::display_adapter::publish_runtime_display_input_boundary(
        crate::display_adapter::RuntimeDisplayMode::Unavailable,
        crate::input_adapter::is_available(),
    );
    crate::runtime_snapshot::record_display_availability(false, crate::runtime_uptime::millis());
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
