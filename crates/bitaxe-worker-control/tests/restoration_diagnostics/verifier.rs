//! Verifier-level attribution and the no-write guarantee for every rejection kind.
use bitaxe_worker_control::{
    AuthorizationOperation, ContextAttribution, LeaseAuthorizationVerifier, ReplayGuardAttribution,
    SignatureAttribution, WorkLeaseAuthorizationVerifier, WorkerLeaseAuthorizationContext,
    WorkerLeaseGrant, WorkerLeaseRenewal,
};

use super::support::*;

const CURRENT: &str = "CCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCCC";
const OTHER: &str = "DDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDDD";

fn context(binding: &str) -> WorkerLeaseAuthorizationContext {
    WorkerLeaseAuthorizationContext::parse(binding).expect("context")
}

fn grant(binding: &str, sequence: u64) -> WorkerLeaseGrant {
    serde_json::from_value(start_payload(binding, sequence)).expect("grant")
}

fn renewal(binding: &str, sequence: u64) -> WorkerLeaseRenewal {
    serde_json::from_value(renew_payload(binding, sequence)).expect("renewal")
}

fn verifier(store: &PersistedStore) -> WorkLeaseAuthorizationVerifier<PersistedStore> {
    WorkLeaseAuthorizationVerifier::new(trust(), store.clone())
}

fn attribution(
    verifier: &WorkLeaseAuthorizationVerifier<PersistedStore>,
) -> (
    SignatureAttribution,
    ContextAttribution,
    ReplayGuardAttribution,
) {
    let record = verifier
        .authorization_rejections()
        .and_then(|log| log.maybe_last())
        .expect("recorded rejection");
    (record.signature, record.context, record.replay_guard)
}

fn tampered(binding: &str, sequence: u64) -> WorkerLeaseGrant {
    let mut payload = start_payload(binding, sequence);
    let compact = payload["authorization"]
        .as_str()
        .expect("compact")
        .to_owned();
    let flipped = if compact.ends_with('A') { "B" } else { "A" };
    payload["authorization"] = format!("{}{flipped}", &compact[..compact.len() - 1]).into();
    serde_json::from_value(payload).expect("grant")
}

#[test]
fn a_valid_signature_for_another_context_is_a_fresh_mismatch() {
    // Arrange
    let store = PersistedStore::default();
    let mut verifier = verifier(&store);

    // Act
    let result = verifier.verify_start(&grant(OTHER, 1), &context(CURRENT));

    // Assert
    assert_eq!(
        result.expect_err("mismatch").category(),
        "invalid_authorization"
    );
    assert_eq!(
        attribution(&verifier),
        (
            SignatureAttribution::Valid,
            ContextAttribution::Mismatch,
            ReplayGuardAttribution::Fresh
        )
    );
}

#[test]
fn a_changed_signature_is_attributed_as_invalid_without_a_replay_lookup() {
    // Arrange
    let store = PersistedStore::default();
    let mut verifier = verifier(&store);

    // Act
    let _ = verifier.verify_start(&tampered(CURRENT, 1), &context(CURRENT));

    // Assert
    assert_eq!(
        attribution(&verifier),
        (
            SignatureAttribution::Invalid,
            ContextAttribution::Current,
            ReplayGuardAttribution::NotEvaluated
        )
    );
    assert_eq!(store.counts().loads, 0);
}

#[test]
fn a_malformed_authorization_is_not_evaluated() {
    // Arrange
    let store = PersistedStore::default();
    let mut verifier = verifier(&store);
    let mut payload = start_payload(CURRENT, 1);
    payload["authorization"] = "not.a.jws".into();
    let malformed: WorkerLeaseGrant = serde_json::from_value(payload).expect("grant");

    // Act
    let _ = verifier.verify_start(&malformed, &context(CURRENT));

    // Assert
    assert_eq!(
        attribution(&verifier),
        (
            SignatureAttribution::NotEvaluated,
            ContextAttribution::Mismatch,
            ReplayGuardAttribution::NotEvaluated
        )
    );
}

#[test]
fn an_in_context_replay_is_at_or_below_the_durable_high_water() {
    // Arrange
    let store = PersistedStore::default();
    let mut verifier = verifier(&store);
    verifier
        .verify_start(&grant(CURRENT, 3), &context(CURRENT))
        .expect("first Start");

    // Act
    let result = verifier.verify_start(&grant(CURRENT, 3), &context(CURRENT));

    // Assert
    assert_eq!(result.expect_err("replay").category(), "replay");
    assert_eq!(
        attribution(&verifier),
        (
            SignatureAttribution::Valid,
            ContextAttribution::Current,
            ReplayGuardAttribution::AtOrBelowDurableHighWater
        )
    );
}

#[test]
fn acceptance_advances_the_high_water_without_a_rejection_record() {
    // Arrange
    let store = PersistedStore::default();
    let mut verifier = verifier(&store);

    // Act
    verifier
        .verify_start(&grant(CURRENT, 1), &context(CURRENT))
        .expect("Start");

    // Assert
    let log = verifier.authorization_rejections().expect("log");
    assert!(log.high_water_advanced_this_boot());
    assert_eq!((log.boot_rejections(), log.maybe_last()), (0, None));
}

#[test]
fn no_rejection_kind_writes_durable_state() {
    // Arrange
    let store = PersistedStore::default();
    let mut verifier = verifier(&store);
    verifier
        .verify_start(&grant(CURRENT, 3), &context(CURRENT))
        .expect("Start(3)");
    verifier
        .verify_renewal(&renewal(CURRENT, 4), CHALLENGE_ID, &context(CURRENT))
        .expect("Renew(4)");
    let before = store.counts();

    // Act
    let results = [
        verifier.verify_start(&grant(OTHER, 9), &context(CURRENT)),
        verifier.verify_start(&grant(OTHER, 2), &context(CURRENT)),
        verifier.verify_start(&tampered(CURRENT, 9), &context(CURRENT)),
        verifier.verify_start(&grant(CURRENT, 3), &context(CURRENT)),
        verifier.verify_renewal(&renewal(CURRENT, 4), CHALLENGE_ID, &context(CURRENT)),
        verifier.verify_renewal(&renewal(OTHER, 9), CHALLENGE_ID, &context(CURRENT)),
    ];
    verifier.record_context_rejection(AuthorizationOperation::Start, ContextAttribution::Expired);

    // Assert
    assert!(results.iter().all(Result::is_err));
    let after = store.counts();
    assert_eq!(after.compare_and_store, before.compare_and_store);
    assert_eq!(after.mark_effect_pending, before.mark_effect_pending);
    assert_eq!(after.clear_effect_pending, before.clear_effect_pending);
    assert_eq!(
        verifier
            .authorization_rejections()
            .map(|log| log.boot_rejections()),
        Some(7)
    );
}
