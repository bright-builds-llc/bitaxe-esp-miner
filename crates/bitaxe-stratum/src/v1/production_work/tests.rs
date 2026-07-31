use bitaxe_asic::bm1366::{
    production::ProductionAsicBlocker,
    result::Bm1366NonceResult,
    work::{Bm1366JobId, Bm1366WorkFields},
};

use super::*;
use crate::error::StratumV1Error;
use crate::v1::messages::{ExtranonceAssignment, MiningNotify, PoolDifficulty};
use crate::v1::mining::{MiningWork, MiningWorkBuilder};

mod share_qualification;

#[test]
fn production_work_enqueue_registers_valid_job_for_current_generation() {
    // Arrange
    let mut registry = ProductionWorkRegistry::new();
    let job_id = Bm1366JobId::new(0x28);
    let work = sample_work(job_id, "pool-job-hidden", false);

    // Act
    registry
        .enqueue_pool_work(work)
        .expect("pool work should enqueue");

    // Assert
    assert_eq!(registry.generation(), PoolSessionGeneration::initial());
    assert!(registry.valid_jobs().contains(job_id));
}

#[test]
fn production_work_dispatch_preserves_pool_context_and_payload() {
    // Arrange
    let mut registry = ProductionWorkRegistry::new();
    let job_id = Bm1366JobId::new(0x30);
    registry
        .enqueue_pool_work(sample_work(job_id, "pool-job-hidden", false))
        .expect("pool work should enqueue");

    // Act
    let dispatch = registry.dispatch_next().expect("work should dispatch");
    let active = registry
        .maybe_active_work(job_id)
        .expect("dispatched work should be active");

    // Assert
    assert_eq!(dispatch.generation, PoolSessionGeneration::initial());
    assert_eq!(dispatch.work_payload.job_id(), job_id);
    assert_eq!(dispatch.work.asic_job_id, job_id);
    assert_eq!(active.generation, PoolSessionGeneration::initial());
    assert_eq!(active.asic_job_id, job_id);
    assert_eq!(active.ntime, 0x6470_25b5);
    assert_eq!(active.target_context.compact_nbits, 0x1705_ae3a);
    assert_eq!(
        active.target_context.maybe_pool_difficulty,
        Some(PoolDifficulty { difficulty: 1.25 })
    );
    assert!(active.dispatched);
    assert_eq!(active.observed_candidate_count, 0);
}

#[test]
fn production_work_generation_advances_once_per_session_invalidation() {
    // Arrange
    let mut registry = ProductionWorkRegistry::new();

    // Act
    registry.invalidate_for_reconnect();
    let after_reconnect = registry.generation();
    registry.invalidate_for_authorization_reset();
    let after_authorization = registry.generation();
    registry.invalidate_for_session_replacement();
    let after_replacement = registry.generation();

    // Assert
    assert_eq!(PoolSessionGeneration::initial().raw(), 0);
    assert_eq!(after_reconnect.raw(), 1);
    assert_eq!(after_authorization.raw(), 2);
    assert_eq!(after_replacement.raw(), 3);
}

#[test]
fn production_work_record_debug_redacts_raw_context() {
    // Arrange
    let mut registry = ProductionWorkRegistry::new();
    let job_id = Bm1366JobId::new(0x38);
    registry
        .enqueue_pool_work(sample_work(job_id, "pool-job-hidden", false))
        .expect("pool work should enqueue");
    let _dispatch = registry.dispatch_next().expect("work should dispatch");
    let record = registry
        .maybe_active_work(job_id)
        .expect("dispatched work should be active");

    // Act
    let rendered = format!("{record:?}");

    // Assert
    assert!(rendered.contains("ProductionWorkRecord"));
    assert!(rendered.contains("generation"));
    assert!(!rendered.contains("pool-job-hidden"));
    assert!(!rendered.contains("4de05269"));
    assert!(!rendered.contains("00000000"));
    assert!(!rendered.contains("1705ae3a"));
    assert!(!rendered.contains("647025b5"));
    assert!(!rendered.contains("0x38"));
}

