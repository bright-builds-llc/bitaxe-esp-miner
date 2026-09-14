//! Exclusive confirmed-settings projection, separate from direct NVS observation.
use super::*;

pub(super) fn collect_settings_projection_profiled(
    timing: &LiveStageProfiler,
) -> SettingsProjection {
    let maybe_started = timing.maybe_start_stage();
    let confirmed_settings = crate::settings_adapter::current_settings_snapshot();
    let loaded = reload_snapshot(&confirmed_settings);
    let maybe_hostname = match loaded.maybe_loaded_value("hostname") {
        Some(LoadedValue::Str(hostname)) => Some(hostname.clone()),
        _ => None,
    };
    let maybe_frequency = match loaded.maybe_loaded_value("asicfrequency_f") {
        Some(LoadedValue::Float(frequency)) => Some(f64::from(*frequency)),
        _ => None,
    };
    let maybe_voltage = match loaded.maybe_loaded_value("asicvoltage") {
        Some(LoadedValue::U16(voltage)) => Some(*voltage),
        _ => None,
    };
    let maybe_auto_fan_speed = match loaded.maybe_loaded_value("autofanspeed") {
        Some(LoadedValue::Bool(auto_fan_speed)) => Some(*auto_fan_speed),
        _ => None,
    };
    let maybe_manual_fan_speed = match loaded.maybe_loaded_value("manualfanspeed") {
        Some(LoadedValue::U16(manual_fan_speed)) => Some(*manual_fan_speed),
        _ => None,
    };
    let start_mining_on_boot = match loaded.maybe_loaded_value("mineonboot") {
        Some(LoadedValue::Bool(value)) => *value,
        _ => true,
    };
    drop(loaded);
    drop(confirmed_settings);
    timing.finish_stage(LiveStage::ConfirmedSettings, maybe_started);
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

// Ordinary snapshots keep their pre-diagnostic value and stack boundaries.
pub(super) fn collect_settings_projection() -> SettingsProjection {
    let confirmed_settings = crate::settings_adapter::current_settings_snapshot();
    let loaded = reload_snapshot(&confirmed_settings);
    SettingsProjection {
        maybe_hostname: match loaded.maybe_loaded_value("hostname") {
            Some(LoadedValue::Str(hostname)) => Some(hostname.clone()),
            _ => None,
        },
        maybe_frequency: match loaded.maybe_loaded_value("asicfrequency_f") {
            Some(LoadedValue::Float(frequency)) => Some(f64::from(*frequency)),
            _ => None,
        },
        maybe_voltage: match loaded.maybe_loaded_value("asicvoltage") {
            Some(LoadedValue::U16(voltage)) => Some(*voltage),
            _ => None,
        },
        maybe_auto_fan_speed: match loaded.maybe_loaded_value("autofanspeed") {
            Some(LoadedValue::Bool(auto_fan_speed)) => Some(*auto_fan_speed),
            _ => None,
        },
        maybe_manual_fan_speed: match loaded.maybe_loaded_value("manualfanspeed") {
            Some(LoadedValue::U16(manual_fan_speed)) => Some(*manual_fan_speed),
            _ => None,
        },
        start_mining_on_boot: match loaded.maybe_loaded_value("mineonboot") {
            Some(LoadedValue::Bool(value)) => *value,
            _ => true,
        },
        system_info: Box::new(SystemInfoSettingsSnapshot::from_nvs_snapshot(
            &crate::settings_adapter::current_system_info_settings_snapshot(),
        )),
    }
}

pub(super) fn apply_settings_snapshot(snapshot: &mut ApiSnapshot, settings: SettingsProjection) {
    snapshot.system_info_settings = settings.system_info;
    snapshot.project_settings.start_mining_on_boot = settings.start_mining_on_boot;
    if let Some(hostname) = settings.maybe_hostname {
        snapshot.platform.hostname = hostname;
    }

    if let Some(frequency) = settings.maybe_frequency {
        snapshot.config.asic_frequency_mhz = frequency;
    }

    if let Some(voltage) = settings.maybe_voltage {
        snapshot.config.asic_voltage_mv = voltage;
    }

    if let Some(auto_fan_speed) = settings.maybe_auto_fan_speed {
        snapshot.config.auto_fan_speed = auto_fan_speed;
    }

    if let Some(manual_fan_speed) = settings.maybe_manual_fan_speed {
        snapshot.config.manual_fan_speed = manual_fan_speed;
    }
}

pub(super) fn apply_wifi_snapshot(
    snapshot: &mut ApiSnapshot,
    wifi: crate::wifi_adapter::WifiRuntimeSnapshot,
) {
    snapshot.platform.wifi_status = wifi.wifi_status;
    snapshot.platform.ssid = wifi.ssid;
    snapshot.platform.ipv4 = wifi.ipv4;
    snapshot.platform.ipv6 = wifi.ipv6;
    snapshot.platform.mac_addr = wifi.mac_addr;
    snapshot.platform.ap_enabled = wifi.ap_enabled;
    if let Some(rssi) = wifi.maybe_rssi_dbm {
        snapshot.safe_telemetry.wifi_rssi_dbm = rssi;
    }
}
