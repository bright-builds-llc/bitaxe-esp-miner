use serde_json::json;

use super::*;

struct FixedSource {
    maybe_log: Option<AuthorizationRejectionLog>,
    maybe_fingerprint: Option<StateFingerprint>,
}

impl AuthorizationRejectionSource for FixedSource {
    fn rejection_log(&self) -> Option<&AuthorizationRejectionLog> {
        self.maybe_log.as_ref()
    }

    fn high_water_fingerprint(&self) -> Result<Option<StateFingerprint>, LeaseAuthorizationError> {
        Ok(self.maybe_fingerprint)
    }
}

fn source(log: AuthorizationRejectionLog) -> FixedSource {
    FixedSource {
        maybe_log: Some(log),
        maybe_fingerprint: Some(StateFingerprint::from_digest([0xab; 32])),
    }
}

#[test]
fn empty_log_reviews_as_zero_rejections_with_a_null_last_record() {
    // Arrange
    let source = source(AuthorizationRejectionLog::default());

    // Act
    let review = authorization_rejection_review(&source)
        .expect("review")
        .expect("log present");

    // Assert
    assert_eq!(
        serde_json::to_value(review).expect("review JSON"),
        json!({
            "schema": "worker-authorization-rejection-review-v1",
            "bootRejections": 0,
            "last": null,
            "highWater": {"advancedThisBoot": false, "fingerprintSha256": "ab".repeat(32)},
        })
    );
}

#[test]
fn review_reports_the_latest_record_with_its_boot_ordinal() {
    // Arrange
    let mut log = AuthorizationRejectionLog::default();
    log.record(
        AuthorizationOperation::Start,
        SignatureAttribution::NotEvaluated,
        ContextAttribution::Expired,
        ReplayGuardAttribution::NotEvaluated,
    );
    log.record(
        AuthorizationOperation::Renew,
        SignatureAttribution::Valid,
        ContextAttribution::Current,
        ReplayGuardAttribution::AtOrBelowDurableHighWater,
    );
    log.mark_high_water_advanced();

    // Act
    let review = authorization_rejection_review(&source(log))
        .expect("review")
        .expect("log present");

    // Assert
    assert_eq!(
        serde_json::to_value(review).expect("review JSON"),
        json!({
            "schema": "worker-authorization-rejection-review-v1",
            "bootRejections": 2,
            "last": {"ordinal": 2, "operation": "renew", "signature": "valid",
                "context": "current", "replayGuard": "at_or_below_durable_high_water"},
            "highWater": {"advancedThisBoot": true, "fingerprintSha256": "ab".repeat(32)},
        })
    );
}

#[test]
fn every_attribution_category_serializes_to_its_closed_wire_label() {
    // Arrange
    let values = (
        [
            SignatureAttribution::Invalid,
            SignatureAttribution::NotEvaluated,
        ],
        [ContextAttribution::Mismatch, ContextAttribution::Absent],
        [
            ReplayGuardAttribution::Fresh,
            ReplayGuardAttribution::Unavailable,
        ],
    );

    // Act
    let labels = serde_json::to_value(values).expect("labels");

    // Assert
    assert_eq!(
        labels,
        json!([
            ["invalid", "not_evaluated"],
            ["mismatch", "absent"],
            ["fresh", "unavailable"]
        ])
    );
}

#[test]
fn a_source_without_a_log_is_unsupported() {
    // Arrange
    let source = FixedSource {
        maybe_log: None,
        maybe_fingerprint: Some(StateFingerprint::from_digest([1; 32])),
    };

    // Act
    let review = authorization_rejection_review(&source).expect("review");

    // Assert
    assert!(review.is_none());
}

#[test]
fn a_missing_high_water_fingerprint_fails_as_persistence() {
    // Arrange
    let source = FixedSource {
        maybe_log: Some(AuthorizationRejectionLog::default()),
        maybe_fingerprint: None,
    };

    // Act
    let review = authorization_rejection_review(&source);

    // Assert
    assert_eq!(
        review.expect_err("fingerprint required"),
        LeaseAuthorizationError::Persistence
    );
}