#[test]
fn production_dispatch_debug_redacts_raw_context() {
    // Arrange
    let mut registry = ProductionWorkRegistry::new();
    registry
        .enqueue_pool_work(sample_work(
            Bm1366JobId::new(0x40),
            "pool-job-hidden",
            false,
        ))
        .expect("pool work should enqueue");
    let dispatch = registry.dispatch_next().expect("work should dispatch");

    // Act
    let rendered = format!("{dispatch:?}");

    // Assert
    assert!(rendered.contains("ProductionDispatch"));
    assert!(rendered.contains("generation"));
    assert!(!rendered.contains("pool-job-hidden"));
    assert!(!rendered.contains("4de05269"));
    assert!(!rendered.contains("00000000"));
    assert!(!rendered.contains("1705ae3a"));
    assert!(!rendered.contains("647025b5"));
    assert!(!rendered.contains("0x40"));
}

#[test]
fn production_work_clean_jobs_invalidates_queued_active_and_valid_jobs() {
    // Arrange
    let mut registry = ProductionWorkRegistry::new();
    let queued_stale_job_id = Bm1366JobId::new(0x48);
    let active_stale_job_id = Bm1366JobId::new(0x50);
    let current_job_id = Bm1366JobId::new(0x58);
    registry
        .enqueue_pool_work(sample_work(queued_stale_job_id, "queued-stale-job", false))
        .expect("queued stale work should enqueue");
    registry
        .enqueue_pool_work(sample_work(active_stale_job_id, "active-stale-job", false))
        .expect("active stale work should enqueue");
    let stale_dispatch = registry
        .dispatch_next()
        .expect("stale work should dispatch");
    assert_eq!(stale_dispatch.work.asic_job_id, queued_stale_job_id);
    assert!(registry.valid_jobs().contains(queued_stale_job_id));
    assert!(registry.valid_jobs().contains(active_stale_job_id));

    // Act
    registry
        .enqueue_pool_work(sample_work(current_job_id, "current-job", true))
        .expect("clean-jobs work should enqueue");

    // Assert
    assert_eq!(registry.generation().raw(), 1);
    assert!(!registry.valid_jobs().contains(queued_stale_job_id));
    assert!(!registry.valid_jobs().contains(active_stale_job_id));
    assert!(registry.valid_jobs().contains(current_job_id));
    assert!(registry.maybe_active_work(queued_stale_job_id).is_none());
    let current_dispatch = registry
        .dispatch_next()
        .expect("current work should be the only queued dispatch");
    assert_eq!(current_dispatch.work.asic_job_id, current_job_id);
}

#[test]
fn production_work_reconnect_advances_generation_and_clears_work() {
    // Arrange
    let mut registry = ProductionWorkRegistry::new();
    let clean_job_id = Bm1366JobId::new(0x60);
    let active_job_id = Bm1366JobId::new(0x68);
    registry
        .enqueue_pool_work(sample_work(clean_job_id, "clean-job", true))
        .expect("clean work should enqueue");
    assert_eq!(registry.generation().raw(), 1);
    registry
        .enqueue_pool_work(sample_work(active_job_id, "active-job", false))
        .expect("active work should enqueue");
    let _active_dispatch = registry.dispatch_next().expect("work should dispatch");
    assert!(registry.valid_jobs().contains(clean_job_id));
    assert!(registry.valid_jobs().contains(active_job_id));
    assert!(registry.maybe_active_work(clean_job_id).is_some());

    // Act
    registry.invalidate_for_reconnect();

    // Assert
    assert_eq!(registry.generation().raw(), 2);
    assert!(!registry.valid_jobs().contains(clean_job_id));
    assert!(!registry.valid_jobs().contains(active_job_id));
    assert!(registry.maybe_active_work(clean_job_id).is_none());
    assert!(matches!(
        registry.dispatch_next(),
        Err(StratumV1Error::QueueEmpty)
    ));
}

