const SETTINGS_ADAPTER_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/settings_adapter.rs");
const HTTP_API_SOURCE: &str = include_str!("../../../firmware/bitaxe/src/http_api.rs");
const RUNTIME_SNAPSHOT_SOURCE: &str =
    include_str!("../../../firmware/bitaxe/src/runtime_snapshot.rs");

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
    assert!(transaction.contains("*current = candidate.into_snapshot()"));
    assert!(!transaction.contains("refresh_current_settings_snapshot_best_effort"));
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
    assert!(!handler.contains("plan_settings_patch_body(&body)"));
    assert!(!handler.contains("from_accepted_patch"));
    assert!(!HTTP_API_SOURCE.contains("apply_persisted_settings_writes"));
    assert!(!SETTINGS_ADAPTER_SOURCE.contains("apply_persisted_settings_writes"));
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
    let response = handler
        .find("send_settings_response(request, success.public_response())")
        .expect("public success response");
    let effects = handler
        .find("schedule_settings_effects(effects)")
        .expect("post-response effect scheduling");

    // Act / Assert
    assert!(execute < response && response < effects);
    assert!(RUNTIME_SNAPSHOT_SOURCE.contains("settings_adapter::current_settings_snapshot()"));
    assert!(RUNTIME_SNAPSHOT_SOURCE.contains("snapshot.platform.hostname = hostname.clone()"));
    assert!(!RUNTIME_SNAPSHOT_SOURCE.contains("apply_persisted_settings_writes"));
}

fn source_between<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    let (_, tail) = source.split_once(start).expect("source start delimiter");
    let (section, _) = tail.split_once(end).expect("source end delimiter");
    section
}
