const PRODUCTION_SOURCE: &str = include_str!("production_mining_session.rs");
const RUNTIME_SOURCE: &str = include_str!("runtime_snapshot.rs");

#[test]
fn production_block_effect_mutates_only_the_runtime_snapshot_owner() {
    // Arrange
    let effect = "ProductionSessionEffect::RecordBlockFound";
    let owner_call = "crate::runtime_snapshot::record_found_block()";

    // Act
    let effect_count = PRODUCTION_SOURCE.matches(effect).count();
    let owner_call_count = PRODUCTION_SOURCE.matches(owner_call).count();

    // Assert
    assert_eq!(effect_count, 1);
    assert_eq!(owner_call_count, 1);
    assert!(!PRODUCTION_SOURCE.contains("show_new_block ="));
    assert!(!PRODUCTION_SOURCE.contains("block_found ="));
}

#[test]
fn found_block_and_dismissal_share_one_command_visible_state() {
    // Arrange
    let found_function = "pub fn record_found_block()";
    let dismiss_function = "pub fn apply_block_found_dismiss_command";

    // Act
    let found_at = RUNTIME_SOURCE
        .find(found_function)
        .expect("found-block owner should exist");
    let dismiss_at = RUNTIME_SOURCE
        .find(dismiss_function)
        .expect("dismiss owner should exist");
    let source = &RUNTIME_SOURCE[dismiss_at.min(found_at)..];

    // Assert
    assert!(source.contains("mutate_command_visible_state"));
    assert!(source.contains("state.block_found.record_found_block()"));
    assert!(source.contains("apply_block_found_dismiss_effect(effect)"));
}
