//! Submit-response classification for live Stratum v1 share outcomes.
//!
//! Reference breadcrumbs:
//! - `reference/esp-miner/main/system.c`
//! - `reference/esp-miner/components/stratum/stratum_api.c`
//! - `docs/parity/evidence/phase-24-bm1366-production-work-path/result-correlation.md`
//! - Parity checklist rows `STR-009` and `STR-011`

use std::fmt;

use crate::jsonrpc::StratumRequestId;
use crate::v1::messages::StratumResponse;
use crate::v1::production_work::{PoolSessionGeneration, SubmitIntent};

#[derive(Clone, PartialEq)]
pub enum SubmitResponseObservation {
    Response(StratumResponse),
    FakePoolOnlyResponse(StratumResponse),
    StaleGeneration {
        observed_generation: PoolSessionGeneration,
        response: StratumResponse,
    },
    Timeout,
    Reconnect,
    Malformed,
    Blocked { reason: &'static str },
    SocketStopped,
}

impl fmt::Debug for SubmitResponseObservation {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Response(_)
            | Self::FakePoolOnlyResponse(_)
            | Self::StaleGeneration { .. } => formatter
                .debug_struct("SubmitResponseObservation")
                .field("pool_response", &"redacted")
                .finish(),
            Self::Timeout => formatter.write_str("SubmitResponseObservation::Timeout"),
            Self::Reconnect => formatter.write_str("SubmitResponseObservation::Reconnect"),
            Self::Malformed => formatter.write_str("SubmitResponseObservation::Malformed"),
            Self::Blocked { reason } => formatter
                .debug_struct("SubmitResponseObservation::Blocked")
                .field("reason", reason)
                .finish(),
            Self::SocketStopped => formatter.write_str("SubmitResponseObservation::SocketStopped"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RedactedSubmitRejectReason {
    PoolRejectedShare,
    Unknown,
}

impl RedactedSubmitRejectReason {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::PoolRejectedShare => "pool_rejected_share",
            Self::Unknown => "unknown_rejected_share",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SubmitClassification {
    Accepted,
    Rejected {
        reason: RedactedSubmitRejectReason,
    },
    Timeout,
    Reconnect,
    Malformed,
    NoObservedShare,
    Blocked {
        reason: &'static str,
    },
    Stopped,
}

impl SubmitClassification {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Rejected { .. } => "rejected",
            Self::Timeout => "timeout",
            Self::Reconnect => "reconnect",
            Self::Malformed => "malformed",
            Self::NoObservedShare => "no_observed_share",
            Self::Blocked { .. } => "blocked",
            Self::Stopped => "stopped",
        }
    }
}

#[must_use]
pub fn classify_submit_response(
    _intent: &SubmitIntent,
    _request_id: StratumRequestId,
    _observation: SubmitResponseObservation,
) -> SubmitClassification {
    SubmitClassification::NoObservedShare
}

#[must_use]
pub fn classify_maybe_submit_response(
    maybe_intent: Option<&SubmitIntent>,
    request_id: StratumRequestId,
    observation: SubmitResponseObservation,
) -> SubmitClassification {
    let Some(intent) = maybe_intent else {
        return SubmitClassification::Blocked {
            reason: "submit_intent_missing",
        };
    };

    classify_submit_response(intent, request_id, observation)
}

#[cfg(test)]
mod tests {
    use bitaxe_asic::bm1366::{
        result::Bm1366NonceResult,
        work::Bm1366JobId,
    };

    use super::*;
    use crate::v1::messages::{ExtranonceAssignment, MiningNotify, PoolDifficulty, StratumResponse, StratumResponseError};
    use crate::v1::mining::MiningWorkBuilder;
    use crate::v1::production_work::{
        CorrelationOutcome, ProductionNonceObservation, ProductionWorkRegistry,
    };

    #[test]
    fn submit_response_classifier_accepts_matching_live_submit_intent() {
        // Arrange
        let intent = submit_intent();
        let request_id = StratumRequestId::new(7);

        // Act
        let classification = classify_submit_response(
            &intent,
            request_id,
            SubmitResponseObservation::Response(success_response(7)),
        );

        // Assert
        assert_eq!(classification, SubmitClassification::Accepted);
    }

    #[test]
    fn submit_response_classifier_rejects_with_redacted_reason_label() {
        // Arrange
        let intent = submit_intent();
        let request_id = StratumRequestId::new(7);

        // Act
        let classification = classify_submit_response(
            &intent,
            request_id,
            SubmitResponseObservation::Response(rejected_response(7, "raw pool reject text")),
        );
        let rendered_observation = format!(
            "{:?}",
            SubmitResponseObservation::Response(rejected_response(7, "raw pool reject text"))
        );

        // Assert
        assert_eq!(
            classification,
            SubmitClassification::Rejected {
                reason: RedactedSubmitRejectReason::PoolRejectedShare
            }
        );
        assert_eq!(RedactedSubmitRejectReason::PoolRejectedShare.as_str(), "pool_rejected_share");
        assert!(!rendered_observation.contains("raw pool reject text"));
    }

