//! Firmware collection boundary for pure AxeOS API response snapshots.

mod screen;
mod screen_projection;

pub use screen::collect_screen_snapshot;

use std::sync::{Mutex, OnceLock};

use crate::log_buffer::RetainedPairStorageError;
use crate::operator_snapshot_publication::OperatorSnapshotPublisher;
use bitaxe_api::{
    apply_block_found_dismiss_effect, apply_identify_mode_effect,
    apply_mining_operator_intent_effect, project_api_views, project_system_info,
    scoreboard_response, statistics_response, ApiSnapshot, BlockFoundDismissEffect,
    BlockFoundNotificationState, IdentifyMode, IdentifyModeEffect, IdentifyModeState,
    MiningOperatorIntentEffect, OperatorSnapshotIdentity, OperatorSnapshotLockHealth,
    OperatorSnapshotPublishError, PlatformFact, PlatformIdentity, PlatformSnapshot,
    ProjectedApiViews, SafeTelemetrySnapshot, ScoreboardEntryWire, StatisticsHistory,
    StatisticsSample, StatisticsWire, SystemInfoSettingsSnapshot, SystemInfoWire,
};
use bitaxe_config::{reload_snapshot, LoadedValue};
use bitaxe_stratum::v1::telemetry_projection::RuntimeProjectionSampleMarker;
use bitaxe_stratum::v1::{
    production_session::ProductionSessionSnapshot, production_work::PoolSessionGeneration,
    state::MiningRuntimeState, telemetry_projection::RuntimeTelemetryProjection,
};
static COMMAND_VISIBLE_STATE: OnceLock<Mutex<CommandVisibleState>> = OnceLock::new();
static OPERATOR_SNAPSHOT_PUBLISHER: OnceLock<OperatorSnapshotPublisher> = OnceLock::new();
static STATISTICS_HISTORY: OnceLock<Mutex<StatisticsHistory>> = OnceLock::new();

struct OperatorSnapshotCandidate {
    projection: RuntimeTelemetryProjection,
    maybe_sample_marker: Option<RuntimeProjectionSampleMarker>,
    block_found: BlockFoundNotificationState,
    platform_identity: PlatformIdentity,
    platform: PlatformSnapshot,
    runtime_health: bitaxe_core::runtime_health::RuntimeHealthSnapshot,
    safe_telemetry: SafeTelemetrySnapshot,
    settings: SettingsProjection,
    wifi: crate::wifi_adapter::WifiRuntimeSnapshot,
}

struct SettingsProjection {
    maybe_hostname: Option<String>,
    maybe_frequency: Option<f64>,
    maybe_voltage: Option<u16>,
    maybe_auto_fan_speed: Option<bool>,
    maybe_manual_fan_speed: Option<u16>,
    start_mining_on_boot: bool,
    system_info: SystemInfoSettingsSnapshot,
}

struct CompletedOperatorSnapshot<T> {
    output: T,
    retained_marker: String,
    retained_runtime_health: String,
}

#[derive(Debug, Clone, PartialEq)]
struct CommandVisibleState {
    mining: MiningRuntimeState,
    runtime_projection: RuntimeTelemetryProjection,
    identify: IdentifyModeState,
    block_found: BlockFoundNotificationState,
    work_received: u64,
}

impl Default for CommandVisibleState {
    fn default() -> Self {
        Self {
            mining: MiningRuntimeState::default(),
            runtime_projection: RuntimeTelemetryProjection::new(PoolSessionGeneration::initial()),
            identify: IdentifyModeState::inactive(),
            block_found: BlockFoundNotificationState {
                block_found: 0,
                show_new_block: false,
            },
            work_received: 0,
        }
    }
}

/// Collects current firmware facts and overlays them on the safe Ultra 205 API
/// snapshot used by the pure contract mappers.
pub fn collect_api_snapshot() -> ApiSnapshot {
    match publish_operator_snapshot(
        false,
        |snapshot, _projection, _maybe_sample_marker| snapshot,
        Ok::<ApiSnapshot, core::convert::Infallible>,
    ) {
        Ok(snapshot) => snapshot,
        Err(error) => panic!("operator snapshot publication failed: {error:?}"),
    }
}