#[test]
fn production_work_records_pool_context() {
    // Arrange
    let mut registry = ProductionWorkRegistry::new();
    let job_id = Bm1366JobId::new(0x70);
    registry
        .enqueue_pool_work(sample_work(job_id, "context-job", false))
        .expect("pool work should enqueue");

    // Act
    let dispatch = registry.dispatch_next().expect("work should dispatch");
    let active = registry
        .maybe_active_work(job_id)
        .expect("dispatched work should be active");

    // Assert
    assert_eq!(dispatch.generation.raw(), 0);
    assert_eq!(active.stratum_job_id, "context-job");
    assert_eq!(active.extranonce2, "00000000");
    assert_eq!(active.ntime, 0x6470_25b5);
    assert_eq!(active.target_context.compact_nbits, 0x1705_ae3a);
    assert_eq!(
        active.target_context.maybe_pool_difficulty,
        Some(PoolDifficulty { difficulty: 1.25 })
    );
}

#[test]
fn production_work_registry_debug_redacts_queued_active_context() {
    // Arrange
    let mut registry = ProductionWorkRegistry::new();
    let job_id = Bm1366JobId::new(0x78);
    registry
        .enqueue_pool_work(sample_work(job_id, "registry-hidden-job", false))
        .expect("pool work should enqueue");
    let _dispatch = registry.dispatch_next().expect("work should dispatch");

    // Act
    let rendered = format!("{registry:?}");

    // Assert
    assert!(rendered.contains("ProductionWorkRegistry"));
    assert!(!rendered.contains("registry-hidden-job"));
    assert!(!rendered.contains("4de05269"));
    assert!(!rendered.contains("00000000"));
    assert!(!rendered.contains("1705ae3a"));
    assert!(!rendered.contains("647025b5"));
    assert!(!rendered.contains("0x78"));
}

#[test]
fn production_correlation_returns_submit_intent_for_active_generation() {
    // Arrange
    let mut registry = registry_with_dispatched_work(Bm1366JobId::new(0x80));
    let observation = ProductionNonceObservation {
        observed_generation: registry.generation(),
        result: sample_nonce_result(Bm1366JobId::new(0x80)),
    };

    // Act
    let outcome = registry.correlate_nonce_result(observation);

    // Assert
    let CorrelationOutcome::SubmitIntent(intent) = outcome else {
        panic!("active current-generation result should produce submit intent");
    };
    assert_eq!(intent.generation, PoolSessionGeneration::initial());
    assert_eq!(intent.asic_job_id, Bm1366JobId::new(0x80));
    assert_eq!(intent.submission.job_id, "correlated-job");
}

#[test]
fn production_correlation_rejects_uncorrelated_result() {
    // Arrange
    let mut registry = ProductionWorkRegistry::new();
    let observation = ProductionNonceObservation {
        observed_generation: registry.generation(),
        result: sample_nonce_result(Bm1366JobId::new(0x88)),
    };

    // Act
    let outcome = registry.correlate_nonce_result(observation);

    // Assert
    assert_eq!(
        outcome,
        CorrelationOutcome::Blocked {
            reason: ProductionAsicBlocker::JobUncorrelated
        }
    );
}

#[test]
fn production_correlation_rejects_stale_active_record() {
    // Arrange
    let mut registry = registry_with_dispatched_work(Bm1366JobId::new(0x90));
    registry.force_active_record_generation_for_test(
        Bm1366JobId::new(0x90),
        PoolSessionGeneration::initial().next(),
    );
    let observation = ProductionNonceObservation {
        observed_generation: registry.generation(),
        result: sample_nonce_result(Bm1366JobId::new(0x90)),
    };

    // Act
    let outcome = registry.correlate_nonce_result(observation);

    // Assert
    assert_eq!(
        outcome,
        CorrelationOutcome::Blocked {
            reason: ProductionAsicBlocker::WorkStale
        }
    );
}

