//! Firmware collection boundary for pure AxeOS API response snapshots.

use std::sync::{Mutex, OnceLock};

use bitaxe_api::{
    apply_block_found_dismiss_effect, apply_identify_mode_effect, apply_mining_activity_effect,
    ApiSnapshot, BlockFoundDismissEffect, BlockFoundNotificationState, IdentifyMode,
    IdentifyModeEffect, IdentifyModeState, MiningActivityEffect, PlatformSnapshot,
    SafeTelemetrySnapshot,
};
use bitaxe_config::{reload_snapshot, LoadedValue};
use bitaxe_stratum::v1::state::MiningRuntimeState;
use esp_idf_svc::sys;

static COMMAND_VISIBLE_STATE: OnceLock<Mutex<CommandVisibleState>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq)]
struct CommandVisibleState {
    mining: MiningRuntimeState,
    identify: IdentifyModeState,
    block_found: BlockFoundNotificationState,
}

impl Default for CommandVisibleState {
    fn default() -> Self {
        Self {
            mining: MiningRuntimeState::default(),
            identify: IdentifyModeState::inactive(),
            block_found: BlockFoundNotificationState {
                block_found: 0,
                show_new_block: false,
            },
        }
    }
}

/// Collects current firmware facts and overlays them on the safe Ultra 205 API
/// snapshot used by the pure contract mappers.
pub fn collect_api_snapshot() -> ApiSnapshot {
    let mut snapshot = ApiSnapshot::safe_ultra_205();
    let command_state = command_visible_state();
    snapshot.mining = command_state.mining;
    snapshot.block_found = command_state.block_found;
    snapshot.platform = collect_platform_snapshot(snapshot.platform);
    snapshot.safe_telemetry =
        SafeTelemetrySnapshot::from_report(crate::safety_adapter::collect_safety_report());
    apply_wifi_snapshot(&mut snapshot);
    apply_settings_snapshot(&mut snapshot);
    snapshot
}

/// Returns the current command-visible mining state.
pub fn mining_runtime_state() -> MiningRuntimeState {
    command_visible_state().mining
}

/// Returns the current identify mode used to plan the next identify command.
pub fn identify_mode() -> IdentifyMode {
    command_visible_state().identify.mode_at(uptime_millis())
}

/// Returns the current block-found notification state.
pub fn block_found_notification_state() -> BlockFoundNotificationState {
    command_visible_state().block_found
}

/// Applies an API-visible mining command effect.
pub fn apply_mining_activity_command(effect: MiningActivityEffect) {
    mutate_command_visible_state(|state| apply_mining_activity_effect(&mut state.mining, effect));
}

/// Replaces API-visible mining state after Phase 21 controlled evidence runs.
pub fn replace_mining_runtime_state_for_evidence(mining: MiningRuntimeState) {
    mutate_command_visible_state(|state| {
        state.mining = mining;
    });
}

/// Replaces API-visible mining state after Phase 25 safe stop completes.
pub fn replace_mining_runtime_state_after_phase25_safe_stop(mining: MiningRuntimeState) {
    replace_mining_runtime_state_for_evidence(mining);
}

/// Applies an API-visible identify command effect.
pub fn apply_identify_mode_command(effect: IdentifyModeEffect) {
    let now_ms = uptime_millis();
    mutate_command_visible_state(|state| {
        apply_identify_mode_effect(&mut state.identify, effect, now_ms);
    });
}

/// Applies an API-visible block-found dismiss command effect.
pub fn apply_block_found_dismiss_command(effect: BlockFoundDismissEffect) {
    mutate_command_visible_state(|state| {
        state.block_found = apply_block_found_dismiss_effect(effect);
    });
}

fn command_visible_state() -> CommandVisibleState {
    let state = COMMAND_VISIBLE_STATE.get_or_init(|| Mutex::new(CommandVisibleState::default()));
    let Ok(state) = state.lock() else {
        log::warn!("axeos_runtime_state=unavailable reason=mutex_poisoned");
        return CommandVisibleState::default();
    };

    state.clone()
}