fn complete_operator_snapshot(
    candidate: OperatorSnapshotCandidate,
    operator_snapshot_identity: OperatorSnapshotIdentity,
) -> ApiSnapshot {
    let mut snapshot = complete_api_snapshot(candidate);
    snapshot.operator_snapshot_identity = operator_snapshot_identity;
    snapshot
}

fn complete_api_snapshot(candidate: OperatorSnapshotCandidate) -> ApiSnapshot {
    let mut snapshot = ApiSnapshot::safe_ultra_205();
    snapshot.mining = candidate.projection.state().clone();
    snapshot.block_found = candidate.block_found;
    snapshot.platform_identity = candidate.platform_identity;
    snapshot.platform = candidate.platform;
    snapshot.runtime_health = candidate.runtime_health;
    snapshot.safe_telemetry = candidate.safe_telemetry;
    apply_wifi_snapshot(&mut snapshot, candidate.wifi);
    apply_settings_snapshot(&mut snapshot, candidate.settings);
    snapshot
}

/// Returns the current command-visible mining state.
pub fn mining_runtime_state() -> MiningRuntimeState {
    command_visible_state().mining
}

/// Applies the persisted boot preference to this boot's initial intent.
pub fn apply_boot_mining_preference(start_mining_on_boot: bool) {
    use bitaxe_stratum::v1::state::MiningOperatorIntent;

    mutate_command_visible_state(|state| {
        state.mining.set_operator_intent(if start_mining_on_boot {
            MiningOperatorIntent::Run
        } else {
            MiningOperatorIntent::Paused
        });
    });
}

/// Collects projection-backed API views and drains at most one pending sample marker.
pub fn collect_projected_api_views(timestamp_ms: u64, response_time_ms: f64) -> ProjectedApiViews {
    collect_projected_api_views_with_sample_policy(timestamp_ms, response_time_ms, true)
}

/// Returns projection-backed `/api/system/info` data without consuming statistics markers.
pub fn publish_projected_system_info<T, E>(
    _timestamp_ms: u64,
    issue: impl FnOnce(SystemInfoWire) -> Result<T, E>,
) -> Result<T, OperatorSnapshotPublishError<RetainedPairStorageError, E>> {
    publish_operator_snapshot(
        false,
        |snapshot, projection, _maybe_sample_marker| project_system_info(snapshot, &projection),
        issue,
    )
}

/// Returns projection-backed `/api/system/statistics` data.
pub fn projected_statistics(timestamp_ms: u64) -> StatisticsWire {
    statistics_response(timestamp_ms, None, &statistics_samples())
}

/// Records one producer-cadence statistics sample or clears disabled history.
pub fn record_statistics_sample(timestamp_ms: u64, frequency_seconds: u16) {
    if frequency_seconds == 0 {
        mutate_statistics_history(|history| {
            history.disable();
        });
        return;
    }

    let snapshot = complete_api_snapshot(collect_operator_snapshot_candidate(false));
    let sample = StatisticsSample::from_snapshot(&snapshot, timestamp_ms, 0.0);
    mutate_statistics_history(|history| {
        if let Err(error) = history.record(sample, frequency_seconds) {
            log::warn!("statistics_history=sample_rejected category={error:?}");
        }
    });
}

/// Returns projection-backed `/api/system/scoreboard` data.
pub fn projected_scoreboard(_timestamp_ms: u64) -> Vec<ScoreboardEntryWire> {
    scoreboard_response(&crate::scoreboard_adapter::entries())
}

/// Returns projection-backed `/api/ws/live` payload JSON.
pub fn publish_projected_live_telemetry_payload<T, E>(
    timestamp_ms: u64,
    issue: impl FnOnce(serde_json::Value) -> Result<T, E>,
) -> Result<T, OperatorSnapshotPublishError<RetainedPairStorageError, E>> {
    publish_operator_snapshot(
        false,
        |snapshot, projection, maybe_sample_marker| {
            project_api_views(
                snapshot,
                &projection,
                maybe_sample_marker,
                timestamp_ms,
                0.0,
            )
        },
        |views| issue(views.telemetry_payload),
    )
}

/// Publishes the sole owner's immutable mining-session snapshot.
pub fn publish_production_session_snapshot(snapshot: ProductionSessionSnapshot) {
    mutate_command_visible_state(|state| {
        let hashrate = state.mining.hashrate_inputs.clone();
        state.work_received = snapshot.job_transition.pool_notify_count;
        state.mining = snapshot.mining;
        state.mining.record_hashrate_inputs(hashrate);
        state
            .runtime_projection
            .replace_session_state(state.mining.clone());
    });
}