#[test]
fn production_correlation_rejects_duplicate_result() {
    // Arrange
    let mut registry = registry_with_dispatched_work(Bm1366JobId::new(0x98));
    let observation = ProductionNonceObservation {
        observed_generation: registry.generation(),
        result: sample_nonce_result(Bm1366JobId::new(0x98)),
    };
    let _first = registry.correlate_nonce_result(observation);

    // Act
    let outcome = registry.correlate_nonce_result(observation);

    // Assert
    assert_eq!(
        outcome,
        CorrelationOutcome::Ignored {
            reason: NonSubmitReason::DuplicateCandidate
        }
    );
}

#[test]
fn production_correlation_allows_multiple_distinct_candidates_for_one_job() {
    // Arrange
    let job_id = Bm1366JobId::new(0x98);
    let mut registry = registry_with_dispatched_work(job_id);
    let first = ProductionNonceObservation {
        observed_generation: registry.generation(),
        result: sample_nonce_result(job_id),
    };
    let mut second = first;
    second.result.nonce = second.result.nonce.wrapping_add(1);

    // Act
    let first_outcome = registry.correlate_nonce_result(first);
    let second_outcome = registry.correlate_nonce_result(second);

    // Assert
    assert!(matches!(first_outcome, CorrelationOutcome::SubmitIntent(_)));
    assert!(matches!(
        second_outcome,
        CorrelationOutcome::SubmitIntent(_)
    ));
    assert_eq!(
        registry
            .maybe_active_work(job_id)
            .expect("active work should remain")
            .observed_candidate_count,
        2
    );
}

#[test]
fn production_correlation_rejects_stored_target_context_drift() {
    // Arrange
    let mut registry = registry_with_dispatched_work(Bm1366JobId::new(0xa0));
    registry.force_active_compact_nbits_for_test(Bm1366JobId::new(0xa0), 0x1d00_ffff);
    let observation = ProductionNonceObservation {
        observed_generation: registry.generation(),
        result: sample_nonce_result(Bm1366JobId::new(0xa0)),
    };

    // Act
    let outcome = registry.correlate_nonce_result(observation);

    // Assert
    assert_eq!(
        outcome,
        CorrelationOutcome::Blocked {
            reason: ProductionAsicBlocker::TargetMismatch
        }
    );
}

#[test]
fn production_correlation_rejects_wrong_session_generation() {
    // Arrange
    let mut registry = registry_with_dispatched_work(Bm1366JobId::new(0xa8));
    let observation = ProductionNonceObservation {
        observed_generation: registry.generation().next(),
        result: sample_nonce_result(Bm1366JobId::new(0xa8)),
    };

    // Act
    let outcome = registry.correlate_nonce_result(observation);

    // Assert
    assert_eq!(
        outcome,
        CorrelationOutcome::Blocked {
            reason: ProductionAsicBlocker::WrongSession
        }
    );
}

#[test]
fn submit_intent_debug_redacts_raw_context() {
    // Arrange
    let mut registry = registry_with_dispatched_work(Bm1366JobId::new(0xb0));
    let outcome = registry.correlate_nonce_result(ProductionNonceObservation {
        observed_generation: registry.generation(),
        result: sample_nonce_result(Bm1366JobId::new(0xb0)),
    });
    let CorrelationOutcome::SubmitIntent(intent) = outcome else {
        panic!("active work should produce submit intent");
    };

    // Act
    let rendered = format!("{intent:?}");

    // Assert
    assert!(rendered.contains("SubmitIntent"));
    assert!(rendered.contains("submit_context"));
    assert!(!rendered.contains("correlated-job"));
    assert!(!rendered.contains("00000000"));
    assert!(!rendered.contains("12345678"));
    assert!(!rendered.contains("00002000"));
    assert!(!rendered.contains("1705ae3a"));
}

#[test]
fn no_debug_for_submit_context_leaks_raw_values() {
    // Arrange
    let mut registry = registry_with_dispatched_work(Bm1366JobId::new(0xb8));
    let outcome = registry.correlate_nonce_result(ProductionNonceObservation {
        observed_generation: registry.generation(),
        result: sample_nonce_result(Bm1366JobId::new(0xb8)),
    });

    // Act
    let rendered = format!("{outcome:?}");

    // Assert
    assert!(rendered.contains("CorrelationOutcome"));
    assert!(!rendered.contains("correlated-job"));
    assert!(!rendered.contains("00000000"));
    assert!(!rendered.contains("12345678"));
    assert!(!rendered.contains("00002000"));
    assert!(!rendered.contains("1705ae3a"));
}

