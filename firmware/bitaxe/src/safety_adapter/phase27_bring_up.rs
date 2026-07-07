//! Phase 27 hardware safety bring-up before BM1366 UART init.
//!
//! Runs only when compile-time Phase 27 live bridge mode is active.

use std::sync::{Mutex, OnceLock};

use anyhow::Result;
use bitaxe_config::defaults::ultra_205_defaults;
use bitaxe_safety::{
    power::PowerObservation,
    thermal::{
        FanControlDecision, FanControlInputs, FanControlMode, ThermalObservation, ThermalReading,
    },
};
use esp_idf_svc::hal::gpio::{InputPin, OutputPin};

use super::{
    asic_enable::AsicEnable,
    ds4432u, emc2101,
    i2c_bus::BitaxeI2cBus,
    ina260::{self, Ina260Sample},
};

const BRING_UP_SETTLE_MS: u64 = 500;
pub const RESET_PULSE_LOW_MS: u32 = 100;
pub const RESET_PULSE_HIGH_MS: u32 = 100;

pub trait Phase27BringUpReset {
    fn hold_reset_low(&mut self) -> Result<()>;
    fn reset_pulse(&mut self, low_ms: u32, high_ms: u32) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct Phase27SafetySnapshot {
    pub bring_up_complete: bool,
    pub fan_duty_percent: u8,
    pub fan_rpm: u16,
    pub maybe_power: Option<PowerObservation>,
    pub maybe_thermal: Option<ThermalObservation>,
}

impl Phase27SafetySnapshot {
    fn empty() -> Self {
        Self {
            bring_up_complete: false,
            fan_duty_percent: 0,
            fan_rpm: 0,
            maybe_power: None,
            maybe_thermal: None,
        }
    }
}

static PHASE27_SAFETY_SNAPSHOT: OnceLock<Mutex<Phase27SafetySnapshot>> = OnceLock::new();

fn snapshot_state() -> &'static Mutex<Phase27SafetySnapshot> {
    PHASE27_SAFETY_SNAPSHOT.get_or_init(|| Mutex::new(Phase27SafetySnapshot::empty()))
}

#[must_use]
pub fn phase27_bring_up_complete() -> bool {
    snapshot_state()
        .lock()
        .ok()
        .is_some_and(|state| state.bring_up_complete)
}

pub fn phase27_safety_snapshot() -> Phase27SafetySnapshot {
    snapshot_state()
        .lock()
        .ok()
        .map(|state| state.clone())
        .unwrap_or_else(Phase27SafetySnapshot::empty)
}

pub fn run_phase27_hardware_bring_up<I2C, SDA, SCL, ENABLE, RESET>(
    i2c: I2C,
    sda: SDA,
    scl: SCL,
    asic_enable_pin: ENABLE,
    asic_reset: &mut RESET,
) -> Result<()>
where
    I2C: esp_idf_svc::hal::i2c::I2c + 'static,
    SDA: InputPin + OutputPin + 'static,
    SCL: InputPin + OutputPin + 'static,
    ENABLE: OutputPin + 'static,
    RESET: Phase27BringUpReset,
{
    if !crate::mining_evidence_mode::MiningEvidenceMode::current().is_phase27_live_hardware_bridge()
    {
        return Ok(());
    }

    log::info!("phase27_safety_bring_up=started");

    let mut bus = BitaxeI2cBus::new(i2c, sda, scl)?;
    asic_reset.hold_reset_low()?;

    let mut asic_enable = AsicEnable::new(asic_enable_pin)?;
    asic_enable.enable_power()?;

    let defaults = ultra_205_defaults();
    ds4432u::set_core_voltage_mv(&mut bus, defaults.asic_voltage_mv())?;

    emc2101::init(&mut bus)?;

    let fan_inputs = FanControlInputs {
        mode: FanControlMode::Startup,
        observation: ThermalObservation::from_reading(Some(ThermalReading {
            chip_temp_celsius: 25.0,
            board_temp_celsius: None,
            vr_temp_celsius: None,
        })),
    };
    let fan_decision = FanControlDecision::from_inputs(fan_inputs)?;
    emc2101::set_fan_duty_percent(&mut bus, fan_decision.duty_percent)?;

    std::thread::sleep(std::time::Duration::from_millis(BRING_UP_SETTLE_MS));

    let power_sample = match ina260::read_sample(&mut bus) {
        Ok(sample) => {
            log::info!("safety_power_status=observed");
            Some(sample)
        }
        Err(error) => {
            log::warn!("safety_power_status=unavailable error={error:#}");
            None
        }
    };

    let chip_temp = emc2101::read_external_temp_celsius(&mut bus);
    let fan_rpm = emc2101::read_fan_rpm(&mut bus).unwrap_or(0);

    let thermal_observation = match chip_temp {
        Ok(temp) if temp.is_finite() && temp > -40.0 => {
            ThermalObservation::from_reading(Some(ThermalReading {
                chip_temp_celsius: temp,
                board_temp_celsius: None,
                vr_temp_celsius: None,
            }))
        }
        Ok(_) => ThermalObservation::from_reading(None),
        Err(error) => {
            log::warn!("safety_thermal_status=unavailable category=read_error");
            log::warn!("safety_thermal_read_error={error:#}");
            ThermalObservation::from_reading(None)
        }
    };
    log_phase27_thermal_status(thermal_observation);

    log::info!(
        "safety_fan_status=startup_duty percent={} rpm={fan_rpm}",
        fan_decision.duty_percent
    );

    store_snapshot(
        fan_decision.duty_percent,
        fan_rpm,
        power_sample,
        thermal_observation,
    );

    asic_reset.reset_pulse(RESET_PULSE_LOW_MS, RESET_PULSE_HIGH_MS)?;
    log::info!("asic_reset_status=post_bring_up_pulse");

    log::info!("phase27_safety_bring_up=complete");
    Ok(())
}

fn log_phase27_thermal_status(observation: ThermalObservation) {
    match observation.status {
        bitaxe_safety::thermal::ThermalObservationStatus::Fresh => {
            log::info!("safety_thermal_status=observed category=fresh");
        }
        bitaxe_safety::thermal::ThermalObservationStatus::Fault { reason } => {
            log::warn!("safety_thermal_status=fault category={reason}");
        }
        bitaxe_safety::thermal::ThermalObservationStatus::Unavailable { reason } => {
            log::warn!("safety_thermal_status=unavailable category={reason}");
        }
    }
}

fn store_snapshot(
    fan_duty_percent: u8,
    fan_rpm: u16,
    maybe_power_sample: Option<Ina260Sample>,
    thermal_observation: ThermalObservation,
) {
    let Ok(mut state) = snapshot_state().lock() else {
        return;
    };
    state.bring_up_complete = true;
    state.fan_duty_percent = fan_duty_percent;
    state.fan_rpm = fan_rpm;
    state.maybe_power = maybe_power_sample.map(ina260::power_observation_from_sample);
    state.maybe_thermal = Some(thermal_observation);
}
