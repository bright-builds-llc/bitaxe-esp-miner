//! The mining profile the device's saved settings choose (ADR-0037). Read at a
//! lease Start, before any budget is charged.
use bitaxe_config::mining_settings::{saved_asic_frequency_mhz, saved_core_voltage_mv};
use bitaxe_stratum::v1::production_session::{DeviceProfileRefusal, MiningHardwareProfile};

/// Checked against the built board's catalog ranges, and its regulator's voltage ceiling.
pub(crate) fn device_mining_profile() -> Result<MiningHardwareProfile, DeviceProfileRefusal> {
    let snapshot = super::current_settings_snapshot();
    MiningHardwareProfile::from_device_settings_for_board(
        crate::board_identity::catalog_entry(),
        saved_asic_frequency_mhz(&snapshot),
        saved_core_voltage_mv(&snapshot),
    )
}