    #[test]
    fn submit_response_classifier_never_accepts_mismatched_absent_fake_or_stale_inputs() {
        // Arrange
        let intent = submit_intent();
        let request_id = StratumRequestId::new(7);

        // Act
        let mismatched = classify_submit_response(
            &intent,
            request_id,
            SubmitResponseObservation::Response(success_response(8)),
        );
        let absent_id = classify_submit_response(
            &intent,
            request_id,
            SubmitResponseObservation::Response(response_without_id(true)),
        );
        let absent_intent = classify_maybe_submit_response(
            None,
            request_id,
            SubmitResponseObservation::Response(success_response(7)),
        );
        let fake_only = classify_submit_response(
            &intent,
            request_id,
            SubmitResponseObservation::FakePoolOnlyResponse(success_response(7)),
        );
        let stale_generation = classify_submit_response(
            &intent,
            request_id,
            SubmitResponseObservation::StaleGeneration {
                observed_generation: intent.generation.next(),
                response: success_response(7),
            },
        );

        // Assert
        assert_eq!(mismatched, SubmitClassification::NoObservedShare);
        assert_eq!(absent_id, SubmitClassification::NoObservedShare);
        assert_eq!(
            absent_intent,
            SubmitClassification::Blocked {
                reason: "submit_intent_missing"
            }
        );
        assert_eq!(fake_only, SubmitClassification::NoObservedShare);
        assert_eq!(
            stale_generation,
            SubmitClassification::Blocked {
                reason: "stale_generation"
            }
        );
    }

    #[test]
    fn submit_response_classifier_distinguishes_non_response_observations() {
        // Arrange
        let intent = submit_intent();
        let request_id = StratumRequestId::new(7);

        // Act
        let timeout = classify_submit_response(&intent, request_id, SubmitResponseObservation::Timeout);
        let reconnect =
            classify_submit_response(&intent, request_id, SubmitResponseObservation::Reconnect);
        let malformed =
            classify_submit_response(&intent, request_id, SubmitResponseObservation::Malformed);
        let blocked = classify_submit_response(
            &intent,
            request_id,
            SubmitResponseObservation::Blocked {
                reason: "precondition_blocked",
            },
        );
        let stopped =
            classify_submit_response(&intent, request_id, SubmitResponseObservation::SocketStopped);

        // Assert
        assert_eq!(timeout, SubmitClassification::Timeout);
        assert_eq!(reconnect, SubmitClassification::Reconnect);
        assert_eq!(malformed, SubmitClassification::Malformed);
        assert_eq!(
            blocked,
            SubmitClassification::Blocked {
                reason: "precondition_blocked"
            }
        );
        assert_eq!(stopped, SubmitClassification::Stopped);
    }

    fn submit_intent() -> SubmitIntent {
        let mut registry = ProductionWorkRegistry::new();
        let job_id = Bm1366JobId::new(0x28);
        registry
            .enqueue_pool_work(sample_work(job_id))
            .expect("sample work should enqueue");
        let _dispatch = registry.dispatch_next().expect("work should dispatch");
        let outcome = registry.correlate_nonce_result(ProductionNonceObservation {
            observed_generation: registry.generation(),
            result: Bm1366NonceResult {
                job_id,
                nonce: 0x1234_5678,
                asic_index: 0,
                core_id: 1,
                small_core_id: 0,
                version_bits: 0x0000_2000,
            },
        });
        let CorrelationOutcome::SubmitIntent(intent) = outcome else {
            panic!("sample active work should produce submit intent");
        };
        intent
    }

    fn sample_work(job_id: Bm1366JobId) -> crate::v1::mining::MiningWork {
        MiningWorkBuilder::new(
            MiningNotify {
                job_id: "correlated-job".to_owned(),
                prev_block_hash: "00".repeat(32),
                coinbase_1: "0200000001".to_owned(),
                coinbase_2: "ffffffff".to_owned(),
                merkle_branches: Vec::new(),
                version: 0x2000_0004,
                nbits: 0x1705_ae3a,
                ntime: 0x6470_25b5,
                clean_jobs: false,
            },
            ExtranonceAssignment {
                extranonce1: "4de05269".to_owned(),
                extranonce2_len: 4,
            },
        )
        .with_pool_difficulty(PoolDifficulty { difficulty: 1.25 })
        .build(job_id)
        .expect("sample work should build")
    }

    fn success_response(id: u64) -> StratumResponse {
        StratumResponse {
            maybe_id: Some(StratumRequestId::new(id)),
            success: true,
            maybe_error: None,
            maybe_extranonce: None,
            maybe_version_mask: None,
        }
    }

    fn rejected_response(id: u64, reason: &str) -> StratumResponse {
        StratumResponse {
            maybe_id: Some(StratumRequestId::new(id)),
            success: false,
            maybe_error: Some(StratumResponseError {
                maybe_code: Some(21),
                message: reason.to_owned(),
            }),
            maybe_extranonce: None,
            maybe_version_mask: None,
        }
    }

    fn response_without_id(success: bool) -> StratumResponse {
        StratumResponse {
            maybe_id: None,
            success,
            maybe_error: None,
            maybe_extranonce: None,
            maybe_version_mask: None,
        }
    }
}
