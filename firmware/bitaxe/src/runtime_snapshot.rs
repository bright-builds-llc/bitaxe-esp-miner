//! Firmware collection boundary for pure AxeOS API response snapshots.

use std::sync::{Mutex, OnceLock};

use bitaxe_api::{
    apply_block_found_dismiss_effect, apply_identify_mode_effect, apply_mining_activity_effect,
    project_api_views, project_system_info, scoreboard_response, statistics_response, ApiSnapshot,
    BlockFoundDismissEffect, BlockFoundNotificationState, IdentifyMode, IdentifyModeEffect,
    IdentifyModeState, MiningActivityEffect, OperatorSnapshotIdentity, OperatorSnapshotSequence,
    PlatformFact, PlatformIdentity, PlatformSnapshot, ProjectedApiViews, SafeTelemetrySnapshot,
    ScoreboardEntryWire, StatisticsWire, SystemInfoWire,
};
use bitaxe_config::{reload_snapshot, LoadedValue};
use bitaxe_stratum::v1::telemetry_projection::RuntimeProjectionSampleMarker;
use bitaxe_stratum::v1::{
    messages::PoolDifficulty,
    production_work::PoolSessionGeneration,
    state::{HashrateInputs, MiningRuntimeState, PoolLifecycleStatus, ShareDifficulty},
    submit_response::SubmitClassification,
    telemetry_projection::{
        ProjectionShareOutcome, RuntimeProjectionSampleSource, RuntimeTelemetryEvent,
        RuntimeTelemetryProjection, RuntimeTelemetrySequence,
    },
};
static COMMAND_VISIBLE_STATE: OnceLock<Mutex<CommandVisibleState>> = OnceLock::new();
static OPERATOR_SNAPSHOT_SEQUENCE: OnceLock<Mutex<OperatorSnapshotSequence>> = OnceLock::new();

#[derive(Debug, Clone, PartialEq)]
struct CommandVisibleState {
    mining: MiningRuntimeState,
    runtime_projection: RuntimeTelemetryProjection,
    next_runtime_sequence: u64,
    identify: IdentifyModeState,
    block_found: BlockFoundNotificationState,
}

