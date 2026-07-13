//! Observe-only Ultra 205 power safety adapter.
#![allow(dead_code)]

use bitaxe_safety::power::VoltageEffectPlan;

pub const DS4432U_I2C_ADDRESS: u8 = 0x48;
pub const DS4432U_OUTPUT0_REGISTER: u8 = 0xF8;
pub const DS4432U_OUTPUT1_REGISTER: u8 = 0xF9;
pub const INA260_I2C_ADDRESS: u8 = 0x40;
pub const INA260_CURRENT_REGISTER: u8 = 0x01;
pub const INA260_BUS_VOLTAGE_REGISTER: u8 = 0x02;
pub const INA260_POWER_REGISTER: u8 = 0x03;

pub fn interpret_voltage_effect(plan: VoltageEffectPlan) {
    if crate::mining_evidence_mode::MiningEvidenceMode::current().is_phase27_live_hardware_bridge()
    {
        if let VoltageEffectPlan::WriteDs4432u { setpoint_mv, .. } = plan {
            log::info!("safety_voltage_effect=armed setpoint_mv={setpoint_mv}");
            return;
        }
    }

    match plan {
        VoltageEffectPlan::NoWrite { reason } | VoltageEffectPlan::SuppressWrite { reason } => {
            log::warn!("safety_voltage_effect=suppressed reason={reason}");
        }
        VoltageEffectPlan::WriteDs4432u {
            i2c_address,
            output_registers,
            setpoint_mv,
        } => {
            log::warn!(
                "safety_voltage_effect=write_suppressed reason=hardware_evidence_pending i2c_address=0x{i2c_address:02x} output0=0x{:02x} output1=0x{:02x} setpoint_mv={setpoint_mv}",
                output_registers[0],
                output_registers[1]
            );
        }
    }
}

pub fn suppress_voltage_write(reason: &'static str) {
    log::warn!("safety_voltage_effect=suppressed reason={reason}");
}
