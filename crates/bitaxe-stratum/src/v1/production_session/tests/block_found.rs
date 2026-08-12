use super::*;

#[test]
fn network_qualifying_nonce_emits_block_effect_before_submit_and_scoreboard() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active_with_nbits(&mut adapter, "207fffff");
    adapter.drive(wake(ready(), 4));
    let observation = dispatched_observation(&adapter);
    let effects_before = adapter.effects.len();

    // Act
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation,
        now_ms: 5,
    });

    // Assert
    let effects = &adapter.effects[effects_before..];
    let block_at = effects
        .iter()
        .position(|effect| matches!(effect, ProductionSessionEffect::RecordBlockFound))
        .expect("network-qualified nonce should record a block");
    let submit_at = effects
        .iter()
        .position(|effect| matches!(effect, ProductionSessionEffect::WritePoolLine { .. }))
        .expect("pool-qualified nonce should submit");
    let scoreboard_at = effects
        .iter()
        .position(|effect| matches!(effect, ProductionSessionEffect::RecordScoreboard { .. }))
        .expect("valid nonce should reach the scoreboard");
    assert!(block_at < submit_at);
    assert!(submit_at < scoreboard_at);
    assert_eq!(
        format!("{:?}", effects[block_at]),
        "ProductionSessionEffect::RecordBlockFound"
    );
}

#[test]
fn nonce_below_network_target_emits_no_block_effect() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
    adapter.drive(wake(ready(), 4));
    let observation = dispatched_observation(&adapter);
    let effects_before = adapter.effects.len();

    // Act
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation,
        now_ms: 5,
    });

    // Assert
    assert!(!adapter.effects[effects_before..]
        .iter()
        .any(|effect| matches!(effect, ProductionSessionEffect::RecordBlockFound)));
}

#[test]
fn duplicate_network_qualifying_nonce_matches_upstream_per_result_counting() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active_with_nbits(&mut adapter, "207fffff");
    adapter.drive(wake(ready(), 4));
    let observation = dispatched_observation(&adapter);

    // Act
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation,
        now_ms: 5,
    });
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation,
        now_ms: 6,
    });

    // Assert
    let block_count = adapter
        .effects
        .iter()
        .filter(|effect| matches!(effect, ProductionSessionEffect::RecordBlockFound))
        .count();
    let submit_count = adapter
        .writes
        .iter()
        .filter(|(_, line)| line.contains("mining.submit"))
        .count();
    assert_eq!(block_count, 2);
    assert_eq!(submit_count, 1);
}

#[test]
fn stale_network_qualifying_nonce_emits_no_block_effect() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active_with_nbits(&mut adapter, "207fffff");
    adapter.drive(wake(ready(), 4));
    let mut observation = dispatched_observation(&adapter);
    observation.observed_generation = observation.observed_generation.next();
    let effects_before = adapter.effects.len();

    // Act
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation,
        now_ms: 5,
    });

    // Assert
    assert!(!adapter.effects[effects_before..]
        .iter()
        .any(|effect| matches!(effect, ProductionSessionEffect::RecordBlockFound)));
}

#[test]
fn malformed_network_target_emits_no_block_effect() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active_with_nbits(&mut adapter, "17000000");
    adapter.drive(wake(ready(), 4));
    let observation = dispatched_observation(&adapter);
    let effects_before = adapter.effects.len();

    // Act
    adapter.drive(ProductionSessionEvent::AsicResult {
        observation,
        now_ms: 5,
    });

    // Assert
    assert!(!adapter.effects[effects_before..]
        .iter()
        .any(|effect| matches!(effect, ProductionSessionEffect::RecordBlockFound)));
    assert_eq!(
        adapter.session.snapshot().mining.maybe_blocked_reason,
        Some("production_target_mismatch")
    );
}