#[test]
fn seventeen_dispatch_stride_wraparound_replaces_first_job_record() {
    // Arrange: stride-8 job ids as generated by the live runtime
    // (wrapping_add(8) % 128), so the 17th dispatch reuses id 0.
    // Upstream frees the prior job at the reused slot
    // (reference/esp-miner/components/asic/bm1366.c:323-325); the
    // registry mirrors that with HashMap insert replacement.
    let mut registry = ProductionWorkRegistry::new();
    let mut job_id_raw: u8 = 0;
    let mut dispatched_ids = Vec::new();

    // Act: 17 sequential enqueue+dispatch cycles wrap the 16-slot id space.
    for round in 0..17_u8 {
        let job_id = Bm1366JobId::new(job_id_raw);
        job_id_raw = job_id_raw.wrapping_add(8) % 128;
        let stratum_job = format!("wraparound-job-{round}");
        let mut work = sample_work(job_id, &stratum_job, false);
        work.maybe_pool_difficulty = Some(PoolDifficulty {
            difficulty: 1.0e-30,
        });
        registry
            .enqueue_pool_work(work)
            .expect("wraparound work should enqueue");
        let dispatch = registry
            .dispatch_next()
            .expect("wraparound work should dispatch");
        dispatched_ids.push(dispatch.work.asic_job_id.raw());
    }
    let correlation = registry.correlate_nonce_result(ProductionNonceObservation {
        observed_generation: registry.generation(),
        result: sample_nonce_result(Bm1366JobId::new(0)),
    });

    // Assert: ids run 0,8,...,120 then wrap to 0; the reused id-0 slot
    // holds the 17th job and correlation resolves to the NEW job's work.
    let expected_ids: Vec<u8> = (0..17_u16).map(|round| ((round * 8) % 128) as u8).collect();
    assert_eq!(dispatched_ids, expected_ids);
    let replaced = registry
        .maybe_active_work(Bm1366JobId::new(0))
        .expect("reused id 0 should stay active");
    assert_eq!(replaced.stratum_job_id, "wraparound-job-16");
    let CorrelationOutcome::SubmitIntent(intent) = correlation else {
        panic!("stale-id correlation should resolve to the replacing job");
    };
    assert_eq!(intent.submission.job_id, "wraparound-job-16");
}

fn registry_with_dispatched_work(job_id: Bm1366JobId) -> ProductionWorkRegistry {
    let mut registry = ProductionWorkRegistry::new();
    let mut work = sample_work(job_id, "correlated-job", false);
    work.maybe_pool_difficulty = Some(PoolDifficulty {
        difficulty: 1.0e-30,
    });
    registry
        .enqueue_pool_work(work)
        .expect("pool work should enqueue");
    let _dispatch = registry.dispatch_next().expect("work should dispatch");
    registry
}

fn sample_nonce_result(job_id: Bm1366JobId) -> Bm1366NonceResult {
    Bm1366NonceResult {
        job_id,
        nonce: 0x1234_5678,
        asic_index: 0,
        core_id: 1,
        small_core_id: 0,
        version_bits: 0x0000_2000,
    }
}

fn sample_work(job_id: Bm1366JobId, stratum_job_id: &str, clean_jobs: bool) -> MiningWork {
    MiningWorkBuilder::new(
        MiningNotify {
            job_id: stratum_job_id.to_owned(),
            prev_block_hash: "00".repeat(32),
            coinbase_1: "0200000001".to_owned(),
            coinbase_2: "ffffffff".to_owned(),
            merkle_branches: Vec::new(),
            version: 0x2000_0004,
            nbits: 0x1705_ae3a,
            ntime: 0x6470_25b5,
            clean_jobs,
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