/// Publishes a monitor sample without replacing the production-session state.
pub fn publish_hashrate_snapshot(snapshot: bitaxe_core::hashrate::HashrateSnapshot) {
    mutate_command_visible_state(|state| {
        state.mining.record_hashrate_inputs(snapshot);
        state
            .runtime_projection
            .replace_session_state(state.mining.clone());
    });
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

/// Applies current-boot operator intent without deriving mining state.
pub fn apply_mining_operator_intent_command(effect: MiningOperatorIntentEffect) {
    mutate_command_visible_state(|state| {
        apply_mining_operator_intent_effect(&mut state.mining, effect);
    });
}

/// Applies an API-visible identify command effect.
pub fn apply_identify_mode_command(effect: IdentifyModeEffect) {
    let now_ms = crate::runtime_uptime::millis();
    mutate_command_visible_state(|state| {
        apply_identify_mode_effect(&mut state.identify, effect, now_ms);
    });
}

/// Result of atomically testing and cancelling identify for a short click.
pub enum ButtonIdentifyCancellation {
    Cancelled,
    Inactive,
    StateUnavailable,
}

/// Atomically cancels identify only when it is active at this instant.
pub fn cancel_identify_if_active_at(now_ms: u64) -> ButtonIdentifyCancellation {
    mutate_command_visible_state_with_result(
        ButtonIdentifyCancellation::StateUnavailable,
        |state| {
            if state.identify.mode_at(now_ms) != IdentifyMode::Active {
                return ButtonIdentifyCancellation::Inactive;
            }
            apply_identify_mode_effect(&mut state.identify, IdentifyModeEffect::Disable, now_ms);
            ButtonIdentifyCancellation::Cancelled
        },
    )
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

fn statistics_samples() -> Vec<StatisticsSample> {
    let history = STATISTICS_HISTORY.get_or_init(|| Mutex::new(StatisticsHistory::new()));
    match history.lock() {
        Ok(history) => history.samples().to_vec(),
        Err(poisoned) => {
            log::warn!("statistics_history=degraded reason=mutex_poisoned_inner_retained");
            poisoned.into_inner().samples().to_vec()
        }
    }
}

fn mutate_statistics_history(mutate: impl FnOnce(&mut StatisticsHistory)) {
    let history = STATISTICS_HISTORY.get_or_init(|| Mutex::new(StatisticsHistory::new()));
    match history.lock() {
        Ok(mut history) => mutate(&mut history),
        Err(poisoned) => {
            log::warn!("statistics_history=degraded reason=mutex_poisoned_inner_retained");
            mutate(&mut poisoned.into_inner());
        }
    }
}

fn mutate_command_visible_state(mutate: impl FnOnce(&mut CommandVisibleState)) {
    let state = COMMAND_VISIBLE_STATE.get_or_init(|| Mutex::new(CommandVisibleState::default()));
    let Ok(mut state) = state.lock() else {
        log::warn!("axeos_runtime_state=unavailable reason=mutex_poisoned");
        return;
    };

    mutate(&mut state);
}

fn collect_projected_api_views_with_sample_policy(
    timestamp_ms: u64,
    response_time_ms: f64,
    drain_sample_marker: bool,
) -> ProjectedApiViews {
    match publish_operator_snapshot(
        drain_sample_marker,
        |snapshot, projection, maybe_sample_marker| {
            project_api_views(
                snapshot,
                &projection,
                maybe_sample_marker,
                timestamp_ms,
                response_time_ms,
            )
        },
        Ok::<ProjectedApiViews, core::convert::Infallible>,
    ) {
        Ok(views) => views,
        Err(error) => panic!("operator snapshot publication failed: {error:?}"),
    }
}

fn publish_operator_snapshot<Publication, T, E>(
    drain_sample_marker: bool,
    project: impl FnOnce(
        ApiSnapshot,
        RuntimeTelemetryProjection,
        Option<RuntimeProjectionSampleMarker>,
    ) -> Publication,
    issue: impl FnOnce(Publication) -> Result<T, E>,
) -> Result<T, OperatorSnapshotPublishError<RetainedPairStorageError, E>> {
    let publisher = OPERATOR_SNAPSHOT_PUBLISHER.get_or_init(OperatorSnapshotPublisher::new);
    let result = publisher.publish(
        crate::boot_evidence::operator_snapshot_boot_session(),
        || collect_operator_snapshot_candidate(drain_sample_marker),
        |candidate, identity| {
            let maybe_sample_marker = candidate.maybe_sample_marker;
            let projection = candidate.projection.clone();
            let snapshot = complete_operator_snapshot(candidate, identity);
            let retained_marker = identity.retained_marker();
            let retained_runtime_health = bitaxe_api::retained_runtime_health_record(
                identity.boot_session(),
                identity.revision(),
                &snapshot.runtime_health,
            );
            CompletedOperatorSnapshot {
                output: project(snapshot, projection, maybe_sample_marker),
                retained_marker,
                retained_runtime_health,
            }
        },
        |publication| {
            crate::operator_snapshot_retention::retain_completed_operator_snapshot(
                &publication.retained_marker,
                &publication.retained_runtime_health,
            )
        },
        |publication| issue(publication.output),
    );
    log_recovered_publication_lock(&result);
    result.map(|publication| publication.output)
}

fn collect_operator_snapshot_candidate(drain_sample_marker: bool) -> OperatorSnapshotCandidate {
    let (projection, maybe_sample_marker, block_found) =
        runtime_projection_for_api_views(drain_sample_marker);
    let platform_identity = crate::platform_identity::collect();
    let platform =
        collect_platform_snapshot(PlatformSnapshot::safe_ultra_205(), &platform_identity);
    let runtime_health = crate::runtime_health_adapter::collect(crate::runtime_uptime::millis());
    let observations = crate::safety_adapter::observation_snapshot();
    let safe_telemetry = SafeTelemetrySnapshot::from_observations(&observations);
    let settings = collect_settings_projection();
    let wifi = crate::wifi_adapter::current_wifi_snapshot();
    OperatorSnapshotCandidate {
        projection,
        maybe_sample_marker,
        block_found,
        platform_identity,
        platform,
        runtime_health,
        safe_telemetry,
        settings,
        wifi,
    }
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
                state.maybe_drain_pending_runtime_sample_marker()
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

fn log_recovered_publication_lock<T, RetentionError, IssueError>(
    result: &Result<
        bitaxe_api::OperatorSnapshotPublication<T>,
        OperatorSnapshotPublishError<RetentionError, IssueError>,
    >,
) {
    let recovered = match result {
        Ok(publication) => publication.lock_health == OperatorSnapshotLockHealth::RecoveredPoison,
        Err(error) => {
            error.maybe_lock_health() == Some(OperatorSnapshotLockHealth::RecoveredPoison)
        }
    };
    if recovered {
        log::warn!("operator_snapshot_publisher=recovered reason=mutex_poisoned");
    }
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
    fn maybe_drain_pending_runtime_sample_marker(
        &mut self,
    ) -> Option<RuntimeProjectionSampleMarker> {
        self.runtime_projection.maybe_drain_pending_sample_marker()
    }
}

fn collect_settings_projection() -> SettingsProjection {
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
        system_info: SystemInfoSettingsSnapshot::from_nvs_snapshot(
            &crate::settings_adapter::current_system_info_settings_snapshot(),
        ),
    }
}

fn apply_settings_snapshot(snapshot: &mut ApiSnapshot, settings: SettingsProjection) {
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

fn apply_wifi_snapshot(snapshot: &mut ApiSnapshot, wifi: crate::wifi_adapter::WifiRuntimeSnapshot) {
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

fn collect_platform_snapshot(
    mut platform: PlatformSnapshot,
    identity: &PlatformIdentity,
) -> PlatformSnapshot {
    platform.boot_ordinal = crate::boot_evidence::operator_snapshot_boot_ordinal();
    platform.reset_reason_category =
        crate::boot_evidence::operator_snapshot_reset_reason_category()
            .label()
            .to_owned();
    platform.version = crate::build_label().to_owned();
    platform.semantic_version = crate::semantic_version().to_owned();
    platform.source_commit = crate::firmware_commit().to_owned();
    platform.reference_commit = crate::reference_commit().to_owned();
    platform.app_elf_sha256 = crate::app_elf_sha256();
    platform.build_timestamp_utc = crate::build_timestamp_utc().to_owned();
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
mod tests {
    use super::*;
}
