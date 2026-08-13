//! Production shell for the qualified fan controller.

use std::{thread, time::Duration};

use anyhow::{Context, Result};
use bitaxe_config::{reload_snapshot, LoadedValue};
use bitaxe_core::runtime_orchestration::PeriodicDeadline;
use bitaxe_safety::thermal::{ThermalObservation, ThermalReading};
use bitaxe_stratum::v1::state::{MiningOperatorIntent, PoolLifecycleStatus};

use crate::{
    fan_controller_plan::{
        FanControllerPlan, FanControllerSettings, FanControllerState, FanRuntimeStatus,
        FAN_CONTROLLER_CADENCE_MS,
    },
    safety_adapter::{FanDutyPercent, SafetyActuationCommand, SafetyActuationRequestOutcome},
};

const THREAD_NAME: &str = "fan-controller";
const THREAD_STACK_BYTES: usize = 8 * 1024;

pub(crate) fn start() -> Result<()> {
    thread::Builder::new()
        .name(THREAD_NAME.to_owned())
        .stack_size(THREAD_STACK_BYTES)
        .spawn(run)
        .context("spawn fan controller")?;
    log::info!("fan_controller=started cadence_ms={FAN_CONTROLLER_CADENCE_MS}");
    Ok(())
}

fn run() -> ! {
    let mut state = RuntimeState::default();
    let started_at_ms = crate::runtime_uptime::millis();
    let mut schedule = PeriodicDeadline::new(started_at_ms, FAN_CONTROLLER_CADENCE_MS)
        .expect("fan controller cadence is nonzero");

    loop {
        let now_ms = crate::runtime_uptime::millis();
        if schedule.is_due(now_ms) {
            service_iteration(&mut state, now_ms);
            match schedule.advance_past(crate::runtime_uptime::millis()) {
                Ok(advance) if advance.missed_slots() > 0 => log::warn!(
                    "fan_controller=overrun category=deadline_missed slots={}",
                    advance.missed_slots()
                ),
                Ok(_) => {}
                Err(_) => {
                    log::error!("fan_controller=fault category=deadline_overflow action=halt");
                    park_forever();
                }
            }
        }
        sleep_until(schedule.next_deadline_ms());
    }
}

#[derive(Debug, Default)]
struct RuntimeState {
    controller: FanControllerState,
    maybe_last_warning: Option<&'static str>,
}

impl RuntimeState {
    fn warn_once(&mut self, category: &'static str) {
        if self.maybe_last_warning == Some(category) {
            return;
        }
        self.maybe_last_warning = Some(category);
        log::warn!("fan_controller=deferred category={category}");
    }

    fn clear_warning(&mut self) {
        self.maybe_last_warning = None;
    }
}

fn service_iteration(state: &mut RuntimeState, now_ms: u64) {
    let Some(settings) = current_settings() else {
        state.controller.invalidate_applied_duty();
        state.warn_once("settings_unavailable");
        return;
    };
    let runtime = current_runtime_status();
    let observation = current_thermal_observation();
    let plan = match state
        .controller
        .plan(settings, runtime, observation, now_ms)
    {
        Ok(plan) => plan,
        Err(_) => {
            state.controller.invalidate_applied_duty();
            state.warn_once("plan_invalid");
            return;
        }
    };
    let FanControllerPlan::Apply { percent, mode } = plan else {
        if let FanControllerPlan::SafeBlocked { reason } = plan {
            state.warn_once(reason);
        }
        return;
    };
    let Ok(validated_percent) = FanDutyPercent::try_from(percent) else {
        state.controller.invalidate_applied_duty();
        state.warn_once("duty_out_of_range");
        return;
    };
    match crate::safety_adapter::request_safety_actuation(SafetyActuationCommand::SetFanDuty(
        validated_percent,
    )) {
        SafetyActuationRequestOutcome::Applied => {
            state.controller.record_applied(percent);
            state.clear_warning();
            log::info!(
                "fan_controller=applied mode={} duty_percent={}",
                mode.as_str(),
                percent
            );
        }
        outcome => {
            state.controller.record_apply_failure(now_ms);
            state.warn_once(actuation_failure_category(outcome));
        }
    }
}

fn current_settings() -> Option<FanControllerSettings> {
    let loaded = reload_snapshot(&crate::settings_adapter::current_settings_snapshot());
    FanControllerSettings::parse(
        loaded_bool(&loaded, "autofanspeed")?,
        i64::from(loaded_u16(&loaded, "manualfanspeed")?),
        i64::from(loaded_u16(&loaded, "minfanspeed")?),
        i64::from(loaded_u16(&loaded, "temptarget")?),
        loaded_bool(&loaded, "overheat_mode")?,
    )
    .ok()
}

fn loaded_bool(loaded: &bitaxe_config::PersistenceDecision, key: &str) -> Option<bool> {
    let Some(LoadedValue::Bool(value)) = loaded.maybe_loaded_value(key) else {
        return None;
    };
    Some(*value)
}

fn loaded_u16(loaded: &bitaxe_config::PersistenceDecision, key: &str) -> Option<u16> {
    let Some(LoadedValue::U16(value)) = loaded.maybe_loaded_value(key) else {
        return None;
    };
    Some(*value)
}

fn current_runtime_status() -> FanRuntimeStatus {
    let mining = crate::runtime_snapshot::mining_runtime_state();
    FanRuntimeStatus {
        hardware_control_qualified:
            crate::production_mining_session::fan_controller_actuation_qualified(),
        operator_paused: mining.operator_intent == MiningOperatorIntent::Paused,
        pools_unavailable: !matches!(
            mining.lifecycle,
            PoolLifecycleStatus::Active | PoolLifecycleStatus::FallbackActive
        ),
    }
}

fn current_thermal_observation() -> ThermalObservation {
    let observations = crate::safety_adapter::observation_snapshot();
    let maybe_temperature = observations
        .chip_temp_celsius
        .is_fresh()
        .then(|| observations.chip_temp_celsius.maybe_last_good())
        .flatten()
        .map(|sample| *sample.value());
    ThermalObservation::from_reading(maybe_temperature.map(|temperature| ThermalReading {
        chip_temp_celsius: temperature,
        maybe_board_temp_celsius: None,
        maybe_vr_temp_celsius: None,
    }))
}

const fn actuation_failure_category(outcome: SafetyActuationRequestOutcome) -> &'static str {
    match outcome {
        SafetyActuationRequestOutcome::Applied => "applied",
        SafetyActuationRequestOutcome::QueueFull => "queue_full",
        SafetyActuationRequestOutcome::OwnerUnavailable => "owner_unavailable",
        SafetyActuationRequestOutcome::ReplyTimedOut => "reply_timed_out",
        SafetyActuationRequestOutcome::HardwareWriteFailed => "hardware_write_failed",
    }
}

fn sleep_until(deadline_ms: u64) {
    let remaining_ms = deadline_ms.saturating_sub(crate::runtime_uptime::millis());
    thread::sleep(Duration::from_millis(remaining_ms.max(1)));
}

fn park_forever() -> ! {
    loop {
        thread::park();
    }
}
