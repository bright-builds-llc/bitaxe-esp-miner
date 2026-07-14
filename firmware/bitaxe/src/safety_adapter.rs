//! Observe-only firmware safety adapter facade.
//!
//! Phase 27 live bridge mode enables bounded power, thermal, and fan bring-up
//! before BM1366 UART initialization.

mod asic_enable;
mod ds4432u;
mod emc2101;
mod i2c_bus;
mod ina260;
mod observation_store;
pub mod phase27_bring_up;
mod power;
pub mod power_probe;
mod thermal;
mod watchdog;

pub(crate) use i2c_bus::BitaxeI2cBus;
pub(crate) use observation_store::{observation_snapshot, replace_observations_from_producer};
pub use phase27_bring_up::{
    phase27_bring_up_complete, phase27_safety_snapshot, run_phase27_hardware_bring_up,
    Phase27BringUpReset,
};

use bitaxe_safety::{effects::SafetyEffect, status::SafetyStatus};

pub fn interpret_safety_effects(effects: &[SafetyEffect]) {
    for effect in effects {
        interpret_safety_effect(effect);
    }
}

pub fn start_safety_supervisor() {
    if let Err(error) = watchdog::start_safety_supervisor_thread() {
        log::warn!("safety_supervisor=unavailable reason=spawn_failed error={error}");
    }
}

fn interpret_safety_effect(effect: &SafetyEffect) {
    match *effect {
        SafetyEffect::HoldResetLow => {
            log::warn!(
                "safety_effect=hold_reset_low status=unavailable reason=peripherals_unavailable"
            );
        }
        SafetyEffect::DisableAsicEnable => {
            if phase27_bring_up_complete() {
                log::warn!("safety_effect=disable_asic_enable status=deferred");
                return;
            }
            log::warn!("safety_effect=disable_asic_enable status=suppressed");
        }
        SafetyEffect::SuppressVoltageWrite => {
            power::suppress_voltage_write("safety_effect");
        }
        SafetyEffect::SetFanDutyPercent { percent } => {
            if phase27_bring_up_complete() {
                log::info!("safety_fan_effect=armed percent={percent}");
                return;
            }
            thermal::suppress_fan_write(percent, "hardware_evidence_pending");
        }
        SafetyEffect::BlockWorkSubmission { reason } => {
            crate::asic_adapter::publish_mining_loop_blocked_status(reason);
        }
        SafetyEffect::PublishStatus(status) => {
            publish_safety_status(status);
        }
        SafetyEffect::YieldWatchdog { after_ms } => {
            log::warn!("safety_effect=yield_watchdog after_ms={after_ms}");
        }
    }
}

fn publish_safety_status(status: SafetyStatus) {
    log::warn!(
        "safety_status={} reason={}",
        status_label(status),
        status.public_reason()
    );
}

fn status_label(status: SafetyStatus) -> &'static str {
    match status {
        SafetyStatus::Normal => "normal",
        SafetyStatus::Unavailable { .. } => "unavailable",
        SafetyStatus::SafeBlocked { .. } => "safe_blocked",
        SafetyStatus::PowerFault { .. } => "power_fault",
        SafetyStatus::ThermalFault { .. } => "thermal_fault",
        SafetyStatus::FanFault { .. } => "fan_fault",
        SafetyStatus::SelfTestRunning => "self_test_running",
        SafetyStatus::SelfTestPassed => "self_test_passed",
        SafetyStatus::SelfTestFailed { .. } => "self_test_failed",
    }
}
