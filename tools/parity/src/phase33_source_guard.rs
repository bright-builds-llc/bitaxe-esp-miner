const SETTINGS_ADAPTER_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/settings_adapter.rs");
const HTTP_API_SOURCE: &str = include_str!("../../../firmware/bitaxe/src/http_api.rs");
const RUNTIME_SNAPSHOT_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/runtime_snapshot.rs");
const BOOT_EVIDENCE_SOURCE: &str = include_str!("../../../firmware/bitaxe/src/boot_evidence.rs");
const RTC_BOOT_ORDINAL_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/rtc_boot_ordinal.rs");
const WIFI_ADAPTER_SOURCE: &str = include_str!("../../../firmware/bitaxe/src/wifi_adapter.rs");
const MAIN_SOURCE: &str = include_str!("../../../firmware/bitaxe/src/main.rs");

#[test]
fn phase33_settings_source_guard_serializes_reload_through_publication() {
    // Arrange
    let transaction = source_between(
        SETTINGS_ADAPTER_SOURCE,
        "impl SettingsPersistenceTransaction for FirmwareSettingsTransaction",
        "impl SettingsPersistenceAdapter for FirmwareSettingsAdapter",
    );
    let begin = source_between(
        SETTINGS_ADAPTER_SOURCE,
        "fn begin_transaction",
        "/// Best-effort startup load",
    );

    // Act / Assert
    assert!(SETTINGS_ADAPTER_SOURCE.contains("static SETTINGS_TRANSACTION_LOCK: Mutex<()>"));
    assert!(SETTINGS_ADAPTER_SOURCE.contains("_transaction_guard: MutexGuard<'static, ()>"));
    assert!(
        begin
            .find("SETTINGS_TRANSACTION_LOCK")
            .expect("transaction lock")
            < begin.find("EspNvs::new").expect("writable NVS open")
    );
    assert!(transaction.contains("NVS_NAMESPACE, false"));
    assert!(transaction.contains("read_current_settings_snapshot_strict"));
    assert!(transaction.contains("confirm_hostname_snapshot(candidate)"));
    assert!(transaction.contains(".publish(candidate.into_snapshot())"));
    assert!(!transaction.contains("refresh_current_settings_snapshot_best_effort"));
}

#[test]
fn phase33_settings_source_guard_retains_poisoned_confirmed_snapshot() {
    // Arrange
    let read = source_between(
        SETTINGS_ADAPTER_SOURCE,
        "pub fn current_settings_snapshot",
        "fn current_snapshot_cell",
    );

    // Act / Assert
    assert!(read.contains("ConfirmedSnapshotReadHealth::PoisonRecovered"));
    assert!(read.contains("mutex_poisoned_inner_retained"));
    assert!(read.contains("read.into_snapshot()"));
    assert!(!read.contains("NvsSnapshot::new()"));
}

#[test]
fn phase33_settings_source_guard_keeps_candidate_loading_fallible_and_nonpublishing() {
    // Arrange
    let strict_load = source_between(
        SETTINGS_ADAPTER_SOURCE,
        "fn read_current_settings_snapshot_strict",
        "fn read_stored_value_best_effort",
    );
    let strict_value = source_between(
        SETTINGS_ADAPTER_SOURCE,
        "fn read_stored_value_strict",
        "fn read_string_value_strict",
    );

    // Act / Assert
    assert!(strict_load.contains("Result<NvsSnapshot, SettingsAdapterFailure>"));
    assert!(strict_load.contains("nvs.find_key(key).map_err(settings_failure)?"));
    assert!(strict_load.contains("read_stored_value_strict(nvs, key, stored_type)?"));
    assert!(strict_value.contains("settings key has unsupported storage type"));
    assert!(!strict_load.contains("current_snapshot_cell"));
    assert!(!strict_load.contains("apply_writes"));
}

#[test]
fn phase33_settings_source_guard_closes_route_authority_and_optimistic_overlays() {
    // Arrange
    let handler = source_between(
        HTTP_API_SOURCE,
        "fn handle_settings_patch",
        "fn handle_logs_download",
    );

    // Act / Assert
    assert!(handler.contains("decide_v12_settings_body(&body)"));
    assert!(handler.contains("V12SettingsDecision::CompatibilityOnly"));
    assert!(handler.contains("SettingsPersistencePlan::for_hostname(hostname)"));
    let decision = handler
        .find("decide_v12_settings_body(&body)")
        .expect("closed authority decision");
    let compatibility = handler
        .find("V12SettingsDecision::CompatibilityOnly")
        .expect("compatibility-only branch");
    let adapter = handler
        .find("FirmwareSettingsAdapter::open()")
        .expect("authorized adapter open");
    assert!(decision < compatibility && compatibility < adapter);
    assert!(!handler.contains("plan_settings_patch_body(&body)"));
    assert!(!handler.contains("from_accepted_patch"));
    assert!(!HTTP_API_SOURCE.contains("apply_persisted_settings_writes"));
    assert!(!SETTINGS_ADAPTER_SOURCE.contains("apply_persisted_settings_writes"));
}

