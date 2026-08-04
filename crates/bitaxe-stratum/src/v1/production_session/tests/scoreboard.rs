use super::*;

#[test]
fn valid_current_nonce_emits_one_redacted_scoreboard_effect() {
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
    let scoreboard_effects: Vec<_> = adapter.effects[effects_before..]
        .iter()
        .filter_map(|effect| {
            let ProductionSessionEffect::RecordScoreboard { candidate } = effect else {
                return None;
            };
            Some(candidate)
        })
        .collect();
    assert_eq!(scoreboard_effects.len(), 1);
    assert!(scoreboard_effects[0].difficulty().is_finite());
    assert!(scoreboard_effects[0].difficulty() > 0.0);
    let rendered = format!("{:?}", scoreboard_effects[0]);
    assert_eq!(
        rendered,
        "ScoreboardCandidate { redaction: \"scoreboard_candidate_redacted\" }"
    );
    assert!(!rendered.contains(&scoreboard_effects[0].submission().job_id));
    assert!(!rendered.contains(&scoreboard_effects[0].submission().extranonce2));
}

#[test]
fn duplicate_valid_nonce_still_emits_scoreboard_effect_without_second_submit() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
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
    let scoreboard_count = adapter
        .effects
        .iter()
        .filter(|effect| matches!(effect, ProductionSessionEffect::RecordScoreboard { .. }))
        .count();
    let submit_count = adapter
        .writes
        .iter()
        .filter(|(_, line)| line.contains("mining.submit"))
        .count();
    assert_eq!(scoreboard_count, 2);
    assert_eq!(submit_count, 1);
}

#[test]
fn stale_generation_nonce_emits_no_scoreboard_effect() {
    // Arrange
    let mut adapter = DeterministicProductionSessionAdapter::new(Some(pools(false)));
    establish_active(&mut adapter);
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
        .any(|effect| matches!(effect, ProductionSessionEffect::RecordScoreboard { .. })));
}
