const PRODUCTION_SOURCE: &str = include_str!("production_mining_session.rs");
const RUNTIME_SOURCE: &str = include_str!("runtime_snapshot.rs");

fn function_source<'a>(source: &'a str, signature: &str, next_signature: &str) -> &'a str {
    let start = source.find(signature).expect("function must exist");
    let end = source[start..]
        .find(next_signature)
        .map_or(source.len(), |offset| start + offset);
    &source[start..end]
}

#[test]
fn command_intent_has_a_distinct_boot_lifetime_owner() {
    // Arrange
    let command = function_source(
        RUNTIME_SOURCE,
        "pub fn apply_mining_operator_intent_command",
        "pub fn apply_identify_mode_command",
    );
    let publication = function_source(
        RUNTIME_SOURCE,
        "pub fn publish_production_session_snapshot",
        "pub fn publish_hashrate_snapshot",
    );

    // Act
    let requested_owner_count = RUNTIME_SOURCE
        .matches("requested_operator_intent: RequestedOperatorIntent,\n")
        .count();

    // Assert
    assert_eq!(requested_owner_count, 1);
    assert!(command.contains("state.requested_operator_intent.apply(effect)"));
    assert!(publication.contains("state.mining = snapshot.mining"));
    assert!(!publication.contains("requested_operator_intent"));
}

#[test]
fn authoritative_readiness_reads_requested_intent_not_the_projection() {
    // Arrange
    let readiness = function_source(
        PRODUCTION_SOURCE,
        "fn read_authoritative_readiness",
        "fn maybe_execute",
    );

    // Act
    let requested_reads = readiness
        .matches("crate::runtime_snapshot::requested_mining_operator_intent()")
        .count();

    // Assert
    assert_eq!(requested_reads, 1);
    assert!(!readiness.contains("mining.operator_intent"));
}