#[test]
fn phase33_settings_source_guard_keeps_compatibility_and_errors_effect_free() {
    // Arrange
    let handler = source_between(
        HTTP_API_SOURCE,
        "fn handle_settings_patch",
        "fn handle_logs_download",
    );
    let compatibility = source_between(
        handler,
        "V12SettingsDecision::CompatibilityOnly",
        "let mut adapter",
    );

    // Act / Assert
    assert!(compatibility.contains("SettingsPublicResponse::EmptySuccess"));
    assert!(compatibility.contains("return Ok(())"));
    assert!(!compatibility.contains("FirmwareSettingsAdapter::open"));
    assert!(!compatibility.contains("prepare_settings_effects"));
    assert!(handler.contains("error.public_error().body()"));
    assert!(!handler.contains("body={"));
    assert!(!handler.contains("hostname.as_str()"));
}

#[test]
fn phase33_settings_source_guard_responds_after_publish_and_projects_confirmed_truth() {
    // Arrange
    let handler = source_between(
        HTTP_API_SOURCE,
        "fn handle_settings_patch",
        "fn handle_logs_download",
    );
    let execute = handler
        .find("execute_settings_persistence_plan")
        .expect("confirmed executor call");
    let ownership = handler
        .find("maybe_acquire_best_effort_effect_lease(prepare_settings_effects)")
        .expect("best-effort worker ownership before response");
    let response = handler
        .find("send_settings_response(request, success.public_response())")
        .expect("public success response");
    let release = handler
        .find(".release_after_response()")
        .expect("post-response effect release");
    let confirmed_success = source_between(
        handler,
        "axeos_settings_patch=persistence_confirmed",
        "send_settings_response(request, success.public_response())",
    );

    // Act / Assert
    assert!(execute < ownership && ownership < response && response < release);
    assert!(confirmed_success.contains("effects_degraded category=worker_unavailable"));
    assert!(!confirmed_success.contains("send_text_error"));
    assert!(!handler.contains("settings effect worker unavailable after ownership"));
    assert!(RUNTIME_SNAPSHOT_SOURCE
        .contains("let confirmed_settings = crate::settings_adapter::current_settings_snapshot()"));
    assert!(RUNTIME_SNAPSHOT_SOURCE.contains("reload_snapshot(&confirmed_settings)"));
    assert!(RUNTIME_SNAPSHOT_SOURCE.contains("snapshot.platform.hostname = hostname"));
    assert!(!RUNTIME_SNAPSHOT_SOURCE.contains("apply_persisted_settings_writes"));
}

#[test]
fn phase33_system_info_source_guard_avoids_duplicate_full_view_materialization() {
    // Arrange
    let projection = source_between(
        RUNTIME_SNAPSHOT_SOURCE,
        "pub fn publish_projected_system_info",
        "/// Returns projection-backed `/api/system/statistics` data.",
    );

    // Act / Assert
    assert!(projection.contains("publish_operator_snapshot("));
    assert!(projection.contains("project_system_info(snapshot, &projection)"));
    assert!(!projection.contains("collect_projected_api_views_with_sample_policy"));
    assert!(!projection.contains("ProjectedApiViews"));
    assert!(!projection.contains("telemetry_payload"));
}

#[test]
fn phase33_settings_source_guard_limits_post_response_effect_to_hostname() {
    // Arrange
    let apply_effects = source_between(
        HTTP_API_SOURCE,
        "fn apply_settings_effects",
        "fn apply_hostname_effect",
    );

    // Act / Assert
    assert!(apply_effects.contains("BestEffortApplyHostname"));
    assert!(apply_effects.contains("apply_hostname_effect(hostname)"));
    assert!(!apply_effects.contains("controlled_mining_runtime"));
    assert!(!apply_effects.contains("live_stratum_runtime"));
    assert!(!apply_effects.contains("refresh_from_settings"));
}

