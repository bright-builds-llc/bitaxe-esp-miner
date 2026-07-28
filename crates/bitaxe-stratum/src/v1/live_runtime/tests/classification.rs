use crate::v1::messages::PoolDifficulty;
use crate::v1::state::ShareDifficulty;

use super::*;

#[test]
fn accepted_share_uses_zero_difficulty_when_pool_value_is_absent() {
    // Arrange
    let mut runtime = runtime();

    // Act
    runtime.record_submit_classification(SubmitClassification::Accepted);

    // Assert
    assert_eq!(runtime.state().counters.accepted, 1);
    assert_eq!(
        runtime.state().counters.maybe_best_difficulty,
        Some(ShareDifficulty::new(0.0))
    );
}

#[test]
fn accepted_share_uses_current_pool_difficulty() {
    // Arrange
    let mut runtime = runtime();
    runtime
        .maybe_apply_server_message(StratumV1ServerMessage::SetDifficulty(PoolDifficulty {
            difficulty: 64.0,
        }))
        .expect("difficulty should apply");

    // Act
    runtime.record_submit_classification(SubmitClassification::Accepted);

    // Assert
    assert_eq!(
        runtime.state().counters.maybe_best_difficulty,
        Some(ShareDifficulty::new(64.0))
    );
}

#[test]
fn rejected_share_records_only_redacted_reason() {
    // Arrange
    let cases = [
        (
            RedactedSubmitRejectReason::PoolRejectedShare,
            "pool_rejected_share",
        ),
        (
            RedactedSubmitRejectReason::Unknown,
            "unknown_rejected_share",
        ),
    ];

    // Act / Assert
    for (reason, expected) in cases {
        let mut runtime = runtime();
        runtime.record_submit_classification(SubmitClassification::Rejected { reason });
        assert_eq!(runtime.state().counters.rejected, 1);
        assert_eq!(runtime.state().counters.rejected_reasons, [expected]);
    }
}

#[test]
fn non_share_classifications_do_not_change_counters() {
    // Arrange
    let classifications = [
        SubmitClassification::Timeout,
        SubmitClassification::Reconnect,
        SubmitClassification::Malformed,
        SubmitClassification::NoObservedShare,
        SubmitClassification::Blocked {
            reason: "test_block",
        },
        SubmitClassification::Stopped,
    ];

    // Act / Assert
    for classification in classifications {
        let mut runtime = runtime();
        let before = runtime.state().counters.clone();
        runtime.record_submit_classification(classification);
        assert_eq!(runtime.state().counters, before);
    }
}