fn mutate_command_visible_state(mutate: impl FnOnce(&mut CommandVisibleState)) {
    let state = COMMAND_VISIBLE_STATE.get_or_init(|| Mutex::new(CommandVisibleState::default()));
    let Ok(mut state) = state.lock() else {
        log::warn!("axeos_runtime_state=unavailable reason=mutex_poisoned");
        return;
    };

    mutate(&mut state);
}

fn apply_settings_snapshot(snapshot: &mut ApiSnapshot) {
    let settings = crate::settings_adapter::current_settings_snapshot();
    let loaded = reload_snapshot(&settings);

    if let Some(LoadedValue::Str(hostname)) = loaded.loaded_value("hostname") {
        snapshot.platform.hostname = hostname.clone();
    }

    if let Some(LoadedValue::Float(frequency)) = loaded.loaded_value("asicfrequency_f") {
        snapshot.config.asic_frequency_mhz = f64::from(*frequency);
    }

    if let Some(LoadedValue::U16(voltage)) = loaded.loaded_value("asicvoltage") {
        snapshot.config.asic_voltage_mv = *voltage;
    }

    if let Some(LoadedValue::Bool(auto_fan_speed)) = loaded.loaded_value("autofanspeed") {
        snapshot.config.auto_fan_speed = *auto_fan_speed;
    }

    if let Some(LoadedValue::U16(manual_fan_speed)) = loaded.loaded_value("manualfanspeed") {
        snapshot.config.manual_fan_speed = *manual_fan_speed;
    }
}

fn apply_wifi_snapshot(snapshot: &mut ApiSnapshot) {
    let wifi = crate::wifi_adapter::current_wifi_snapshot();
    snapshot.platform.wifi_status = wifi.wifi_status;
    snapshot.platform.ssid = wifi.ssid;
    snapshot.platform.ipv4 = wifi.ipv4;
    snapshot.platform.mac_addr = wifi.mac_addr;
    snapshot.platform.ap_enabled = wifi.ap_enabled;
    if let Some(rssi) = wifi.maybe_rssi_dbm {
        snapshot.safe_telemetry.wifi_rssi_dbm = rssi;
    }
}

fn collect_platform_snapshot(mut platform: PlatformSnapshot) -> PlatformSnapshot {
    platform.version = crate::firmware_commit().to_owned();
    platform.idf_version = crate::ESP_IDF_VERSION.to_owned();
    platform.reset_reason = crate::reset_reason().to_string();
    platform.running_partition = crate::partition_label();
    platform.psram_available = crate::psram_status() == "available";
    platform.free_heap = heap_free(sys::MALLOC_CAP_DEFAULT);
    platform.free_heap_internal = heap_free(sys::MALLOC_CAP_INTERNAL);
    platform.free_heap_spiram = heap_free(sys::MALLOC_CAP_SPIRAM);
    platform.min_free_heap = heap_min_free(sys::MALLOC_CAP_DEFAULT);
    platform.max_alloc_heap = heap_largest_free_block(sys::MALLOC_CAP_DEFAULT);
    platform.uptime_seconds = uptime_seconds();
    platform
}

fn heap_free(caps: u32) -> u64 {
    unsafe { sys::heap_caps_get_free_size(caps) as u64 }
}

fn heap_min_free(caps: u32) -> u64 {
    unsafe { sys::heap_caps_get_minimum_free_size(caps) as u64 }
}

fn heap_largest_free_block(caps: u32) -> u64 {
    unsafe { sys::heap_caps_get_largest_free_block(caps) as u64 }
}

fn uptime_seconds() -> u64 {
    let uptime_micros = unsafe { sys::esp_timer_get_time() };
    if uptime_micros <= 0 {
        return 0;
    }

    (uptime_micros as u64) / 1_000_000
}

