//! Side-effect-free private projection for the physical screen owner.

use bitaxe_api::{IdentifyMode, SafetyTelemetryStatus};
use bitaxe_config::{reload_snapshot, LoadedValue};
use bitaxe_core::screen::ScreenSnapshot;

use super::{collect_operator_snapshot_candidate, command_visible_state, complete_api_snapshot};

/// Collects private physical-screen facts without operator publication side effects.
pub fn collect_screen_snapshot(now_ms: u64) -> ScreenSnapshot {
    let candidate = collect_operator_snapshot_candidate(false);
    let wifi = candidate.wifi.clone();
    let snapshot = complete_api_snapshot(candidate);
    let command_state = command_visible_state();
    let identify_active = command_state.identify.mode_at(now_ms) == IdentifyMode::Active;
    let work_received = command_state.work_received;
    let telemetry_fresh = snapshot.safe_telemetry.status == SafetyTelemetryStatus::Fresh;
    let pool_host = screen_pool_host(snapshot.mining.fallback_active);
    let mining_paused =
        snapshot.mining.operator_intent == bitaxe_stratum::v1::state::MiningOperatorIntent::Paused;

    ScreenSnapshot {
        maybe_self_test: (snapshot.runtime_health.passive_self_test_state()
            == bitaxe_core::runtime_health::PassiveSelfTestState::Running)
            .then(|| ["running".to_owned(), String::new(), String::new()]),
        overheat: snapshot.mining.maybe_blocked_reason == Some("overheat_safe_stop"),
        identify_active,
        wifi_connected: wifi.wifi_status == "connected",
        ap_enabled: wifi.ap_enabled,
        ssid: wifi.ssid,
        ap_ssid: wifi.ap_ssid,
        wifi_status: wifi.wifi_status,
        ipv4: wifi.ipv4,
        model: snapshot.catalog.family().to_owned(),
        board: snapshot.catalog.board_version().to_owned(),
        version: snapshot.platform.version,
        pool_host,
        maybe_hashrate_ghs: Some(snapshot.mining.hashrate_inputs.current_ghs),
        maybe_power_watts: telemetry_fresh.then_some(snapshot.safe_telemetry.power_watts),
        maybe_best_difficulty: snapshot
            .mining
            .counters
            .maybe_best_difficulty
            .map(|difficulty| difficulty.raw()),
        maybe_temperature_celsius: telemetry_fresh
            .then_some(snapshot.safe_telemetry.chip_temp_celsius),
        // Pool difficulty is a share target, not Bitcoin network difficulty.
        // Keep this unavailable until the coinbase owner publishes that fact.
        maybe_network_difficulty: None,
        maybe_rssi_dbm: wifi.maybe_rssi_dbm,
        uptime_seconds: now_ms / 1_000,
        shares_accepted: snapshot.mining.counters.accepted,
        shares_rejected: snapshot.mining.counters.rejected,
        work_received,
        mining_paused,
        show_new_block: snapshot.block_found.show_new_block,
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
