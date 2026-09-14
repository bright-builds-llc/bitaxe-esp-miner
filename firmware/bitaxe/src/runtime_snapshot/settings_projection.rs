//! Exclusive confirmed-settings projection, separate from direct NVS observation.
use super::*;

pub(super) fn collect_settings_projection(timing: &LiveStageProfiler) -> SettingsProjection {
    let (
        maybe_hostname,
        maybe_frequency,
        maybe_voltage,
        maybe_auto_fan_speed,
        maybe_manual_fan_speed,
        start_mining_on_boot,
    ) = timing.measure(LiveStage::ConfirmedSettings, || {
        let confirmed_settings = crate::settings_adapter::current_settings_snapshot();
        let loaded = reload_snapshot(&confirmed_settings);
        (
            match loaded.maybe_loaded_value("hostname") {
                Some(LoadedValue::Str(hostname)) => Some(hostname.clone()),
                _ => None,
            },
            match loaded.maybe_loaded_value("asicfrequency_f") {
                Some(LoadedValue::Float(frequency)) => Some(f64::from(*frequency)),
                _ => None,
            },
            match loaded.maybe_loaded_value("asicvoltage") {
                Some(LoadedValue::U16(voltage)) => Some(*voltage),
                _ => None,
            },
            match loaded.maybe_loaded_value("autofanspeed") {
                Some(LoadedValue::Bool(auto_fan_speed)) => Some(*auto_fan_speed),
                _ => None,
            },
            match loaded.maybe_loaded_value("manualfanspeed") {
                Some(LoadedValue::U16(manual_fan_speed)) => Some(*manual_fan_speed),
                _ => None,
            },
            match loaded.maybe_loaded_value("mineonboot") {
                Some(LoadedValue::Bool(value)) => *value,
                _ => true,
            },
        )
    });
    let system_info = Box::new(SystemInfoSettingsSnapshot::from_nvs_snapshot(
        &crate::settings_adapter::current_system_info_settings_snapshot_profiled(timing),
    ));
    SettingsProjection {
        maybe_hostname,
        maybe_frequency,
        maybe_voltage,
        maybe_auto_fan_speed,
        maybe_manual_fan_speed,
        start_mining_on_boot,
        system_info,
    }
}