#[test]
fn phase33_restart_source_guard_completes_response_before_delayed_restart() {
    // Arrange
    let startup = source_between(
        HTTP_API_SOURCE,
        "pub fn start_http_api",
        "fn start_live_telemetry_cadence_task",
    );
    let handler = source_between(
        HTTP_API_SOURCE,
        "fn handle_command",
        "fn handle_firmware_ota_update",
    );
    let prepare = source_between(
        HTTP_API_SOURCE,
        "fn prepare_deferred_command_effect",
        "fn apply_command_effect",
    );
    let release = source_between(
        HTTP_API_SOURCE,
        "fn apply_command_effect",
        "fn prepare_restart_after_response",
    );
    let worker = source_between(
        HTTP_API_SOURCE,
        "fn initialize_deferred_effect_worker",
        "fn apply_settings_effects",
    );
    let restart = source_between(
        HTTP_API_SOURCE,
        "fn prepare_restart_after_response",
        "fn record_firmware_ota_status",
    );

    // Act
    let ownership = handler
        .find("prepare_deferred_command_effect(&effect)?")
        .expect("worker ownership");
    let response = handler
        .find("send_json(request, &plan.response)?")
        .expect("response write");
    let effect = handler
        .find("apply_command_effect(effect, maybe_deferred_effect)?")
        .expect("effect release");
    let delay = worker
        .find("std::thread::sleep(Duration::from_millis(RESTART_POST_RESPONSE_DELAY_MS))")
        .expect("post-response delay");
    let marker = worker
        .find("axeos_command_effect=restart_after_response")
        .expect("restart marker");
    let restart_call = worker.find("sys::esp_restart()").expect("restart call");

    // Assert
    assert!(HTTP_API_SOURCE.contains("const RESTART_POST_RESPONSE_DELAY_MS: u64 = 1_000;"));
    assert!(HTTP_API_SOURCE.contains("const DEFERRED_EFFECT_THREAD_STACK_BYTES: usize = 8 * 1024;"));
    assert!(
        startup
            .find("initialize_deferred_effect_worker()")
            .expect("worker startup")
            < startup
                .find("EspHttpServer::new")
                .expect("HTTP server startup")
    );
    assert!(ownership < response && response < effect);
    assert!(prepare.contains("prepare_restart_after_response().map(Some)"));
    assert!(release.contains("deferred_effect.release_after_response()"));
    assert!(worker.contains(".name(\"deferred-effects\".to_owned())"));
    assert!(worker.contains(".stack_size(DEFERRED_EFFECT_THREAD_STACK_BYTES)"));
    assert!(restart.contains(".acquire(DeferredFirmwareEffect::Restart)"));
    assert!(!restart.contains("sys::esp_restart()"));
    assert!(delay < marker && marker < restart_call);
    assert!(!HTTP_API_SOURCE.contains("fallback=inline"));
}

#[test]
fn phase33_boot_identity_source_guard_initializes_rtc_before_workers() {
    // Arrange
    let main = source_between(MAIN_SOURCE, "fn main()", "let safe_state");
    let initialize = source_between(
        BOOT_EVIDENCE_SOURCE,
        "pub fn initialize_observer()",
        "/// Publishes the connected HTTP origin",
    );

    // Act / Assert
    assert!(RTC_BOOT_ORDINAL_SOURCE.contains("#[link_section = \".rtc_noinit\"]"));
    assert!(RTC_BOOT_ORDINAL_SOURCE.contains("read_volatile"));
    assert!(RTC_BOOT_ORDINAL_SOURCE.contains("write_volatile"));
    assert!(initialize.contains("rtc_boot_ordinal::initialize(reset_reason)"));
    assert!(
        initialize
            .find("rtc_boot_ordinal::initialize(reset_reason)")
            .expect("RTC init")
            < initialize
                .find(".spawn(observe_boot_lifetime)")
                .expect("observer spawn")
    );
    assert!(
        main.find("boot_evidence::initialize_observer()")
            .expect("boot observer")
            < main.find("let safe_state").unwrap_or(main.len())
    );
}

#[test]
fn phase33_boot_identity_source_guard_replays_identity_and_typed_origin() {
    // Arrange / Act / Assert
    assert!(BOOT_EVIDENCE_SOURCE.contains("emit_boot_identity(nonce, ordinal, reset_reason"));
    assert!(BOOT_EVIDENCE_SOURCE.contains("BOOT_EVIDENCE_INTERVAL_MS"));
    assert!(BOOT_EVIDENCE_SOURCE.contains("ORIGIN_REPLAY_WINDOW_MS"));
    assert!(BOOT_EVIDENCE_SOURCE.contains("runtime_origin_marker"));
    assert!(WIFI_ADAPTER_SOURCE.contains("boot_evidence::publish_connected_origin"));
    assert!(BOOT_EVIDENCE_SOURCE.contains("RuntimeHeartbeatModel"));
}

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let (_, tail) = source.split_once(start).expect("source start delimiter");
    let (section, _) = tail.split_once(end).expect("source end delimiter");
    section
}