fn uptime_millis() -> u64 {
    let uptime_micros = unsafe { sys::esp_timer_get_time() };
    if uptime_micros <= 0 {
        return 0;
    }

    (uptime_micros as u64) / 1_000
}

#[cfg(test)]
mod tests {
    use bitaxe_stratum::v1::{
        messages::PoolDifficulty,
        production_work::PoolSessionGeneration,
        state::{HashrateInputs, MiningActivityStatus, PoolLifecycleStatus, WorkSubmissionGate},
        submit_response::SubmitClassification,
        telemetry_projection::RuntimeProjectionSampleSource,
    };

    use super::*;

    #[test]
    fn runtime_projection_lifecycle_and_hashrate_events_update_visible_state() {
        // Arrange
        reset_command_visible_state_for_test();
        let inputs = HashrateInputs {
            hashes_done: 8_192,
            elapsed_ms: 2_048,
            rolling_hashrate_hs: 4_096.0,
        };

        // Act
        publish_runtime_lifecycle(PoolLifecycleStatus::Active);
        publish_runtime_hashrate_inputs(inputs);
        publish_runtime_pool_difficulty(PoolDifficulty { difficulty: 16.0 });

        // Assert
        let mining = mining_runtime_state();
        assert_eq!(mining.lifecycle, PoolLifecycleStatus::Active);
        assert_eq!(mining.hashrate_inputs, inputs);
        assert_eq!(
            mining.maybe_pool_difficulty,
            Some(PoolDifficulty { difficulty: 16.0 })
        );
        assert_eq!(mining.counters.accepted, 0);
        assert_eq!(mining.counters.rejected, 0);
    }

    #[test]
    fn runtime_projection_sample_markers_drain_once_per_producer_boundary() {
        // Arrange
        reset_command_visible_state_for_test();

        // Act
        publish_runtime_bounded_sample_marker(RuntimeProjectionSampleSource::RuntimeEvent);
        let maybe_first_marker = drain_pending_runtime_sample_marker_for_test();
        let maybe_second_marker = drain_pending_runtime_sample_marker_for_test();

        // Assert
        let first_marker = maybe_first_marker.expect("runtime boundary should emit sample marker");
        assert_eq!(first_marker.source, RuntimeProjectionSampleSource::RuntimeEvent);
        assert!(maybe_second_marker.is_none());
    }

    #[test]
    fn runtime_projection_submit_classification_gates_counters_by_generation() {
        // Arrange
        reset_command_visible_state_for_test();
        let current_generation = PoolSessionGeneration::initial();
        let stale_generation = current_generation.next();

        // Act
        publish_runtime_submit_classification(
            stale_generation,
            SubmitClassification::Accepted,
            None,
        );
        publish_runtime_submit_classification(
            current_generation,
            SubmitClassification::Blocked {
                reason: "submit_intent_missing",
            },
            None,
        );

        // Assert
        let mining = mining_runtime_state();
        assert_eq!(mining.counters.accepted, 0);
        assert_eq!(mining.counters.rejected, 0);
    }

    #[test]
    fn runtime_projection_safe_stop_resets_active_mining_before_snapshot_collection() {
        // Arrange
        reset_command_visible_state_for_test();
        publish_runtime_work_submission_ready();

        // Act
        publish_runtime_blocked("phase25_safe_stop");
        publish_runtime_safe_stopped("phase25_safe_stop");
        let snapshot = collect_api_snapshot();

        // Assert
        assert_eq!(snapshot.mining.lifecycle, PoolLifecycleStatus::Disconnected);
        assert_eq!(snapshot.mining.mining_activity, MiningActivityStatus::SafeBlocked);
        assert_eq!(snapshot.mining.work_submission, WorkSubmissionGate::Blocked);
        assert_eq!(
            snapshot.mining.maybe_blocked_reason,
            Some("phase25_safe_stop")
        );
    }
}
