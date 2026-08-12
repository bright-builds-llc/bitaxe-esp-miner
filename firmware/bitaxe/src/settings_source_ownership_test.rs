const SETTINGS_HANDLER_SOURCE: &str = include_str!("http_api/settings.rs");
const SETTINGS_ADAPTER_SOURCE: &str = include_str!("settings_adapter.rs");
const STARTUP_SOURCE: &str = include_str!("startup.rs");

#[test]
fn validated_patch_owns_persistence_and_unknown_only_requests_remain_inert() {
    // Arrange
    let persistence_path = SETTINGS_HANDLER_SOURCE
        .split("let accepted =")
        .nth(1)
        .expect("settings handler must contain the validated persistence path");

    // Act / Assert
    assert!(persistence_path.contains("plan_settings_patch_body(&body)"));
    assert!(persistence_path.contains("SettingsPersistencePlan::from_accepted(&accepted)"));
    assert!(persistence_path.contains("if plan.is_empty()"));
    assert!(persistence_path.contains("execute_settings_persistence_plan(&plan, &mut adapter)"));
    assert!(!persistence_path.contains("production_mining_session::notify"));
}

#[test]
fn firmware_adapter_writes_and_reconciles_every_supported_nvs_type() {
    // Arrange
    let required_variants = ["String", "U16", "I32", "U64"];
    let write_path = SETTINGS_ADAPTER_SOURCE
        .split("fn write_nvs(")
        .nth(1)
        .and_then(|source| source.split("fn nvs_write_matches(").next())
        .expect("settings adapter must contain the typed write boundary");
    let reconcile_path = SETTINGS_ADAPTER_SOURCE
        .split("fn nvs_write_matches(")
        .nth(1)
        .expect("settings adapter must contain the typed reconciliation boundary");

    // Act / Assert
    for variant in required_variants {
        let pattern = format!("NvsWrite::{variant}");
        assert!(write_path.contains(&pattern));
        assert!(reconcile_path.contains(&pattern));
    }
}

#[test]
fn public_snapshot_excludes_pool_and_credential_keys() {
    // Arrange
    let strict_reload = SETTINGS_ADAPTER_SOURCE
        .split("fn read_current_settings_snapshot_strict(")
        .nth(1)
        .and_then(|source| source.split("fn maybe_read_protocol(").next())
        .expect("settings adapter must contain strict public snapshot reload");

    // Act / Assert
    assert!(strict_reload.contains("filter(|schema| !is_pool_configuration_key"));
    assert!(SETTINGS_ADAPTER_SOURCE.contains("key.starts_with(\"stratum\")"));
    assert!(SETTINGS_ADAPTER_SOURCE.contains("key.starts_with(\"fbstratum\")"));
}

#[test]
fn defaults_attestation_strictly_reads_private_settings_then_retains_only_closed_facts() {
    // Arrange
    let attestation_path = SETTINGS_ADAPTER_SOURCE
        .split("pub fn current_ultra205_defaults_attestation(")
        .nth(1)
        .and_then(|source| source.split("/// Returns the project-owned").next())
        .expect("settings adapter must contain the defaults attestation boundary");

    // Act / Assert
    assert!(attestation_path.contains("EspNvs::new(partition, NVS_NAMESPACE, false)"));
    assert!(attestation_path.contains("read_all_settings_snapshot_strict(&nvs)"));
    assert!(attestation_path.contains("Ultra205DefaultsAttestation::from_snapshot"));
    assert!(STARTUP_SOURCE.contains("current_ultra205_defaults_attestation()"));
    assert!(STARTUP_SOURCE.contains("retained_marker(!settings_adapter::start_mining_on_boot())"));
    assert!(!SETTINGS_ADAPTER_SOURCE.contains("public-pool.io"));
    assert!(!STARTUP_SOURCE.contains("public-pool.io"));
}
