//! Side-effect-free private projection for the physical screen owner.

use bitaxe_api::{SafeTelemetrySnapshot, SafetyTelemetryStatus};
use bitaxe_config::{reload_snapshot, ultra_205_catalog_entry, LoadedValue};
use bitaxe_core::runtime_health::PassiveSelfTestState;
use bitaxe_core::screen::ScreenSnapshot;

use super::screen_projection::collect as screen_command_projection;

/// Collects private physical-screen facts without operator publication side effects.
pub fn collect_screen_snapshot(now_ms: u64) -> ScreenSnapshot {
    let command = screen_command_projection(now_ms);
    let self_test_running = crate::runtime_health_adapter::collect(now_ms)
        .passive_self_test_state()
        == PassiveSelfTestState::Running;
    let (telemetry_fresh, power_watts, temperature_celsius) = {
        let observations = crate::safety_adapter::observation_snapshot();
        let telemetry = SafeTelemetrySnapshot::from_observations(&observations);
        (
            telemetry.status == SafetyTelemetryStatus::Fresh,
            telemetry.power_watts,
            telemetry.chip_temp_celsius,
        )
    };
    let wifi = crate::wifi_adapter::current_wifi_snapshot();
    let catalog = ultra_205_catalog_entry();
    let pool_host = screen_pool_host(command.fallback_active);

    ScreenSnapshot {
        maybe_self_test: self_test_running
            .then(|| ["running".to_owned(), String::new(), String::new()]),
        overheat: command.overheat,
        identify_active: command.identify_active,
        wifi_connected: wifi.wifi_status == "connected",
        ap_enabled: wifi.ap_enabled,
        ssid: wifi.ssid,
        ap_ssid: wifi.ap_ssid,
        wifi_status: wifi.wifi_status,
        ipv4: wifi.ipv4,
        model: catalog.family().to_owned(),
        board: catalog.board_version().to_owned(),
        version: crate::build_label().to_owned(),
        pool_host,
        maybe_hashrate_ghs: Some(command.hashrate_ghs),
        maybe_power_watts: telemetry_fresh.then_some(power_watts),
        maybe_best_difficulty: command.maybe_best_difficulty,
        maybe_temperature_celsius: telemetry_fresh.then_some(temperature_celsius),
        // Pool difficulty is a share target, not Bitcoin network difficulty.
        // Keep this unavailable until the coinbase owner publishes that fact.
        maybe_network_difficulty: None,
        maybe_rssi_dbm: wifi.maybe_rssi_dbm,
        uptime_seconds: now_ms / 1_000,
        shares_accepted: command.shares_accepted,
        shares_rejected: command.shares_rejected,
        work_received: command.work_received,
        mining_paused: command.mining_paused,
        show_new_block: command.show_new_block,
        ..ScreenSnapshot::default()
    }
}

fn screen_pool_host(fallback_active: bool) -> String {
    let confirmed_settings = crate::settings_adapter::current_settings_snapshot();
    let loaded = reload_snapshot(&confirmed_settings);
    let key = if fallback_active {
        "fbstratumurl"
    } else {
        "stratumurl"
    };
    match loaded.maybe_loaded_value(key) {
        Some(LoadedValue::Str(value)) => value.clone(),
        _ => String::new(),
    }
}
