const ADAPTER_SOURCE: &str = include_str!("scoreboard_adapter.rs");
const PRODUCTION_SOURCE: &str = include_str!("production_mining_session.rs");
const PRODUCTION_SCOREBOARD_SOURCE: &str = include_str!("production_mining_session/scoreboard.rs");
const RUNTIME_SOURCE: &str = include_str!("runtime_snapshot.rs");
const STARTUP_SOURCE: &str = include_str!("startup.rs");

#[test]
fn startup_loads_scoreboard_once_before_production_owner() {
    // Arrange
    let initialize = "scoreboard_adapter::initialize()";
    let production = "production_mining_session::start()";

    // Act
    let initialize_count = STARTUP_SOURCE.matches(initialize).count();
    let initialize_at = STARTUP_SOURCE.find(initialize).expect("initialize call");
    let production_at = STARTUP_SOURCE.find(production).expect("production start");

    // Assert
    assert_eq!(initialize_count, 1);
    assert!(initialize_at < production_at);
}

#[test]
fn indexed_persistence_uses_exact_twenty_upstream_keys() {
    // Arrange
    let key_shape = "format!(\"scoreboard_{:02}\", index + 1)";

    // Act
    let key_shape_count = ADAPTER_SOURCE.matches(key_shape).count();

    // Assert
    assert_eq!(key_shape_count, 1);
    assert!(ADAPTER_SOURCE.contains("MAX_SCOREBOARD_ENTRIES"));
    assert!(ADAPTER_SOURCE.contains("for index in 0..MAX_SCOREBOARD_ENTRIES"));
}

#[test]
fn persistence_commits_and_reloads_before_publication() {
    // Arrange
    let record_start = ADAPTER_SOURCE
        .find("pub fn record_candidate")
        .expect("record function");
    let source = &ADAPTER_SOURCE[record_start..];
    let persistence_start = ADAPTER_SOURCE
        .find("fn persist_and_confirm")
        .expect("persistence function");
    let persistence_source = &ADAPTER_SOURCE[persistence_start..];

    // Act
    let persist_call = source.find(".record_with(entry, persist_and_confirm)");
    let commit = persistence_source.find("commit(&nvs)?");
    let reload = persistence_source.find("let confirmed = load_scoreboard()?");
    let comparison = persistence_source.find("if confirmed != expected");

    // Assert
    assert!(persist_call.is_some());
    assert!(commit < reload);
    assert!(reload < comparison);
}

#[test]
fn api_projection_only_clones_the_scoreboard_owner() {
    // Arrange
    let function = "pub fn projected_scoreboard(_timestamp_ms: u64)";
    let start = RUNTIME_SOURCE.find(function).expect("scoreboard projection");
    let end = RUNTIME_SOURCE[start..]
        .find("/// Returns projection-backed `/api/ws/live`")
        .map(|offset| start + offset)
        .expect("projection boundary");
    let source = &RUNTIME_SOURCE[start..end];

    // Act
    let owner_read = source.matches("scoreboard_adapter::entries()").count();

    // Assert
    assert_eq!(owner_read, 1);
    assert!(!source.contains("record_candidate"));
    assert!(!source.contains("collect_projected_api_views"));
}

#[test]
fn production_effect_is_redacted_and_failure_is_category_only() {
    // Arrange
    let effect = "ProductionSessionEffect::RecordScoreboard { candidate }";

    // Act
    let effect_count = PRODUCTION_SOURCE.matches(effect).count();

    // Assert
    assert_eq!(effect_count, 1);
    assert!(PRODUCTION_SCOREBOARD_SOURCE.contains("scoreboard_adapter::record_candidate(candidate)"));
    assert!(PRODUCTION_SCOREBOARD_SOURCE.contains("scoreboard=record_failed category={}"));
    assert!(!PRODUCTION_SCOREBOARD_SOURCE.contains("scoreboard=record_failed error="));
}
