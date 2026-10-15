//! Source ownership of the all-time best difficulty: one NVS owner, restored before
//! mining starts, recorded on the mining owner thread from every scoreboard candidate.

const ADAPTER_SOURCE: &str = include_str!("best_difficulty_adapter.rs");
const OWNER_HOOK_SOURCE: &str = include_str!("production_mining_session/best_difficulty.rs");
const PRODUCTION_SOURCE: &str = include_str!("production_mining_session.rs");
const PRESERVATION_SOURCE: &str = include_str!("settings_adapter/preservation/model.rs");
const STARTUP_SOURCE: &str = include_str!("startup.rs");

#[test]
fn startup_restores_the_best_once_before_the_production_owner() {
    // Arrange
    let initialize = "best_difficulty_adapter::initialize()";
    let production = "production_mining_session::start()";

    // Act
    let count = STARTUP_SOURCE.matches(initialize).count();
    let initialize_at = STARTUP_SOURCE.find(initialize).expect("initialize call");
    let production_at = STARTUP_SOURCE.find(production).expect("production start");

    // Assert
    assert_eq!(count, 1);
    assert!(initialize_at < production_at);
}

#[test]
fn the_adapter_owns_the_reference_u64_key_without_settings_locks() {
    // Arrange / Act
    let key_declarations = ADAPTER_SOURCE.matches("\"bestdiff\"").count();

    // Assert: upstream `NVS_CONFIG_BEST_DIFF` is a u64; settings locks would stall mining.
    assert_eq!(key_declarations, 1);
    assert!(ADAPTER_SOURCE.contains(".set_u64(BEST_DIFF_KEY, all_time)"));
    assert!(ADAPTER_SOURCE.contains(".get_u64(BEST_DIFF_KEY)"));
    assert!(ADAPTER_SOURCE.contains("reload_mismatch"));
    assert!(!ADAPTER_SOURCE.contains("SETTINGS_TRANSACTION_LOCK"));
    assert!(!ADAPTER_SOURCE.contains("MutationGuard"));
}

#[test]
fn every_scoreboard_candidate_feeds_the_best_before_the_scoreboard() {
    // Arrange
    let arm = PRODUCTION_SOURCE
        .find("ProductionSessionEffect::RecordScoreboard { candidate } =>")
        .expect("scoreboard arm");
    let source = &PRODUCTION_SOURCE[arm..];

    // Act
    let best_at = source
        .find("best_difficulty::record(candidate.difficulty());")
        .expect("best-difficulty record");
    let scoreboard_at = source.find("scoreboard::record(candidate)").expect("scoreboard");

    // Assert
    assert!(best_at < scoreboard_at);
    assert!(OWNER_HOOK_SOURCE.contains("best_difficulty_adapter::persist(all_time)"));
}

#[test]
fn the_best_stays_outside_the_worker_preservation_fingerprint() {
    // Arrange / Act / Assert: leases legitimately raise the best, as upstream mining does.
    assert!(!PRESERVATION_SOURCE.contains("bestdiff"));
}