impl Default for CommandVisibleState {
    fn default() -> Self {
        Self {
            mining: MiningRuntimeState::default(),
            runtime_projection: RuntimeTelemetryProjection::new(PoolSessionGeneration::initial()),
            next_runtime_sequence: 1,
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
    let operator_snapshot_identity = reserve_operator_snapshot_identity();
    let command_state = command_visible_state();
    collect_completed_api_snapshot(
        operator_snapshot_identity,
        command_state.mining,
        command_state.block_found,
    )
}

fn collect_completed_api_snapshot(
    operator_snapshot_identity: OperatorSnapshotIdentity,
    mining: MiningRuntimeState,
    block_found: BlockFoundNotificationState,
) -> ApiSnapshot {
    let mut snapshot = ApiSnapshot::safe_ultra_205();
    snapshot.operator_snapshot_identity = operator_snapshot_identity;
    snapshot.mining = mining;
    snapshot.block_found = block_found;
    snapshot.platform_identity = crate::platform_identity::collect();
    snapshot.platform = collect_platform_snapshot(snapshot.platform, &snapshot.platform_identity);
    let observations = crate::safety_adapter::observation_snapshot();
    snapshot.safe_telemetry = SafeTelemetrySnapshot::from_observations(&observations);
    apply_wifi_snapshot(&mut snapshot);
    apply_settings_snapshot(&mut snapshot);
    retain_completed_operator_snapshot(snapshot.operator_snapshot_identity);
    snapshot
}

/// Returns the current command-visible mining state.
pub fn mining_runtime_state() -> MiningRuntimeState {
    command_visible_state().mining
}

/// Collects projection-backed API views and drains at most one pending sample marker.
pub fn collect_projected_api_views(timestamp_ms: u64, response_time_ms: f64) -> ProjectedApiViews {
    collect_projected_api_views_with_sample_policy(timestamp_ms, response_time_ms, true)
}

/// Returns projection-backed `/api/system/info` data without consuming statistics markers.
pub fn projected_system_info(_timestamp_ms: u64) -> SystemInfoWire {
    let operator_snapshot_identity = reserve_operator_snapshot_identity();
    let (projection, _, block_found) = runtime_projection_for_api_views(false);
    let snapshot = collect_completed_api_snapshot(
        operator_snapshot_identity,
        projection.state().clone(),
        block_found,
    );
    project_system_info(snapshot, &projection)
}

/// Returns projection-backed `/api/system/statistics` data.
pub fn projected_statistics(timestamp_ms: u64) -> StatisticsWire {
    let views = collect_projected_api_views(timestamp_ms, 0.0);
    statistics_response(timestamp_ms, None, &views.statistics_samples)
}

/// Returns projection-backed `/api/system/scoreboard` data.
pub fn projected_scoreboard(timestamp_ms: u64) -> Vec<ScoreboardEntryWire> {
    let views = collect_projected_api_views_with_sample_policy(timestamp_ms, 0.0, false);
    scoreboard_response(&views.scoreboard_entries)
}

/// Returns projection-backed `/api/ws/live` payload JSON.
pub fn projected_live_telemetry_payload(timestamp_ms: u64) -> serde_json::Value {
    collect_projected_api_views_with_sample_policy(timestamp_ms, 0.0, false).telemetry_payload
}

/// Folds a lifecycle event into the shared runtime telemetry projection.
pub fn publish_runtime_lifecycle(lifecycle: PoolLifecycleStatus) -> ProjectionShareOutcome {
    publish_runtime_telemetry_event(|sequence, _generation| {
        RuntimeTelemetryEvent::LifecycleChanged {
            sequence,
            lifecycle,
        }
    })
}

/// Folds a pool-difficulty observation into the shared runtime projection.
pub fn publish_runtime_pool_difficulty(difficulty: PoolDifficulty) -> ProjectionShareOutcome {
    publish_runtime_telemetry_event(|sequence, _generation| {
        RuntimeTelemetryEvent::PoolDifficultyObserved {
            sequence,
            difficulty,
        }
    })
}

/// Folds hashrate inputs into the shared runtime projection.
pub fn publish_runtime_hashrate_inputs(inputs: HashrateInputs) -> ProjectionShareOutcome {
    publish_runtime_telemetry_event(|sequence, _generation| {
        RuntimeTelemetryEvent::HashrateObserved { sequence, inputs }
    })
}

/// Folds work-submission readiness into the shared runtime projection.
pub fn publish_runtime_work_submission_ready() -> ProjectionShareOutcome {
    publish_runtime_telemetry_event(|sequence, _generation| {
        RuntimeTelemetryEvent::WorkSubmissionReady { sequence }
    })
}

/// Folds a redaction-safe blocked-prerequisite label into the runtime projection.
pub fn publish_runtime_blocked(reason: &'static str) -> ProjectionShareOutcome {
    publish_runtime_telemetry_event(|sequence, _generation| RuntimeTelemetryEvent::Blocked {
        sequence,
        reason,
    })
}

/// Folds a producer-bound statistics sample marker into the runtime projection.
pub fn publish_runtime_bounded_sample_marker(
    source: RuntimeProjectionSampleSource,
) -> ProjectionShareOutcome {
    let timestamp_ms = crate::runtime_uptime::millis();
    publish_runtime_telemetry_event(|sequence, _generation| {
        RuntimeTelemetryEvent::BoundedSampleReady {
            sequence,
            timestamp_ms,
            source,
        }
    })
}

/// Folds a submit-response classification into the shared runtime projection.
pub fn publish_runtime_submit_classification(
    generation: PoolSessionGeneration,
    classification: SubmitClassification,
    maybe_share_difficulty: Option<ShareDifficulty>,
) -> ProjectionShareOutcome {
    publish_runtime_telemetry_event(|sequence, _current_generation| {
        RuntimeTelemetryEvent::SubmitClassified {
            sequence,
            generation,
            classification,
            maybe_share_difficulty,
        }
    })
}

/// Folds safe-stop postconditions before HTTP or WebSocket serialization.
pub fn publish_runtime_safe_stopped(reason: &'static str) -> ProjectionShareOutcome {
    publish_runtime_telemetry_event(|sequence, _generation| RuntimeTelemetryEvent::SafeStopped {
        sequence,
        reason,
    })
}

/// Returns the current identify mode used to plan the next identify command.
pub fn identify_mode() -> IdentifyMode {
    command_visible_state()
        .identify
        .mode_at(crate::runtime_uptime::millis())
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

/// Applies an API-visible identify command effect.
pub fn apply_identify_mode_command(effect: IdentifyModeEffect) {
    let now_ms = crate::runtime_uptime::millis();
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

fn publish_runtime_telemetry_event(
    build_event: impl FnOnce(RuntimeTelemetrySequence, PoolSessionGeneration) -> RuntimeTelemetryEvent,
) -> ProjectionShareOutcome {
    mutate_command_visible_state_with_result(ProjectionShareOutcome::NoCounterChange, |state| {
        let sequence = state.next_runtime_sequence();
        let generation = state.runtime_projection.current_generation();
        let outcome = state
            .runtime_projection
            .fold(build_event(sequence, generation));
        state.mining = state.runtime_projection.state().clone();
        outcome
    })
}

fn collect_projected_api_views_with_sample_policy(
    timestamp_ms: u64,
    response_time_ms: f64,
    drain_sample_marker: bool,
) -> ProjectedApiViews {
    let operator_snapshot_identity = reserve_operator_snapshot_identity();
    let (projection, maybe_sample_marker, block_found) =
        runtime_projection_for_api_views(drain_sample_marker);
    let snapshot = collect_completed_api_snapshot(
        operator_snapshot_identity,
        projection.state().clone(),
        block_found,
    );
    project_api_views(
        snapshot,
        &projection,
        maybe_sample_marker,
        timestamp_ms,
        response_time_ms,
    )
}

fn runtime_projection_for_api_views(
    drain_sample_marker: bool,
) -> (
    RuntimeTelemetryProjection,
    Option<RuntimeProjectionSampleMarker>,
    BlockFoundNotificationState,
) {
    mutate_command_visible_state_with_result(
        (
            RuntimeTelemetryProjection::new(PoolSessionGeneration::initial()),
            None,
            BlockFoundNotificationState {
                block_found: 0,
                show_new_block: false,
            },
        ),
        |state| {
            let maybe_sample_marker = if drain_sample_marker {
                state.drain_pending_runtime_sample_marker()
            } else {
                None
            };
            (
                state.runtime_projection.clone(),
                maybe_sample_marker,
                state.block_found,
            )
        },
    )
}

fn reserve_operator_snapshot_identity() -> OperatorSnapshotIdentity {
    let sequence =
        OPERATOR_SNAPSHOT_SEQUENCE.get_or_init(|| Mutex::new(OperatorSnapshotSequence::new()));
    let mut sequence = match sequence.lock() {
        Ok(sequence) => sequence,
        Err(poisoned) => {
            log::warn!("operator_snapshot_sequence=recovered reason=mutex_poisoned");
            poisoned.into_inner()
        }
    };
    sequence
        .next_identity(crate::boot_evidence::operator_snapshot_boot_session())
        .expect("operator snapshot revision cannot exhaust within one boot")
}

fn retain_completed_operator_snapshot(identity: OperatorSnapshotIdentity) {
    let marker = identity.retained_marker();
    log::info!("{marker}");
    crate::log_buffer::append_runtime_log_line(&marker);
}

fn mutate_command_visible_state_with_result<T>(
    fallback: T,
    mutate: impl FnOnce(&mut CommandVisibleState) -> T,
) -> T {
    let state = COMMAND_VISIBLE_STATE.get_or_init(|| Mutex::new(CommandVisibleState::default()));
    let Ok(mut state) = state.lock() else {
        log::warn!("axeos_runtime_state=unavailable reason=mutex_poisoned");
        return fallback;
    };

    mutate(&mut state)
}

impl CommandVisibleState {
    fn next_runtime_sequence(&mut self) -> RuntimeTelemetrySequence {
        let sequence = RuntimeTelemetrySequence::new(self.next_runtime_sequence);
        self.next_runtime_sequence = self.next_runtime_sequence.saturating_add(1);
        sequence
    }

    fn drain_pending_runtime_sample_marker(&mut self) -> Option<RuntimeProjectionSampleMarker> {
        self.runtime_projection.drain_pending_sample_marker()
    }
}

fn apply_settings_snapshot(snapshot: &mut ApiSnapshot) {
    let confirmed_settings = crate::settings_adapter::current_settings_snapshot();
    let loaded = reload_snapshot(&confirmed_settings);

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

fn collect_platform_snapshot(
    mut platform: PlatformSnapshot,
    identity: &PlatformIdentity,
) -> PlatformSnapshot {
    platform.version = crate::build_label().to_owned();
    platform.semantic_version = crate::semantic_version().to_owned();
    platform.source_commit = crate::firmware_commit().to_owned();
    platform.reference_commit = crate::reference_commit().to_owned();
    platform.app_elf_sha256 = crate::app_elf_sha256();
    platform.build_channel = crate::build_channel().to_owned();
    platform.source_dirty = crate::source_dirty();
    platform.maybe_release_tag = crate::maybe_release_tag().map(str::to_owned);
    platform.idf_version = compatibility_string(&identity.esp_idf_version);
    platform.axe_os_version = compatibility_string(&identity.axe_os_static_asset);
    platform.reset_reason = identity.reset_reason.maybe_value().map_or_else(
        || "Unavailable".to_owned(),
        |reason| reason.compatibility_text().to_owned(),
    );
    platform.running_partition = compatibility_string(&identity.running_partition);
    platform.psram_available = identity
        .psram_available
        .maybe_value()
        .copied()
        .unwrap_or(false);
    platform.free_heap = compatibility_u64(&identity.internal_heap_free_bytes);
    platform.free_heap_internal = compatibility_u64(&identity.internal_heap_free_bytes);
    platform.free_heap_spiram = 0;
    platform.min_free_heap = compatibility_u64(&identity.internal_heap_minimum_free_bytes);
    platform.max_alloc_heap = compatibility_u64(&identity.internal_heap_largest_free_block_bytes);
    platform.uptime_seconds = compatibility_u64(&identity.uptime_milliseconds) / 1_000;
    platform
}

fn compatibility_string(fact: &PlatformFact<String>) -> String {
    fact.maybe_value()
        .cloned()
        .unwrap_or_else(|| "Unavailable".to_owned())
}

fn compatibility_u64(fact: &PlatformFact<u64>) -> u64 {
    fact.maybe_value().copied().unwrap_or(0)
}

#[cfg(test)]
pub(crate) fn reset_command_visible_state_for_test() {
    mutate_command_visible_state(|state| {
        *state = CommandVisibleState::default();
    });
}

#[cfg(test)]
fn drain_pending_runtime_sample_marker_for_test() -> Option<RuntimeProjectionSampleMarker> {
    mutate_command_visible_state_with_result(None, |state| {
        state.drain_pending_runtime_sample_marker()
    })
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
        assert_eq!(
            first_marker.source,
            RuntimeProjectionSampleSource::RuntimeEvent
        );
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
        assert_eq!(
            snapshot.mining.mining_activity,
            MiningActivityStatus::SafeBlocked
        );
        assert_eq!(snapshot.mining.work_submission, WorkSubmissionGate::Blocked);
        assert_eq!(
            snapshot.mining.maybe_blocked_reason,
            Some("phase25_safe_stop")
        );
    }

    #[test]
    fn projected_route_helpers_use_projection_state_and_drain_samples_once() {
        // Arrange
        reset_command_visible_state_for_test();
        let inputs = HashrateInputs {
            hashes_done: 4_000,
            elapsed_ms: 2_000,
            rolling_hashrate_hs: 2_000.0,
        };
        publish_runtime_hashrate_inputs(inputs);
        publish_runtime_lifecycle(PoolLifecycleStatus::Active);
        publish_runtime_bounded_sample_marker(RuntimeProjectionSampleSource::RuntimeEvent);

        // Act
        let system_info = projected_system_info(50_000);
        let first_statistics = projected_statistics(50_000);
        let second_statistics = projected_statistics(50_500);
        let scoreboard = projected_scoreboard(50_000);

        // Assert
        assert_eq!(system_info.hash_rate, 2.0);
        assert_eq!(system_info.pool_connection_info, "active");
        assert_eq!(first_statistics.statistics.len(), 1);
        assert!(second_statistics.statistics.is_empty());
        assert!(scoreboard.is_empty());
    }

    #[test]
    fn projected_live_telemetry_payload_reflects_safe_stop_state() {
        // Arrange
        reset_command_visible_state_for_test();
        publish_runtime_work_submission_ready();

        // Act
        publish_runtime_blocked("phase25_safe_stop");
        publish_runtime_safe_stopped("phase25_safe_stop");
        let payload = projected_live_telemetry_payload(60_000);
        let rendered = payload.to_string();

        // Assert
        assert_eq!(payload["miningPaused"], serde_json::Value::Bool(true));
        assert_eq!(
            payload["poolConnectionInfo"],
            serde_json::Value::String("disconnected".to_owned())
        );
        assert!(!rendered.contains(":\"active\""));
    }
}
